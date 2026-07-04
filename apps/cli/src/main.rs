use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use clap::{Parser, Subcommand};
use omega_core::http::HttpClientFactory;
use omega_core::Provider;
use omega_database::repositories::provider_channel_repository::ProviderChannelRepository;
use omega_database::repositories::channel_repository::ChannelRepository;
use omega_database::repositories::match_repository::MatchRepository;
use omega_database::repositories::review_repository::ReviewRepository;
use omega_database::repositories::export_repository::ExportRepository;
use omega_matcher::{AliasEngine, create_review_item, resolve_channel_match};
use omega_database::{connect, migrate};
use omega_providers::iptv_org::parse_iptv_org_m3u;
use omega_providers::{JioProvider, Zee5Provider};
use omega_playlist::{build_indexes, build_m3u, Playlist, PlaylistEntry};
use omega_database::repositories::programme_repository::ProgrammeRepository;
use omega_providers::jio::epg::fetch_jio_epg_for_channel;
use omega_xmltv::{
    build_xmltv_string,
    write_gzip_file,
    write_xmltv_file,
    XmltvChannel,
    XmltvProgramme,
};

#[derive(Debug, Parser)]
#[command(
    name = "omega",
    version,
    about = "Omega IPTV Rust Platform",
    long_about = "Ultra-performance IPTV, XMLTV, playlist, matching, and provider orchestration platform."
)]
struct Cli {
    #[arg(long, default_value = "sqlite://omega.db")]
    database_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the complete production build pipeline
    Build {
        #[arg(long, default_value_t = true)]
        persist: bool,

        #[arg(long)]
        epg_limit: Option<usize>,

        #[arg(long, default_value_t = 0)]
        start_offset: i32,

        #[arg(long, default_value_t = 1)]
        end_offset: i32,

        #[arg(long, default_value_t = false)]
        skip_epg: bool,
    },  
        
    /// Fetch provider data
    Fetch {
        #[command(subcommand)]
        provider: FetchCommand,
    },

    /// Parse source files such as IPTV-org M3U
    Parse {
        #[command(subcommand)]
        source: ParseCommand,
    },

    /// Run unified channel matcher
    Match,

    /// Fetch and persist EPG programme data
    Epg {
        #[command(subcommand)]
        provider: EpgCommand,
    },
   
   /// Generate XMLTV files
    Xmltv,

    /// Generate playlists and indexes
    Playlist,
    
    /// Publish generated artifacts to output/public
    Publish,

    /// Run system health checks
    Doctor,
   
   /// Export generated database artifacts to JSON files
    Export {
        #[command(subcommand)]
        target: ExportCommand,
    },
}

#[derive(Debug, Subcommand)]
enum FetchCommand {
    Zee5 {
        #[arg(long, default_value_t = false)]
        persist: bool,
    },
    Jio {
        #[arg(long, default_value_t = false)]
        persist: bool,
    },
    All {
        #[arg(long, default_value_t = false)]
        persist: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ParseCommand {
    IptvOrg {
        #[arg(long, default_value = "in.m3u")]
        input: String,

        #[arg(long, default_value_t = false)]
        persist: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ExportCommand {
    Unified,
    Review,
    Matches,
    All,
}

#[derive(Debug, Subcommand)]
enum EpgCommand {
    Jio {
        #[arg(long)]
        limit: Option<usize>,

        #[arg(long, default_value_t = 0)]
        start_offset: i32,

        #[arg(long, default_value_t = 1)]
        end_offset: i32,

        #[arg(long, default_value_t = false)]
        persist: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let http = HttpClientFactory::new(
        30,
        "Mozilla/5.0 Omega-IPTV-Rust/1.0",
    );

    let client = http.build()?;

    match cli.command {
        Commands::Build {
            persist,
            epg_limit,
            start_offset,
            end_offset,
            skip_epg,
        } => {
            run_full_build_pipeline(
                client.clone(),
                &cli.database_url,
                persist,
                epg_limit,
                start_offset,
                end_offset,
                skip_epg,
           )
           .await?;
        },

        Commands::Fetch { provider } => match provider {
            FetchCommand::Zee5 { persist } => {
                let provider = Zee5Provider::new(client.clone());

                println!("📡 Checking Zee5 provider health...");
                let healthy = provider.health_check().await?;
                println!("Zee5 health: {}", healthy);

                println!("📺 Fetching Zee5 channels...");
                let channels = provider.fetch_channels().await?;
                println!("✅ Zee5 channels fetched: {}", channels.len());

                for channel in channels.iter().take(10) {
                    println!(" - {} ({})", channel.name, channel.id);
                }

                if persist {
                    let pool = connect_and_migrate(&cli.database_url).await?;
                    let repo = ProviderChannelRepository::new(pool);
                    let saved = persist_channels(&repo, &channels).await?;

                    println!("💾 Zee5 channels saved: {}", saved);
                    println!("✅ Database: {}", cli.database_url);
                }
            }

            FetchCommand::Jio { persist } => {
                let provider = JioProvider::new(client.clone());

                println!("📡 Checking Jio provider health...");
                let healthy = provider.health_check().await?;
                println!("Jio health: {}", healthy);

                println!("📺 Fetching Jio channels...");
                let channels = provider.fetch_channels().await?;
                println!("✅ Jio channels fetched: {}", channels.len());

                for channel in channels.iter().take(10) {
                    println!(" - {} ({})", channel.name, channel.id);
                }

                if persist {
                    let pool = connect_and_migrate(&cli.database_url).await?;
                    let repo = ProviderChannelRepository::new(pool);
                    let saved = persist_channels(&repo, &channels).await?;

                    println!("💾 Jio channels saved: {}", saved);
                    println!("✅ Database: {}", cli.database_url);
                }
            }

            FetchCommand::All { persist } => {
                let zee5 = Zee5Provider::new(client.clone());
                let jio = JioProvider::new(client.clone());

                println!("📡 Checking providers...");

                let zee5_health = zee5.health_check().await.unwrap_or(false);
                let jio_health = jio.health_check().await.unwrap_or(false);

                println!("Zee5 health: {}", zee5_health);
                println!("Jio health : {}", jio_health);

                let db_repo = if persist {
                    let pool = connect_and_migrate(&cli.database_url).await?;
                    Some(ProviderChannelRepository::new(pool))
                } else {
                    None
                };

                let mut total_saved = 0usize;

                if zee5_health {
                    println!("📺 Fetching Zee5 channels...");
                    let channels = zee5.fetch_channels().await?;
                    println!("✅ Zee5 channels fetched: {}", channels.len());

                    if let Some(repo) = &db_repo {
                        let saved = persist_channels(repo, &channels).await?;
                        total_saved += saved;
                        println!("💾 Zee5 channels saved: {}", saved);
                    }
                }

                if jio_health {
                    println!("📺 Fetching Jio channels...");
                    let channels = jio.fetch_channels().await?;
                    println!("✅ Jio channels fetched: {}", channels.len());

                    if let Some(repo) = &db_repo {
                        let saved = persist_channels(repo, &channels).await?;
                        total_saved += saved;
                        println!("💾 Jio channels saved: {}", saved);
                    }
                }

                if persist {
                    println!("✅ Total provider channels saved: {}", total_saved);
                    println!("✅ Database: {}", cli.database_url);
                }
            }
        },

        Commands::Parse { source } => match source {
            ParseCommand::IptvOrg { input, persist } => {
                println!("📄 Parsing IPTV-org M3U");
                println!("Input: {}", input);

                let content = fs::read_to_string(&input)?;
                let channels = parse_iptv_org_m3u(&content)?;

                println!("✅ IPTV-org channels parsed: {}", channels.len());

                for channel in channels.iter().take(10) {
                    println!(" - {} ({})", channel.name, channel.id);
                }

                if persist {
                    let pool = connect_and_migrate(&cli.database_url).await?;
                    let repo = ProviderChannelRepository::new(pool);
                    let saved = persist_channels(&repo, &channels).await?;

                    println!("💾 IPTV-org channels saved: {}", saved);
                    println!("✅ Database: {}", cli.database_url);
                }
            }
        },

        Commands::Match => {
            run_match_pipeline(&cli.database_url).await?;
        }

        Commands::Epg { provider } => match provider {
            EpgCommand::Jio {
                limit,
                start_offset,
                end_offset,
                persist,
            } => {
                run_jio_epg_pipeline(
                    client.clone(),
                    &cli.database_url,
                    limit,
                    start_offset,
                    end_offset,
                    persist,
                )
                .await?;
            }
        },

        Commands::Xmltv => {
            run_xmltv_pipeline(&cli.database_url).await?;
        }

        Commands::Playlist => {
            run_playlist_pipeline(&cli.database_url).await?;
        }
        
        Commands::Export { target } => {
            run_export_pipeline(&cli.database_url, target).await?;
        }

        Commands::Publish => {
            run_publish_pipeline()?;
        }

        Commands::Doctor => {
            println!("🩺 omega doctor: workspace OK");

            let zee5 = Zee5Provider::new(client.clone());
            let jio = JioProvider::new(client.clone());

            let zee5_health = zee5.health_check().await.unwrap_or(false);
            let jio_health = jio.health_check().await.unwrap_or(false);

            println!("Zee5 provider: {}", if zee5_health { "OK" } else { "FAILED" });
            println!("Jio provider : {}", if jio_health { "OK" } else { "FAILED" });

            let pool = connect_and_migrate(&cli.database_url).await?;
            let repo = ProviderChannelRepository::new(pool);

            let zee5_count = repo.count_by_provider("zee5").await.unwrap_or(0);
            let jio_count = repo.count_by_provider("jio").await.unwrap_or(0);
            let iptv_count = repo.count_by_provider("iptv_org").await.unwrap_or(0);

            println!("Database      : OK");
            println!("DB Zee5 count : {}", zee5_count);
            println!("DB Jio count  : {}", jio_count);
            println!("DB IPTV count : {}", iptv_count);
        }
    }

    Ok(())
}

async fn connect_and_migrate(database_url: &str) -> anyhow::Result<omega_database::DbPool> {
    let pool = connect(database_url).await?;
    migrate(&pool).await?;
    Ok(pool)
}

async fn persist_channels(
    repo: &ProviderChannelRepository,
    channels: &[omega_core::Channel],
) -> anyhow::Result<usize> {
    let mut saved = 0usize;

    for channel in channels {
        repo.upsert(channel).await?;
        saved += 1;
    }

    Ok(saved)
}

async fn run_match_pipeline(database_url: &str) -> anyhow::Result<()> {
    use std::collections::HashSet;

    println!("🧠 Starting unified match pipeline");

    let pool = connect_and_migrate(database_url).await?;

    let provider_repo = ProviderChannelRepository::new(pool.clone());
    let channel_repo = ChannelRepository::new(pool.clone());
    let match_repo = MatchRepository::new(pool.clone());
    let review_repo = ReviewRepository::new(pool.clone());
    clear_match_outputs(&pool).await?;
    let jio_channels = provider_repo.list_by_provider("jio").await?;
    let iptv_channels = provider_repo.list_by_provider("iptv_org").await?;
    let zee5_channels = provider_repo.list_by_provider("zee5").await?;

    println!("📦 Jio channels      : {}", jio_channels.len());
    println!("📦 IPTV-org channels : {}", iptv_channels.len());
    println!("📦 Zee5 channels     : {}", zee5_channels.len());

    if jio_channels.is_empty() {
        println!("⚠️ No Jio channels found. Run:");
        println!("cargo run -p omega-cli -- fetch jio --persist");
        return Ok(());
    }

    if iptv_channels.is_empty() {
        println!("⚠️ No IPTV-org channels found. Run:");
        println!("cargo run -p omega-cli -- parse iptv-org --input in.m3u --persist");
        return Ok(());
    }

    let alias_engine = AliasEngine::new();

    let mut matched = 0usize;
    let mut reviewed = 0usize;

    let mut used_iptv_targets: HashSet<String> = HashSet::new();

    for source in jio_channels {
        let available_targets = iptv_channels
            .iter()
            .filter(|target| !used_iptv_targets.contains(&target.id))
            .cloned()
            .collect::<Vec<_>>();

        if available_targets.is_empty() {
            let review = create_review_item(
                source,
                &[],
                &alias_engine,
                "no unused IPTV-org targets left",
            );

            review_repo.insert(&review).await?;
            reviewed += 1;
            continue;
        }

        let result = resolve_channel_match(
            source.clone(),
            &available_targets,
            &alias_engine,
            0.78,
            0.60,
        );

        match_repo.insert(&result).await?;

        if let Some(unified) = &result.unified {
            if let Some(target) = &result.target {
                used_iptv_targets.insert(target.id.clone());
            }

            channel_repo.upsert(unified).await?;
            matched += 1;
        } else {
            let review = create_review_item(
                source,
                &available_targets,
                &alias_engine,
                result
                    .reason
                    .clone()
                    .unwrap_or_else(|| "no confident IPTV-org match".to_string()),
            );

            review_repo.insert(&review).await?;
            reviewed += 1;
        }
    }

    println!("✅ Unified matches saved : {}", matched);
    println!("⚠️ Review items saved    : {}", reviewed);
    println!("🔒 IPTV targets consumed : {}", used_iptv_targets.len());
    println!("✅ Database              : {}", database_url);

    Ok(())
}

async fn run_export_pipeline(
    database_url: &str,
    target: ExportCommand,
) -> anyhow::Result<()> {
    println!("📦 Starting export pipeline");

    let pool = connect_and_migrate(database_url).await?;
    let repo = ExportRepository::new(pool);

    fs::create_dir_all("output")?;

    match target {
        ExportCommand::Unified => {
            export_unified(&repo).await?;
        }

        ExportCommand::Review => {
            export_review(&repo).await?;
        }

        ExportCommand::Matches => {
            export_matches(&repo).await?;
        }

        ExportCommand::All => {
            export_unified(&repo).await?;
            export_review(&repo).await?;
            export_matches(&repo).await?;
        }
    }

    println!("✅ Export pipeline complete");

    Ok(())
}

async fn export_unified(repo: &ExportRepository) -> anyhow::Result<()> {
    let data = repo.unified_channels_json().await?;
    write_json_file("output/unified_channels.json", &data)?;

    println!("✅ output/unified_channels.json entries: {}", data.len());

    Ok(())
}

async fn export_review(repo: &ExportRepository) -> anyhow::Result<()> {
    let data = repo.reviews_json().await?;
    write_json_file("output/review.json", &data)?;

    println!("✅ output/review.json entries: {}", data.len());

    Ok(())
}

async fn export_matches(repo: &ExportRepository) -> anyhow::Result<()> {
    let data = repo.matches_json().await?;
    write_json_file("output/matches.json", &data)?;

    println!("✅ output/matches.json entries: {}", data.len());

    Ok(())
}

fn write_json_file(
    path: impl AsRef<Path>,
    data: &[serde_json::Value],
) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let text = serde_json::to_string_pretty(data)?;
    fs::write(path, text)?;

    Ok(())
}

async fn clear_match_outputs(pool: &omega_database::DbPool) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM matches").execute(pool).await?;
    sqlx::query("DELETE FROM reviews").execute(pool).await?;
    sqlx::query("DELETE FROM channels").execute(pool).await?;

    Ok(())
}

async fn run_playlist_pipeline(database_url: &str) -> anyhow::Result<()> {
    println!("📺 Starting playlist generation pipeline");

    let pool = connect_and_migrate(database_url).await?;
    let repo = ExportRepository::new(pool);

    let unified = repo.unified_channels_json().await?;

    if unified.is_empty() {
        println!("⚠️ No unified channels found.");
        println!("Run this first:");
        println!("cargo run -p omega-cli -- match");
        return Ok(());
    }

    fs::create_dir_all("output")?;
    fs::create_dir_all("output/playlists")?;

    let mut playlist = Playlist::new("omega");

    for item in unified {
        let name = json_string(&item, "display_name")
            .or_else(|| json_string(&item, "canonical_name"))
            .unwrap_or_else(|| "Unknown Channel".to_string());

        let Some(url) = json_string(&item, "url") else {
            continue;
        };

        if url.trim().is_empty() {
            continue;
        }

        let tvg_id = json_string(&item, "tvg_id");
        let tvg_name = Some(name.clone());
        let logo = json_string(&item, "logo");

        let group_name = json_string(&item, "group")
            .or_else(|| json_string(&item, "category"))
            .unwrap_or_else(|| "General".to_string());

        let country = json_string(&item, "country")
            .unwrap_or_else(|| "IN".to_string());

        let group = Some(format!("{} ({})", group_name, country.to_uppercase()));

        let raw_extinf = format!(
            "#EXTINF:-1 tvg-id=\"{}\" tvg-name=\"{}\" tvg-logo=\"{}\" group-title=\"{}\",{}",
            tvg_id.clone().unwrap_or_default(),
            name,
            logo.clone().unwrap_or_default(),
            group.clone().unwrap_or_else(|| "General (IN)".to_string()),
            name
        );

        playlist.push(PlaylistEntry {
            tvg_id,
            tvg_name,
            name,
            label: None,
            group,
            logo,
            url,
            raw_extinf,
        });
    }

    let m3u = build_m3u(&playlist);

    fs::write("output/omega.m3u", &m3u)?;
    fs::write("output/playlists/omega.m3u", &m3u)?;

    build_indexes("output/playlists")?;

    println!("✅ output/omega.m3u entries              : {}", playlist.len());
    println!("✅ output/playlists/omega.m3u            : generated");
    println!("✅ output/playlists/index.m3u            : generated");
    println!("✅ output/playlists/index.genre.m3u      : generated");
    println!("✅ Playlist pipeline complete");

    Ok(())
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn run_jio_epg_pipeline(
    client: reqwest::Client,
    database_url: &str,
    limit: Option<usize>,
    start_offset: i32,
    end_offset: i32,
    persist: bool,
) -> anyhow::Result<()> {
    println!("🗓️ Starting Jio EPG pipeline");
    println!("Offset range: {} -> {}", start_offset, end_offset);

    let pool = connect_and_migrate(database_url).await?;
    let provider_repo = ProviderChannelRepository::new(pool.clone());
    let programme_repo = ProgrammeRepository::new(pool);

    let mut channels = provider_repo.list_by_provider("jio").await?;

    channels.retain(|channel| !channel.hidden);

    if let Some(limit) = limit {
        channels.truncate(limit);
    }

    println!("📺 Jio channels selected: {}", channels.len());

    let mut total_programmes = 0usize;
    let mut saved_programmes = 0usize;

    for (index, channel) in channels.iter().enumerate() {
        println!(
            "📡 [{}/{}] Fetching EPG: {} ({})",
            index + 1,
            channels.len(),
            channel.name,
            channel.id
        );

        match fetch_jio_epg_for_channel(
            &client,
            channel,
            start_offset,
            end_offset,
        )
        .await
        {
            Ok(programmes) => {
                println!("   ✅ programmes: {}", programmes.len());
                total_programmes += programmes.len();

                if persist {
                    for programme in &programmes {
                        programme_repo.insert_ignore(programme).await?;
                        saved_programmes += 1;
                    }
                }
            }

            Err(error) => {
                println!("   ⚠️ failed: {}", error);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    println!("✅ Jio EPG programmes fetched : {}", total_programmes);

    if persist {
        println!("💾 Jio EPG programmes saved   : {}", saved_programmes);
        println!("✅ Database                   : {}", database_url);
    }

    Ok(())
}

async fn run_xmltv_pipeline(database_url: &str) -> anyhow::Result<()> {
    println!("🗓️ Starting XMLTV generation pipeline");

    let pool = connect_and_migrate(database_url).await?;

    let export_repo = ExportRepository::new(pool.clone());
    let programme_repo = ProgrammeRepository::new(pool);

    let channel_rows = export_repo.xmltv_channels_json().await?;
    let map_rows = export_repo.programme_channel_map_json().await?;
    let programmes = programme_repo.list_all().await?;

    println!("📺 XMLTV channels available : {}", channel_rows.len());
    println!("🧾 Programme rows available : {}", programmes.len());
    println!("🔗 Channel map rows         : {}", map_rows.len());

    if channel_rows.is_empty() {
        println!("⚠️ No unified channels found. Run:");
        println!("cargo run -p omega-cli -- match");
        return Ok(());
    }

    if programmes.is_empty() {
        println!("⚠️ No programmes found. Run:");
        println!("cargo run -p omega-cli -- epg jio --limit 20 --persist");
        return Ok(());
    }

    let mut channel_map: HashMap<String, String> = HashMap::new();

    for row in map_rows {
        let Some(source_channel_id) = json_string(&row, "source_channel_id") else {
            continue;
        };

        let Some(tvg_id) = json_string(&row, "tvg_id") else {
            continue;
        };

        channel_map.insert(source_channel_id, tvg_id);
    }

    let mut xml_channels = Vec::new();

    for row in channel_rows {
        let Some(tvg_id) = json_string(&row, "tvg_id") else {
            continue;
        };

        let name = json_string(&row, "display_name")
            .unwrap_or_else(|| tvg_id.clone());

        let mut channel = XmltvChannel::new(tvg_id, name);
        channel.icon = json_string(&row, "logo");

        xml_channels.push(channel);
    }

    let mut xml_programmes = Vec::new();

    for programme in programmes {
        let Some(tvg_id) = channel_map.get(&programme.channel_id) else {
            continue;
        };

        let mut xml_programme = XmltvProgramme::from(&programme);
        xml_programme.channel_id = tvg_id.clone();

        xml_programmes.push(xml_programme);
    }

    println!("📺 XMLTV channels written    : {}", xml_channels.len());
    println!("🧾 XMLTV programmes written  : {}", xml_programmes.len());

    fs::create_dir_all("output")?;

    write_xmltv_file(
        "output/omega.xml",
        &xml_channels,
        &xml_programmes,
        "omega-iptv-rust",
        330,
    )?;

    let xml = build_xmltv_string(
        &xml_channels,
        &xml_programmes,
        "omega-iptv-rust",
        330,
    )?;

    write_gzip_file(
        "output/omega.xml.gz",
        xml.as_bytes(),
    )?;

    println!("✅ output/omega.xml          : generated");
    println!("✅ output/omega.xml.gz       : generated");
    println!("✅ XMLTV pipeline complete");

    Ok(())
}

async fn run_full_build_pipeline(
    client: reqwest::Client,
    database_url: &str,
    persist: bool,
    epg_limit: Option<usize>,
    start_offset: i32,
    end_offset: i32,
    skip_epg: bool,
) -> anyhow::Result<()> {
    println!("🚀 Starting full Omega build pipeline");

    let pool = connect_and_migrate(database_url).await?;
    let provider_repo = ProviderChannelRepository::new(pool);

    let zee5 = Zee5Provider::new(client.clone());
    let jio = JioProvider::new(client.clone());

    println!("📡 Step 1: Fetch Zee5 channels");
    let zee5_channels = zee5.fetch_channels().await?;
    println!("✅ Zee5 channels fetched: {}", zee5_channels.len());

    if persist {
        let saved = persist_channels(&provider_repo, &zee5_channels).await?;
        println!("💾 Zee5 channels saved: {}", saved);
    }

    println!("📡 Step 2: Fetch Jio channels");
    let jio_channels = jio.fetch_channels().await?;
    println!("✅ Jio channels fetched: {}", jio_channels.len());

    if persist {
        let saved = persist_channels(&provider_repo, &jio_channels).await?;
        println!("💾 Jio channels saved: {}", saved);
    }

    println!("📄 Step 3: Parse IPTV-org");

    if Path::new("in.m3u").exists() {
        let content = fs::read_to_string("in.m3u")?;
        let iptv_channels = parse_iptv_org_m3u(&content)?;

        println!("✅ IPTV-org channels parsed: {}", iptv_channels.len());

        if persist {
            let saved = persist_channels(&provider_repo, &iptv_channels).await?;
            println!("💾 IPTV-org channels saved: {}", saved);
        }
    } else {
        println!("⚠️ in.m3u not found, skipping IPTV-org parse");
        println!("Download it with:");
        println!("curl -L -o in.m3u https://raw.githubusercontent.com/iptv-org/iptv/refs/heads/master/streams/in.m3u");
    }

    if !persist {
        println!("⚠️ persist=false, stopping before DB-dependent pipelines");
        return Ok(());
    }

    println!("🧠 Step 4: Run matcher");
    run_match_pipeline(database_url).await?;

    println!("📦 Step 5: Export JSON artifacts");
    run_export_pipeline(database_url, ExportCommand::All).await?;

    println!("📺 Step 6: Generate M3U playlist");
    run_playlist_pipeline(database_url).await?;

    if skip_epg {
        println!("⏭️ Step 7: EPG skipped");
    } else {
        println!("🗓️ Step 7: Fetch Jio EPG");
        run_jio_epg_pipeline(
            client.clone(),
            database_url,
            epg_limit,
            start_offset,
            end_offset,
            true,
        )
        .await?;

        println!("🗓️ Step 8: Generate XMLTV");
        run_xmltv_pipeline(database_url).await?;
    }

    println!("🚢 Step 9: Publish public artifacts");
    run_publish_pipeline()?;

    validate_output_files()?;

    println!("🎉 Full Omega build pipeline complete");
    println!("✅ Database: {}", database_url);
    println!("✅ Output directory: output/");

    Ok(())
}


fn validate_output_files() -> anyhow::Result<()> {
    println!("🔎 Validating output artifacts");

    let required_files = [
        "output/unified_channels.json",
        "output/review.json",
        "output/matches.json",
        "output/omega.m3u",
        "output/omega.xml",
        "output/omega.xml.gz",
        "output/playlists/omega.m3u",
        "output/playlists/index.m3u",
        "output/playlists/index.genre.m3u",
    ];

    for file in required_files {
        let metadata = std::fs::metadata(file)
            .map_err(|error| anyhow::anyhow!("missing required output file {}: {}", file, error))?;

        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("output file is empty: {}", file));
        }

        println!("✅ {} [{} bytes]", file, metadata.len());
    }

    validate_json_array_file("output/unified_channels.json")?;
    validate_json_array_file("output/review.json")?;
    validate_json_array_file("output/matches.json")?;

    let playlist = std::fs::read_to_string("output/omega.m3u")?;

    if !playlist.starts_with("#EXTM3U") {
        return Err(anyhow::anyhow!("output/omega.m3u does not start with #EXTM3U"));
    }

    if !playlist.contains("#EXTINF") {
        return Err(anyhow::anyhow!("output/omega.m3u does not contain #EXTINF entries"));
    }

    let xml = std::fs::read_to_string("output/omega.xml")?;

    let lt = char::from(60);
    let amp = char::from(38);

    let xml_decl = format!("{}?xml", lt);
    let tv_tag = format!("{}tv", lt);
    let channel_tag = format!("{}channel", lt);
    let programme_tag = format!("{}programme", lt);

    let escaped_xml_decl = format!("{}lt;?xml", amp);
    let escaped_programme = format!("{}lt;programme", amp);

    if !xml.starts_with(&xml_decl) {
        return Err(anyhow::anyhow!("output/omega.xml does not start with real XML declaration"));
    }

    if xml.starts_with(&escaped_xml_decl) {
        return Err(anyhow::anyhow!("output/omega.xml starts with escaped XML declaration"));
    }

    if !xml.contains(&tv_tag) {
        return Err(anyhow::anyhow!("output/omega.xml does not contain real tv tag"));
    }

    if !xml.contains(&channel_tag) {
        return Err(anyhow::anyhow!("output/omega.xml does not contain real channel tags"));
    }

    if !xml.contains(&programme_tag) {
        return Err(anyhow::anyhow!("output/omega.xml does not contain real programme tags"));
    }

    if xml.contains(&escaped_programme) {
        return Err(anyhow::anyhow!("output/omega.xml contains escaped programme tags"));
    }

    println!("✅ Output validation complete");

    Ok(())
}

fn validate_json_array_file(path: &str) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&text)?;

    let Some(array) = value.as_array() else {
        return Err(anyhow::anyhow!("{} is not a JSON array", path));
    };

    if array.is_empty() {
        return Err(anyhow::anyhow!("{} contains an empty JSON array", path));
    }

    Ok(())
}


fn run_publish_pipeline() -> anyhow::Result<()> {
    println!("🚢 Starting publish pipeline");

    let public_dir = Path::new("output/public");

    if public_dir.exists() {
        std::fs::remove_dir_all(public_dir)?;
    }

    std::fs::create_dir_all(public_dir)?;

    copy_required_output("output/omega.m3u", "output/public/omega.m3u")?;
    copy_required_output("output/omega.xml", "output/public/omega.xml")?;
    copy_required_output("output/omega.xml.gz", "output/public/omega.xml.gz")?;
    copy_required_output("output/unified_channels.json", "output/public/unified_channels.json")?;
    copy_required_output("output/review.json", "output/public/review.json")?;
    copy_required_output("output/matches.json", "output/public/matches.json")?;

    if Path::new("output/playlists").exists() {
        copy_dir_all("output/playlists", "output/public/playlists")?;
    }

    write_public_manifest()?;

    println!("✅ output/public generated");
    println!("✅ output/public/manifest.json generated");
    println!("✅ Publish pipeline complete");

    Ok(())
}

fn copy_required_output(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if !from.exists() {
        return Err(anyhow::anyhow!("missing required publish input: {}", from.display()));
    }

    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(from, to)?;

    Ok(())
}

fn copy_dir_all(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    std::fs::create_dir_all(to)?;

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(src_path, dst_path)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::copy(src_path, dst_path)?;
        }
    }

    Ok(())
}

fn write_public_manifest() -> anyhow::Result<()> {
    let files = [
        "omega.m3u",
        "omega.xml",
        "omega.xml.gz",
        "unified_channels.json",
        "review.json",
        "matches.json",
    ];

    let mut artifacts = Vec::new();

    for file in files {
        let path = PathBuf::from("output/public").join(file);
        let metadata = std::fs::metadata(&path)?;

        artifacts.push(serde_json::json!({
            "path": file,
            "bytes": metadata.len(),
            "sha256": sha256_file(&path)?,
        }));
    }

    let manifest = serde_json::json!({
        "name": "omega-iptv-rust",
        "generated_at_utc": chrono::Utc::now().to_rfc3339(),
        "artifacts": artifacts
    });

    let text = serde_json::to_string_pretty(&manifest)?;
    std::fs::write("output/public/manifest.json", text)?;

    Ok(())
}

fn sha256_file(path: impl AsRef<Path>) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();

    hasher.update(bytes);

    let digest = hasher.finalize();

    Ok(format!("{:x}", digest))
}

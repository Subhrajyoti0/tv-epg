# Omega IPTV Rust Release Guide

## Overview

Omega IPTV Rust is a production-grade Rust IPTV pipeline.

It supports:
- Zee5 channel fetching
- JioTV channel fetching
- IPTV-org M3U parsing
- SQLite persistence
- Jio to IPTV-org stream matching
- Review generation
- Unified JSON export
- M3U playlist generation
- Jio EPG fetching
- XMLTV generation
- XMLTV gzip generation
- Public artifact publishing for GitHub Pages

---

## Main Safe Build Command

Use this on a local VM:

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- build --epg-limit 20 --start-offset 0 --end-offset 1

This runs:

    Fetch Zee5
    Fetch Jio
    Parse IPTV-org
    Match channels
    Export JSON
    Generate M3U
    Fetch limited Jio EPG
    Generate XMLTV
    Publish public files
    Validate outputs

---

## Fast Build Without EPG

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- build --skip-epg

---

## Full EPG Build

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- build --start-offset 0 --end-offset 5

---

## Health Check

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- doctor

---

## Download IPTV-org India M3U

    curl -L -o in.m3u https://raw.githubusercontent.com/iptv-org/iptv/refs/heads/master/streams/in.m3u

---

## Manual Pipeline Commands

### Fetch Providers

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- fetch all --persist

### Parse IPTV-org

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- parse iptv-org --input in.m3u --persist

### Match Channels

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- match

### Export JSON

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- export all

### Generate Playlist

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- playlist

### Fetch Test Jio EPG

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- epg jio --limit 20 --start-offset 0 --end-offset 1 --persist

### Generate XMLTV

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- xmltv

### Publish Output

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- publish

---

## Output Files

Main output:

    output/unified_channels.json
    output/review.json
    output/matches.json
    output/omega.m3u
    output/omega.xml
    output/omega.xml.gz

Playlist output:

    output/playlists/omega.m3u
    output/playlists/index.m3u
    output/playlists/index.genre.m3u
    output/playlists/countries/
    output/playlists/genres/

Public output:

    output/public/omega.m3u
    output/public/omega.xml
    output/public/omega.xml.gz
    output/public/unified_channels.json
    output/public/review.json
    output/public/matches.json
    output/public/manifest.json
    output/public/playlists/

---

## Release Binary

Build release binary:

    CARGO_BUILD_JOBS=1 cargo build -p omega-cli --release

Binary path:

    target/release/omega

Run:

    ./target/release/omega doctor

---

## GitHub Actions

Main workflow:

    .github/workflows/omega-build.yml

Release workflow:

    .github/workflows/omega-release.yml

---

## GitHub Pages Setup

In GitHub:

    Settings
    Pages
    Build and deployment
    Source: GitHub Actions

After deployment, files should be available as:

    https://subhrajyoti0.github.io/tv-epg/omega.m3u
    https://subhrajyoti0.github.io/tv-epg/omega.xml
    https://subhrajyoti0.github.io/tv-epg/omega.xml.gz
    https://subhrajyoti0.github.io/tv-epg/manifest.json

---

## Verification Commands

Check files:

    ls -lh output
    ls -lh output/public
    ls -lh output/playlists

Check database counts:

    sqlite3 omega.db "SELECT provider_kind, COUNT(*) FROM provider_channels GROUP BY provider_kind;"
    sqlite3 omega.db "SELECT COUNT(*) FROM channels;"
    sqlite3 omega.db "SELECT COUNT(*) FROM programmes;"

Check XMLTV validity:

    python3 - <<'PY'
    from pathlib import Path

    text = Path("output/omega.xml").read_text()

    lt = chr(60)
    amp = chr(38)

    print("real XML declaration:", text.startswith(lt + "?xml"))
    print("escaped XML declaration:", text.startswith(amp + "lt;?xml"))
    print("real programme count:", text.count(lt + "programme"))
    print("escaped programme count:", text.count(amp + "lt;programme"))
    PY

Expected XMLTV result:

    real XML declaration: True
    escaped XML declaration: False
    escaped programme count: 0

Check playlist URL escaping:

    grep "^http" output/omega.m3u | grep "&amp;" | head

Ideal result:

    no output

---

## Low Disk VM Recovery

If linking fails or disk becomes full:

    df -h
    du -sh ~/omega
    du -sh ~/omega/target
    cargo clean
    CARGO_BUILD_JOBS=1 cargo check

Then rerun:

    CARGO_BUILD_JOBS=1 cargo run -p omega-cli -- build --epg-limit 20 --start-offset 0 --end-offset 1

---

## Git Ignore Requirements

Do not commit generated/runtime files:

    target/
    omega.db
    *.db
    *.sqlite
    *.sqlite3
    output/
    logs/
    tmp/
    cache/
    .env
    in.m3u

---

## Operational Notes

- Use CARGO_BUILD_JOBS=1 on small VMs.
- Keep output/public as the GitHub Pages deployment root.
- Keep review.json for future manual review and matcher improvement.
- Do not commit omega.db, output/, target/, or in.m3u.
- Run cargo clean if disk becomes full.

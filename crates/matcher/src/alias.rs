use std::collections::HashMap;

use crate::clean::compact_name;

#[derive(Debug, Clone)]
pub struct AliasEngine {
    aliases: HashMap<String, String>,
}

impl AliasEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            aliases: HashMap::new(),
        };

        engine.seed_defaults();

        engine
    }

    pub fn insert(&mut self, alias: impl Into<String>, canonical: impl Into<String>) {
        let alias = compact_name(&alias.into());
        let canonical = canonical.into();

        self.aliases.insert(alias, canonical);
    }

    pub fn resolve(&self, value: &str) -> Option<String> {
        let key = compact_name(value);
        self.aliases.get(&key).cloned()
    }

    pub fn alias_score(&self, a: &str, b: &str) -> f64 {
        let resolved_a = self.resolve(a);
        let resolved_b = self.resolve(b);

        match (resolved_a, resolved_b) {
            (Some(a), Some(b)) if a == b => 1.0,
            (Some(a), _) if compact_name(&a) == compact_name(b) => 0.95,
            (_, Some(b)) if compact_name(a) == compact_name(&b) => 0.95,
            _ => 0.0,
        }
    }

    fn seed_defaults(&mut self) {
        self.insert("Zee TV", "ZeeTV.in");
        self.insert("Zee TV HD", "ZeeTV.in");
        self.insert("ZeeTV", "ZeeTV.in");

        self.insert("Zee News", "ZeeNews.in");
        self.insert("Zee News HD", "ZeeNews.in");

        self.insert("Zee Cinema", "ZeeCinema.in");
        self.insert("Zee Cinema HD", "ZeeCinema.in");

        self.insert("&TV", "AndTV.in");
        self.insert("&TV HD", "AndTV.in");
        self.insert("And TV", "AndTV.in");

        self.insert("&pictures", "AndPictures.in");
        self.insert("&pictures HD", "AndPictures.in");
        self.insert("And Pictures", "AndPictures.in");

        self.insert("Zee Bangla", "ZeeBangla.in");
        self.insert("Zee Marathi", "ZeeMarathi.in");
        self.insert("Zee Tamil", "ZeeTamil.in");
        self.insert("Zee Telugu", "ZeeTelugu.in");
        self.insert("Zee Kannada", "ZeeKannada.in");
        self.insert("Zee Keralam", "ZeeKeralam.in");

        self.insert("Aaj Tak", "AajTak.in");
        self.insert("Aaj Tak HD", "AajTak.in");

        self.insert("WION", "WION.in");
        self.insert("Zing", "Zing.in");
    }
}

impl Default for AliasEngine {
    fn default() -> Self {
        Self::new()
    }
}

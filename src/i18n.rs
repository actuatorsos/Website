//! Internationalization (i18n) Module
//!
//! Handles loading translations and language switching.

use std::collections::HashMap;
use std::fs;
use tower_cookies::Cookies;

#[derive(Debug, Clone)]
pub struct I18n {
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    /// Initialize i18n service by loading JSONs
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        let locales = vec!["ar", "en"];
        for lang in locales {
            let path = std::path::Path::new("locales").join(format!("{}.json", lang));
            tracing::debug!("Loading translation from: {:?}", path);
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let map: HashMap<String, String> = serde_json::from_str(&content)
                        .unwrap_or_else(|e| {
                            panic!("Failed to parse translation file {:?}: {}", path, e)
                        });
                    tracing::info!("Loaded {} keys for {}", map.len(), lang);
                    translations.insert(lang.to_string(), map);
                }
                Err(e) => {
                    panic!(
                        "Failed to read translation file {:?}: {}. Current dir: {:?}",
                        path,
                        e,
                        std::env::current_dir()
                    );
                }
            }
        }

        tracing::info!("I18n service initialized with languages: ar, en");

        Self { translations }
    }

    /// Get translation for a key
    pub fn t(&self, lang: &str, key: &str) -> String {
        self.translations
            .get(lang)
            .and_then(|map| map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("Missing translation key: {} for lang: {}", key, lang);
                key.to_string()
            })
    }

    /// Get all translations for a language (for passing to templates)
    pub fn get_dictionary(&self, lang: &str) -> HashMap<String, String> {
        self.translations.get(lang).cloned().unwrap_or_default()
    }
}

/// Language parameter for query string/cookie
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Arabic,
    English,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Arabic => "ar",
            Language::English => "en",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "en" => Language::English,
            _ => Language::Arabic, // Default
        }
    }

    pub fn dir(&self) -> &'static str {
        match self {
            Language::Arabic => "rtl",
            Language::English => "ltr",
        }
    }

    pub fn resolve(cookies: &Cookies) -> Self {
        cookies
            .get("lang")
            .map(|c| c.value().to_string())
            .map(|s| Self::from_str(&s))
            .unwrap_or(Language::Arabic)
    }
}

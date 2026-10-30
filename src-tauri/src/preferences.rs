use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh-CN")]
    Chinese,
    #[serde(rename = "en")]
    English,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub language: Language,
    pub locale: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: Language::System,
            locale: "en".into(),
        }
    }
}
pub struct PreferenceStore {
    path: PathBuf,
    value: Mutex<Preferences>,
}
pub fn resolve_locale(language: &Language, system: &str) -> &'static str {
    match language {
        Language::Chinese => "zh-CN",
        Language::English => "en",
        Language::System if system.to_ascii_lowercase().starts_with("zh") => "zh-CN",
        Language::System => "en",
    }
}
impl PreferenceStore {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        let path = root.join("preferences.json");
        let mut value: Preferences = if path.exists() {
            serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        } else {
            Preferences::default()
        };
        value.locale = resolve_locale(&value.language, &value.locale).into();
        Ok(Self {
            path,
            value: Mutex::new(value),
        })
    }
    pub fn get(&self) -> Preferences {
        self.value.lock().unwrap().clone()
    }
    pub fn set(&self, language: Option<Language>, system: &str) -> Result<Preferences, String> {
        let mut guard = self.value.lock().unwrap();
        let language = language.unwrap_or_else(|| guard.language.clone());
        let next = Preferences {
            locale: resolve_locale(&language, system).into(),
            language,
        };
        if *guard == next && self.path.exists() {
            return Ok(next);
        }
        fs::create_dir_all(self.path.parent().unwrap()).map_err(|e| e.to_string())?;
        let temp = self.path.with_extension("json.tmp");
        fs::write(
            &temp,
            serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(&temp, &self.path).map_err(|e| e.to_string())?;
        *guard = next.clone();
        Ok(next)
    }
}
// Native menu text and the React views share the same translation resources.
pub fn text(locale: &str, key: &str) -> String {
    let source = if locale == "en" {
        include_str!("../../src/locales/en.json")
    } else {
        include_str!("../../src/locales/zh-CN.json")
    };
    let values: serde_json::Value = serde_json::from_str(source).expect("valid locale catalog");
    values[key].as_str().unwrap_or(key).to_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn language_fallback_persistence_and_failed_save() {
        let root = tempfile::tempdir().unwrap();
        let store = PreferenceStore::new(root.path().into()).unwrap();
        assert_eq!(store.get().language, Language::System);
        assert_eq!(store.set(None, "zh-Hant-TW").unwrap().locale, "zh-CN");
        assert_eq!(
            store.set(Some(Language::English), "zh-CN").unwrap().locale,
            "en"
        );
        let reopened = PreferenceStore::new(root.path().into()).unwrap();
        assert_eq!(reopened.get(), store.get());
        assert_eq!(store.set(None, "zh-CN").unwrap().locale, "en");
        assert_eq!(
            store.set(Some(Language::System), "fr-FR").unwrap().locale,
            "en"
        );
        assert_eq!(resolve_locale(&Language::Chinese, "en-US"), "zh-CN");
        fs::create_dir(root.path().join("preferences.json.tmp")).unwrap();
        assert!(store.set(Some(Language::Chinese), "en-US").is_err());
        assert_eq!(store.get().language, Language::System);
        assert_eq!(
            PreferenceStore::new(root.path().into()).unwrap().get(),
            store.get()
        );
        assert!(serde_json::from_str::<Language>("\"unknown\"").is_err());
        assert_eq!(text("en", "打开 Flint"), "Open Flint");
        assert_eq!(text("zh-CN", "打开 Flint"), "打开 Flint");
    }
}

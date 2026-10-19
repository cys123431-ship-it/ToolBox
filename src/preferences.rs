use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub executables: BTreeMap<String, String>,
    pub recent: Vec<PathBuf>,
}

pub fn preferences_path() -> PathBuf {
    let root = env::var_os("APPDATA")
        .or_else(|| env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(env::temp_dir);
    root.join("ToolBox").join("preferences.json")
}

impl Preferences {
    pub fn load() -> Self {
        let Ok(content) = fs::read_to_string(preferences_path()) else {
            return Self::default();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = preferences_path();
        let parent = path.parent().ok_or("설정 저장 위치를 찾지 못했습니다.")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let content = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn binary(&self, engine_key: &str) -> Option<&str> {
        self.executables.get(engine_key).map(String::as_str)
    }

    pub fn record_recent(&mut self, path: &Path) {
        let stable = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.recent.retain(|p| p != &stable);
        self.recent.insert(0, stable);
        self.recent.truncate(15);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_list_is_deduplicated_and_bounded() {
        let mut p = Preferences::default();
        for i in 0..22 {
            p.record_recent(Path::new(&format!("sample_{i}.docx")));
        }
        assert_eq!(p.recent.len(), 15);
        let first = p.recent[0].clone();
        p.record_recent(&first);
        assert_eq!(p.recent.len(), 15);
        assert_eq!(p.recent[0], first);
    }

    #[test]
    fn settings_serialize() {
        let mut prefs = Preferences::default();
        prefs
            .executables
            .insert("wordcraft".into(), "C:\\Tools\\wordcraft.exe".into());
        let serialized = serde_json::to_string(&prefs).unwrap();
        let loaded: Preferences = serde_json::from_str(&serialized).unwrap();
        assert_eq!(loaded.binary("wordcraft"), Some("C:\\Tools\\wordcraft.exe"));
    }
}

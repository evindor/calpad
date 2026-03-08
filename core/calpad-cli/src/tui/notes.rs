use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct NoteStore {
    pub notes: Vec<Note>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub sidebar_visible: bool,
    pub active_note_id: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "monokai".to_string(),
            sidebar_visible: true,
            active_note_id: String::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = data_dir().join("config.json");
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = data_dir().join("config.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}

pub fn data_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("calpad");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}", now.as_nanos())
}

impl NoteStore {
    pub fn load() -> Self {
        let path = data_dir().join("notes.json");
        let notes = fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { notes }
    }

    pub fn save(&self) {
        let path = data_dir().join("notes.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.notes) {
            let _ = fs::write(path, json);
        }
    }

    pub fn create(&mut self, content: String) -> String {
        let id = generate_id();
        let title = extract_title(&content);
        let now = now_ts();
        self.notes.insert(
            0,
            Note {
                id: id.clone(),
                title,
                content,
                created_at: now,
                updated_at: now,
            },
        );
        self.save();
        id
    }

    pub fn update(&mut self, id: &str, content: &str) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
            note.content = content.to_string();
            note.title = extract_title(content);
            note.updated_at = now_ts();
        }
    }

    pub fn delete(&mut self, id: &str) {
        self.notes.retain(|n| n.id != id);
        self.save();
    }

    pub fn get(&self, id: &str) -> Option<&Note> {
        self.notes.iter().find(|n| n.id == id)
    }

    pub fn selected_index(&self, id: &str) -> Option<usize> {
        self.notes.iter().position(|n| n.id == id)
    }
}

pub fn extract_title(content: &str) -> String {
    content
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            let l = l.trim();
            let l = l.trim_start_matches('#').trim_start_matches("//").trim();
            // Strip "Label: " pattern
            if let Some(pos) = l.find(": ") {
                if l[..pos]
                    .chars()
                    .all(|c| c.is_ascii_alphabetic() || c == ' ')
                {
                    return l[pos + 2..].trim().chars().take(60).collect();
                }
            }
            l.chars().take(60).collect()
        })
        .unwrap_or_else(|| "Untitled".to_string())
}

pub fn load_cached_rates() -> Option<std::collections::HashMap<String, f64>> {
    let path = data_dir().join("rates.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn cache_rates(rates: &std::collections::HashMap<String, f64>) {
    let path = data_dir().join("rates.json");
    if let Ok(json) = serde_json::to_string(rates) {
        let _ = fs::write(path, json);
    }
}

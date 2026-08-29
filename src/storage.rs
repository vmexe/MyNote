use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::Utc;
use crate::models::Note;

pub struct Storage;

impl Storage {
    pub fn get_data_dir() -> PathBuf {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("mynote")
        } else {
            PathBuf::from(".mynote")
        }
    }

    pub fn get_notes_file() -> PathBuf {
        Self::get_data_dir().join("notes.json")
    }

    pub fn ensure_data_dir() -> Result<PathBuf, String> {
        let dir = Self::get_data_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir {:?}: {}", dir, e))?;
        }
        Ok(dir)
    }

    pub fn get_settings_file() -> PathBuf {
        Self::get_data_dir().join("settings.json")
    }

    pub fn load_settings() -> Settings {
        let path = Self::get_settings_file();
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str::<Settings>(&content).unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    }

    pub fn save_settings(settings: &Settings) -> Result<(), String> {
        Self::ensure_data_dir()?;
        let path = Self::get_settings_file();
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write settings file: {}", e))
    }

    pub fn load_notes() -> Vec<Note> {
        let path = Self::get_notes_file();
        if !path.exists() {
            let default_notes = Self::default_starter_notes();
            let _ = Self::save_notes(&default_notes);
            return default_notes;
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<Note>>(&content) {
                    Ok(notes) => notes,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse notes JSON: {}. Creating backup.", e);
                        let backup_path = Self::get_data_dir().join(format!("notes_backup_{}.json", Utc::now().timestamp()));
                        let _ = fs::copy(&path, &backup_path);
                        Self::default_starter_notes()
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read notes file: {}", e);
                Self::default_starter_notes()
            }
        }
    }

    pub fn save_notes(notes: &[Note]) -> Result<(), String> {
        Self::ensure_data_dir()?;
        let path = Self::get_notes_file();
        let tmp_path = path.with_extension("tmp");

        let json = serde_json::to_string_pretty(notes)
            .map_err(|e| format!("Failed to serialize notes: {}", e))?;

        {
            let mut file = File::create(&tmp_path)
                .map_err(|e| format!("Failed to create temp notes file: {}", e))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write notes data: {}", e))?;
            file.flush()
                .map_err(|e| format!("Failed to flush notes file: {}", e))?;
        }

        fs::rename(&tmp_path, &path)
            .map_err(|e| format!("Failed to atomically replace notes file: {}", e))?;

        Ok(())
    }

    pub fn export_note_to_file(note: &Note, path: &Path) -> Result<(), String> {
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create export file: {}", e))?;

        let mut output = String::new();
        if !note.title.trim().is_empty() {
            output.push_str(&format!("# {}\n\n", note.title.trim()));
        }
        if !note.tags.is_empty() {
            let tags_str = note.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            output.push_str(&format!("Tags: {}\n\n", tags_str));
        }
        output.push_str(&note.content);

        file.write_all(output.as_bytes())
            .map_err(|e| format!("Failed to write exported note: {}", e))?;
        Ok(())
    }

    pub fn export_all_notes_to_dir(notes: &[Note], target_dir: &Path) -> Result<usize, String> {
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)
                .map_err(|e| format!("Failed to create target export directory: {}", e))?;
        }

        let mut count = 0;
        for note in notes {
            if note.is_trashed {
                continue;
            }
            let safe_title: String = note.display_title()
                .chars()
                .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
                .collect();
            let filename = format!("{}_{}.md", safe_title.trim().replace(' ', "_"), &note.id[..8]);
            let dest = target_dir.join(filename);
            if Self::export_note_to_file(note, &dest).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn import_note_from_file(path: &Path) -> Result<Note, String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file for import: {}", e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read file content: {}", e))?;

        let mut title = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Note")
            .to_string();

        let mut note_content = content;

        if let Some(first_line) = note_content.lines().next() {
            if first_line.starts_with("# ") {
                title = first_line.trim_start_matches("# ").trim().to_string();
                note_content = note_content.lines().skip(1).collect::<Vec<_>>().join("\n").trim_start().to_string();
            }
        }

        let mut note = Note::with_title_and_content(title, note_content);
        note.tags.push("imported".to_string());
        Ok(note)
    }

    pub fn default_starter_notes() -> Vec<Note> {
        let now = Utc::now();
        vec![
            Note {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Welcome to MyNote! 🚀".to_string(),
                content: r#"# Welcome to MyNote

**MyNote** is a super-fast, clean, and modern note-taking application designed natively for Linux.

### Key Features:
- ⚡ **Blazing Fast & Lightweight**: Written in Rust + GTK4 & Libadwaita.
- 💾 **Automatic Instant Saving**: Never worry about losing your thoughts.
- 🏷️ **Tags & Organization**: Tag notes with `#work`, `#ideas`, `#todo` to stay organized.
- ⭐ **Favorites / Pinning**: Pin your most important notes to the top.
- 🔍 **Real-time Search**: Instant search through titles, content, and tags.
- 📝 **Rich Markdown Support**: Live preview, formatting toolbar, and keyboard shortcuts.
- 📤 **Export & Import**: Export to Markdown (`.md`) or Plain Text (`.txt`).
- 🗑️ **Trash Bin**: Safely restore deleted notes or empty trash whenever needed.

### Markdown Tips:
- Use `**bold**` or `*italic*`
- Use `# Heading 1`, `## Heading 2`
- Use `- [ ] Task item` for checkboxes
- Use `- Bullet item` for lists
- Use ```` ```rust ```` for code blocks
- Use `> Quotes` for highlights

Enjoy note-taking with MyNote! 🎉"#.to_string(),
                tags: vec!["welcome".to_string(), "guide".to_string()],
                is_pinned: true,
                is_trashed: false,
                created_at: now,
                updated_at: now,
            },
            Note {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Keyboard Shortcuts ⌨️".to_string(),
                content: r#"# Handy Keyboard Shortcuts

Boost your productivity with these keyboard shortcuts:

| Action | Shortcut |
|---|---|
| **New Note** | `Ctrl + N` |
| **Search Notes** | `Ctrl + F` |
| **Toggle Favorite / Pin** | `Ctrl + D` |
| **Export Note** | `Ctrl + E` |
| **Import Note** | `Ctrl + I` |
| **Toggle Markdown Preview** | `Ctrl + P` |
| **Delete / Trash Note** | `Ctrl + Delete` |
| **Keyboard Shortcuts Help** | `Ctrl + ?` |
| **Quit MyNote** | `Ctrl + Q` |

*Tip: Press `Ctrl + F` anytime to quickly filter your notes list.*"#.to_string(),
                tags: vec!["shortcuts".to_string(), "tips".to_string()],
                is_pinned: false,
                is_trashed: false,
                created_at: now,
                updated_at: now,
            },
            Note {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Linux Setup & Project Ideas 🐧".to_string(),
                content: r#"# Linux Setup & Projects

Here is an example note for tracking tasks and ideas:

### Project Goals:
- [x] Configure Linux desktop environment
- [x] Install Rust development toolchain
- [ ] Build a lightning-fast native note app
- [ ] Star MyNote on GitHub ⭐

### Useful Linux Commands:
```bash
# Update system packages
sudo pacman -Syu    # Arch Linux
sudo apt update     # Ubuntu / Debian
sudo dnf upgrade    # Fedora

# Check disk space
df -h
```

> "Simplicity is prerequisite for reliability." — Edsger W. Dijkstra"#.to_string(),
                tags: vec!["linux".to_string(), "ideas".to_string(), "todo".to_string()],
                is_pinned: false,
                is_trashed: false,
                created_at: now,
                updated_at: now,
            },
        ]
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Settings {
    pub last_note_id: Option<String>,
    pub view_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_and_import() {
        let temp_dir = std::env::temp_dir().join(format!("mynote_test_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_note.md");

        let note = Note::with_title_and_content("Test Note", "This is content with **markdown**.");
        let res = Storage::export_note_to_file(&note, &file_path);
        assert!(res.is_ok());

        let imported = Storage::import_note_from_file(&file_path);
        assert!(imported.is_ok());
        let imp_note = imported.unwrap();
        assert_eq!(imp_note.title, "Test Note");
        assert!(imp_note.content.contains("This is content with **markdown**."));
        assert!(imp_note.tags.contains(&"imported".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

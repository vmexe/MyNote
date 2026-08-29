use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub is_trashed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            content: String::new(),
            tags: Vec::new(),
            is_pinned: false,
            is_trashed: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_title_and_content(title: impl Into<String>, content: impl Into<String>) -> Self {
        let mut note = Self::new();
        note.title = title.into();
        note.content = content.into();
        note
    }

    pub fn display_title(&self) -> String {
        if !self.title.trim().is_empty() {
            self.title.trim().to_string()
        } else {
            // Try to extract first line from content
            if let Some(first_line) = self.content.lines().find(|l| !l.trim().is_empty()) {
                let cleaned = first_line.trim().trim_start_matches('#').trim();
                if !cleaned.is_empty() {
                    let mut s = cleaned.to_string();
                    if s.chars().count() > 40 {
                        s = s.chars().take(40).collect::<String>() + "…";
                    }
                    return s;
                }
            }
            "Untitled Note".to_string()
        }
    }

    pub fn excerpt(&self) -> String {
        let mut lines = self.content.lines().filter(|l| !l.trim().is_empty());
        
        let raw_snippet = if !self.title.trim().is_empty() {
            lines.next().unwrap_or("No additional text")
        } else {
            let _ = lines.next();
            lines.next().unwrap_or("No additional text")
        };

        let cleaned = raw_snippet.trim()
            .trim_start_matches('#')
            .trim_start_matches('-')
            .trim_start_matches('*')
            .trim_start_matches('>')
            .trim();

        if cleaned.is_empty() {
            "No additional text".to_string()
        } else if cleaned.chars().count() > 60 {
            cleaned.chars().take(60).collect::<String>() + "…"
        } else {
            cleaned.to_string()
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }

        if self.title.to_lowercase().contains(&q) {
            return true;
        }

        if self.content.to_lowercase().contains(&q) {
            return true;
        }

        for tag in &self.tags {
            if tag.to_lowercase().contains(&q) {
                return true;
            }
        }

        false
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }

    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            0
        } else {
            self.content.lines().count()
        }
    }

    pub fn reading_time_mins(&self) -> usize {
        let words = self.word_count();
        if words == 0 {
            0
        } else {
            (words / 200).max(1)
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    All,
    Favorites,
    Trash,
    Tag(String),
}

impl FilterMode {
    pub fn matches(&self, note: &Note) -> bool {
        match self {
            FilterMode::All => !note.is_trashed,
            FilterMode::Favorites => !note.is_trashed && note.is_pinned,
            FilterMode::Trash => note.is_trashed,
            FilterMode::Tag(tag) => !note.is_trashed && note.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::with_title_and_content("Meeting Notes", "Discuss Q3 goals with team.");
        assert_eq!(note.display_title(), "Meeting Notes");
        assert_eq!(note.word_count(), 5);
        assert_eq!(note.char_count(), 27);
        assert_eq!(note.line_count(), 1);
        assert!(!note.is_pinned);
        assert!(!note.is_trashed);
    }

    #[test]
    fn test_untitled_fallback() {
        let note = Note::with_title_and_content("", "# First Line Heading\nSecond line body");
        assert_eq!(note.display_title(), "First Line Heading");
        assert_eq!(note.excerpt(), "Second line body");
    }

    #[test]
    fn test_empty_note_fallback() {
        let note = Note::new();
        assert_eq!(note.display_title(), "Untitled Note");
        assert_eq!(note.excerpt(), "No additional text");
    }

    #[test]
    fn test_filter_matches() {
        let mut note = Note::with_title_and_content("Rust Guide", "Rust is memory-safe.");
        note.tags.push("programming".to_string());
        note.is_pinned = true;

        assert!(FilterMode::All.matches(&note));
        assert!(FilterMode::Favorites.matches(&note));
        assert!(!FilterMode::Trash.matches(&note));
        assert!(FilterMode::Tag("programming".to_string()).matches(&note));
        assert!(!FilterMode::Tag("music".to_string()).matches(&note));

        note.is_trashed = true;
        assert!(!FilterMode::All.matches(&note));
        assert!(!FilterMode::Favorites.matches(&note));
        assert!(FilterMode::Trash.matches(&note));
    }

    #[test]
    fn test_query_matching() {
        let mut note = Note::with_title_and_content("Shopping List", "Buy milk, eggs, bread");
        note.tags.push("groceries".to_string());

        assert!(note.matches_query("shopping"));
        assert!(note.matches_query("MILK"));
        assert!(note.matches_query("groceries"));
        assert!(!note.matches_query("laptop"));
    }
}

use similar::{ChangeTag, TextDiff};
use crate::models::DiffSegment;

/// Generate a local text diff between two texts using the `similar` crate.
pub fn generate_diff(text_a: &str, text_b: &str) -> Vec<DiffSegment> {
    let diff = TextDiff::from_words(text_a, text_b);
    let mut segments = Vec::new();

    for change in diff.iter_all_changes() {
        let value = change.value().to_string();
        match change.tag() {
            ChangeTag::Equal => {
                segments.push(DiffSegment {
                    tag: "equal".to_string(),
                    old_text: value.clone(),
                    new_text: value,
                });
            }
            ChangeTag::Delete => {
                segments.push(DiffSegment {
                    tag: "delete".to_string(),
                    old_text: value,
                    new_text: String::new(),
                });
            }
            ChangeTag::Insert => {
                segments.push(DiffSegment {
                    tag: "insert".to_string(),
                    old_text: String::new(),
                    new_text: value,
                });
            }
        }
    }

    segments
}

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub change_type: String, // "add", "delete", "equal"
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
    pub content: String,
}

pub fn generate_diff(raw_text: &str, sanitized_text: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(raw_text, sanitized_text);
    let mut diff_lines = Vec::new();

    for change in diff.iter_all_changes() {
        let (change_type, old_no, new_no) = match change.tag() {
            ChangeTag::Delete => ("delete", change.old_index(), None),
            ChangeTag::Insert => ("add", None, change.new_index()),
            ChangeTag::Equal => ("equal", change.old_index(), change.new_index()),
        };

        diff_lines.push(DiffLine {
            change_type: change_type.to_string(),
            old_line_no: old_no.map(|i| i + 1),
            new_line_no: new_no.map(|i| i + 1),
            content: change.value().trim_end_matches('\n').to_string(),
        });
    }

    diff_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_generation() {
        let raw = "line 1\nline 2 with secret sk-123\nline 3";
        let sanitized = "line 1\nline 2 with secret [REDACTED]\nline 3";

        let lines = generate_diff(raw, sanitized);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.change_type == "delete"));
        assert!(lines.iter().any(|l| l.change_type == "add"));
    }
}

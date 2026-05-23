use crate::models::SuggestionItem;

/// Apply approved patches to document content.
/// Sorts patches by span_start descending (end-to-start) to preserve byte offsets.
/// Returns the new document content.
pub fn apply_patches(content: &str, approved_items: &[SuggestionItem]) -> Result<String, String> {
    if approved_items.is_empty() {
        return Ok(content.to_string());
    }

    // Check for conflicts first
    let conflicts = detect_conflicts(approved_items);
    if !conflicts.is_empty() {
        return Err(format!(
            "Cannot apply: {} conflicting suggestion pairs detected. Resolve conflicts first.",
            conflicts.len()
        ));
    }

    let mut result = content.to_string();

    // Sort by span_start descending so we apply from end to beginning
    let mut sorted: Vec<&SuggestionItem> = approved_items.iter().collect();
    sorted.sort_by(|a, b| b.span_start.cmp(&a.span_start));

    for item in &sorted {
        // Validate span is within bounds
        if item.span_end > result.len() {
            return Err(format!(
                "Patch span [{}, {}) exceeds document length {}",
                item.span_start, item.span_end, result.len()
            ));
        }

        // Validate the original text still matches
        let actual = &result[item.span_start..item.span_end];
        if actual != item.original_text {
            return Err(format!(
                "Patch mismatch at [{}, {}): expected '{}', found '{}'",
                item.span_start,
                item.span_end,
                &item.original_text[..item.original_text.len().min(50)],
                &actual[..actual.len().min(50)]
            ));
        }

        result.replace_range(item.span_start..item.span_end, &item.replacement_text);
    }

    Ok(result)
}

/// Compute a preview of the document with approved suggestions applied.
/// Same logic as apply_patches but explicitly named for clarity.
pub fn compute_preview(content: &str, approved_items: &[SuggestionItem]) -> Result<String, String> {
    apply_patches(content, approved_items)
}

/// Compute a preview of the document with approved suggestions applied and highlighted using HTML span tags.
/// Sorts patches by span_start descending (end-to-start) to preserve byte offsets.
/// Returns the new document content with HTML highlights.
pub fn apply_patches_highlighted(content: &str, approved_items: &[SuggestionItem]) -> Result<String, String> {
    if approved_items.is_empty() {
        return Ok(content.to_string());
    }

    // Check for conflicts first
    let conflicts = detect_conflicts(approved_items);
    if !conflicts.is_empty() {
        return Err(format!(
            "Cannot apply: {} conflicting suggestion pairs detected. Resolve conflicts first.",
            conflicts.len()
        ));
    }

    let mut result = content.to_string();

    // Sort by span_start descending so we apply from end to beginning
    let mut sorted: Vec<&SuggestionItem> = approved_items.iter().collect();
    sorted.sort_by(|a, b| b.span_start.cmp(&a.span_start));

    for item in &sorted {
        // Validate span is within bounds
        if item.span_end > result.len() {
            return Err(format!(
                "Patch span [{}, {}) exceeds document length {}",
                item.span_start, item.span_end, result.len()
            ));
        }

        // Validate the original text still matches
        let actual = &result[item.span_start..item.span_end];
        if actual != item.original_text {
            return Err(format!(
                "Patch mismatch at [{}, {}): expected '{}', found '{}'",
                item.span_start,
                item.span_end,
                &item.original_text[..item.original_text.len().min(50)],
                &actual[..actual.len().min(50)]
            ));
        }

        let replacement = format!(
            r#"<span class="preview-highlight-inserted" data-suggestion-id="{}">{}</span>"#,
            item.id, item.replacement_text
        );
        result.replace_range(item.span_start..item.span_end, &replacement);
    }

    Ok(result)
}

/// Highlight original document text showing original phrases and their active review suggestion mappings.
/// Sorts spans descending (end-to-start) to safely insert highlighting tags without shifting indices.
pub fn highlight_original(content: &str, all_items: &[SuggestionItem]) -> Result<String, String> {
    if all_items.is_empty() {
        return Ok(content.to_string());
    }

    let mut result = content.to_string();

    // Sort by span_start descending so we apply from end to beginning
    let mut sorted: Vec<&SuggestionItem> = all_items.iter().collect();
    sorted.sort_by(|a, b| b.span_start.cmp(&a.span_start));

    for item in &sorted {
        // Validate span is within bounds
        if item.span_end > result.len() {
            return Err(format!(
                "Original highlight span [{}, {}) exceeds document length {}",
                item.span_start, item.span_end, result.len()
            ));
        }

        // Validate the original text still matches
        let actual = &result[item.span_start..item.span_end];
        if actual != item.original_text {
            return Err(format!(
                "Original highlight mismatch at [{}, {}): expected '{}', found '{}'",
                item.span_start,
                item.span_end,
                &item.original_text[..item.original_text.len().min(50)],
                &actual[..actual.len().min(50)]
            ));
        }

        let wrapper = format!(
            r#"<span class="preview-highlight-original preview-original-{}" data-suggestion-id="{}">{}</span>"#,
            item.approval_state, item.id, item.original_text
        );
        result.replace_range(item.span_start..item.span_end, &wrapper);
    }

    Ok(result)
}

/// Compute a preview where approved items show their replacement text and pending/rejected items show original text, all wrapped in their correct highlight spans.
pub fn compute_editorial_preview(content: &str, all_items: &[SuggestionItem]) -> Result<String, String> {
    if all_items.is_empty() {
        return Ok(content.to_string());
    }

    let mut result = content.to_string();

    // Filter out conflicting (overlapping) items to ensure safety and prevent index shifting errors
    let mut safe_items: Vec<SuggestionItem> = Vec::new();
    for item in all_items {
        let mut overlap = false;
        for existing in &safe_items {
            if item.span_start < existing.span_end && existing.span_start < item.span_end {
                overlap = true;
                break;
            }
        }
        if !overlap {
            safe_items.push(item.clone());
        }
    }

    // Sort by span_start descending so we process from end to beginning
    safe_items.sort_by(|a, b| b.span_start.cmp(&a.span_start));

    for item in &safe_items {
        // Validate span is within bounds
        if item.span_end > result.len() {
            return Err(format!(
                "Span [{}, {}) exceeds content length {}",
                item.span_start, item.span_end, result.len()
            ));
        }

        // Validate the original text still matches
        let actual = &result[item.span_start..item.span_end];
        if actual != item.original_text {
            return Err(format!(
                "Content mismatch at [{}, {}): expected '{}', found '{}'",
                item.span_start,
                item.span_end,
                &item.original_text[..item.original_text.len().min(50)],
                &actual[..actual.len().min(50)]
            ));
        }

        let replacement = match item.approval_state.as_str() {
            "approved" => {
                // Approved: show replacement text wrapped in inserted span
                format!(
                    r#"<span class="preview-highlight-inserted state-approved" data-suggestion-id="{}">{}</span>"#,
                    item.id, item.replacement_text
                )
            }
            "rejected" => {
                // Rejected: show original text wrapped in rejected span
                format!(
                    r#"<span class="preview-highlight-original preview-original-rejected" data-suggestion-id="{}">{}</span>"#,
                    item.id, item.original_text
                )
            }
            _ => {
                // Pending: show original text wrapped in pending highlight span
                format!(
                    r#"<span class="preview-highlight-original preview-original-pending" data-suggestion-id="{}">{}</span>"#,
                    item.id, item.original_text
                )
            }
        };

        result.replace_range(item.span_start..item.span_end, &replacement);
    }

    Ok(result)
}


/// Detect conflicting (overlapping) patches. Returns pairs of indices.
pub fn detect_conflicts(items: &[SuggestionItem]) -> Vec<(usize, usize)> {
    let mut conflicts = Vec::new();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i].span_start < items[j].span_end && items[j].span_start < items[i].span_end {
                conflicts.push((i, j));
            }
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(original: &str, replacement: &str, start: usize, end: usize) -> SuggestionItem {
        SuggestionItem {
            id: "test".to_string(),
            group_id: "g".to_string(),
            original_text: original.to_string(),
            replacement_text: replacement.to_string(),
            span_start: start,
            span_end: end,
            explanation: String::new(),
            confidence: 1.0,
            approval_state: "approved".to_string(),
            conflict_state: None,
        }
    }

    #[test]
    fn test_single_patch() {
        let content = "The quick brown fox jumps over the lazy dog.";
        let items = vec![make_item("brown", "red", 10, 15)];
        let result = apply_patches(content, &items).unwrap();
        assert_eq!(result, "The quick red fox jumps over the lazy dog.");
    }

    #[test]
    fn test_multiple_patches() {
        let content = "The quick brown fox jumps over the lazy dog.";
        let items = vec![
            make_item("brown", "red", 10, 15),
            make_item("lazy", "energetic", 35, 39),
        ];
        let result = apply_patches(content, &items).unwrap();
        assert_eq!(result, "The quick red fox jumps over the energetic dog.");
    }

    #[test]
    fn test_conflict_detection() {
        let items = vec![
            make_item("brown fox", "red fox", 10, 19),
            make_item("fox jumps", "cat leaps", 16, 25),
        ];
        let conflicts = detect_conflicts(&items);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_empty_patches() {
        let content = "Hello world";
        let result = apply_patches(content, &[]).unwrap();
        assert_eq!(result, "Hello world");
    }
}

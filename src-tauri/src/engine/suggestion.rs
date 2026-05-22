use crate::models::*;
use uuid::Uuid;

/// Parse the raw AI JSON response into normalized suggestion groups and items.
/// Handles markdown fence stripping and validates spans against the document content.
pub fn normalize_ai_response(
    raw_json: &str,
    page_content: &str,
    page_id: &str,
    session_id: &str,
) -> Result<Vec<SuggestionGroupWithItems>, String> {
    // Strip markdown code fences if present
    let json_str = strip_markdown_fences(raw_json);

    let parsed: AiSuggestionResponse = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse AI response JSON: {}. Raw (first 500 chars): {}", e, &raw_json[..raw_json.len().min(500)]))?;

    let mut result = Vec::new();

    for ai_group in parsed.suggestion_groups {
        let group_id = Uuid::new_v4().to_string();
        let mut items = Vec::new();

        for ai_item in &ai_group.items {
            // Find the span in the original content
            match find_text_span(page_content, &ai_item.original_text) {
                Some((start, end)) => {
                    let item_id = Uuid::new_v4().to_string();
                    items.push(SuggestionItem {
                        id: item_id,
                        group_id: group_id.clone(),
                        original_text: ai_item.original_text.clone(),
                        replacement_text: ai_item.replacement_text.clone(),
                        span_start: start,
                        span_end: end,
                        explanation: ai_item.explanation.clone(),
                        confidence: ai_item.confidence.clamp(0.0, 1.0),
                        approval_state: "pending".to_string(),
                        conflict_state: None,
                    });
                }
                None => {
                    // Skip items whose original_text can't be found in the document
                    log::warn!(
                        "Suggestion original_text not found in document: '{}'",
                        &ai_item.original_text[..ai_item.original_text.len().min(80)]
                    );
                }
            }
        }

        if !items.is_empty() {
            // Detect conflicts within this group
            detect_item_conflicts(&mut items);

            let group = SuggestionGroup {
                id: group_id,
                page_id: page_id.to_string(),
                review_session_id: session_id.to_string(),
                category: normalize_category(&ai_group.category),
                label: ai_group.label.clone(),
                approval_state: "pending".to_string(),
            };

            result.push(SuggestionGroupWithItems { group, items });
        }
    }

    // Also detect cross-group conflicts
    detect_cross_group_conflicts(&mut result);

    Ok(result)
}

/// Find the byte offset span of a substring in the content.
/// Returns (start, end) byte offsets.
fn find_text_span(content: &str, search: &str) -> Option<(usize, usize)> {
    if search.is_empty() {
        return None;
    }
    content.find(search).map(|start| (start, start + search.len()))
}

/// Strip markdown code fences from AI response
fn strip_markdown_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    if trimmed.starts_with("```json") {
        let stripped = trimmed.strip_prefix("```json").unwrap_or(trimmed);
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else if trimmed.starts_with("```") {
        let stripped = trimmed.strip_prefix("```").unwrap_or(trimmed);
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else {
        trimmed
    }
}

/// Normalize category string to known values
fn normalize_category(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "spelling" | "spell" => "spelling".to_string(),
        "grammar" | "grammatical" => "grammar".to_string(),
        "vocabulary" | "vocab" | "word_choice" | "word choice" => "vocabulary".to_string(),
        "clarity" | "clear" => "clarity".to_string(),
        "fluency" | "flow" | "readability" => "fluency".to_string(),
        "rewrite" | "rephrase" | "restructure" => "rewrite".to_string(),
        "recommendation" | "suggestion" | "style" | "tone" => "recommendation".to_string(),
        other => other.to_string(),
    }
}

/// Detect overlapping spans within a list of items and mark conflicts
fn detect_item_conflicts(items: &mut [SuggestionItem]) {
    let len = items.len();
    for i in 0..len {
        for j in (i + 1)..len {
            let a_start = items[i].span_start;
            let a_end = items[i].span_end;
            let b_start = items[j].span_start;
            let b_end = items[j].span_end;

            if a_start < b_end && b_start < a_end {
                items[i].conflict_state = Some("overlapping".to_string());
                items[j].conflict_state = Some("overlapping".to_string());
            }
        }
    }
}

/// Detect conflicts across suggestion groups
fn detect_cross_group_conflicts(groups: &mut [SuggestionGroupWithItems]) {
    let all_items: Vec<(usize, usize, usize, usize)> = groups
        .iter()
        .enumerate()
        .flat_map(|(gi, g)| {
            g.items.iter().enumerate().map(move |(ii, item)| {
                (gi, ii, item.span_start, item.span_end)
            })
        })
        .collect();

    for i in 0..all_items.len() {
        for j in (i + 1)..all_items.len() {
            let (gi, ii, a_start, a_end) = all_items[i];
            let (gj, ij, b_start, b_end) = all_items[j];

            if gi != gj && a_start < b_end && b_start < a_end {
                groups[gi].items[ii].conflict_state = Some("overlapping".to_string());
                groups[gj].items[ij].conflict_state = Some("overlapping".to_string());
            }
        }
    }
}

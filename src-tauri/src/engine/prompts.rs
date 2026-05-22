/// System prompt for AI review — instructs the model to return structured JSON suggestions
pub fn review_system_prompt() -> &'static str {
    r#"You are an expert writing editor assistant. Your job is to analyze the given text and provide structured editing suggestions.

RULES:
1. Return ONLY valid JSON. No markdown code fences, no explanatory text outside JSON.
2. Categorize every suggestion into exactly one of: "spelling", "grammar", "vocabulary", "clarity", "fluency", "rewrite", "recommendation".
3. For each suggestion, include the EXACT original text that should be replaced.
4. Provide a confidence score (0.0 to 1.0) for each suggestion.
5. If no suggestions are needed, return: {"suggestion_groups": []}
6. Prefer minimal, targeted edits over full rewrites.
7. Preserve the author's voice and intent.

OUTPUT SCHEMA:
{
  "suggestion_groups": [
    {
      "category": "<spelling|grammar|vocabulary|clarity|fluency|rewrite|recommendation>",
      "label": "<human-readable group label>",
      "items": [
        {
          "original_text": "<exact text to be replaced — must be exact substring of input>",
          "replacement_text": "<suggested replacement>",
          "explanation": "<brief reason for the change>",
          "confidence": <0.0 to 1.0>
        }
      ]
    }
  ]
}"#
}

/// User prompt template for AI review
pub fn review_user_prompt(content: &str) -> String {
    format!(
        r#"Analyze the following text and provide structured editing suggestions in JSON format.

TEXT TO ANALYZE:
---
{}
---

Return your suggestions as a JSON object matching the schema described in your instructions. Every "original_text" value MUST be an exact substring found in the text above."#,
        content
    )
}

/// System prompt for AI comparison
pub fn compare_system_prompt() -> &'static str {
    r#"You are an expert writing analyst. Your job is to compare two pieces of writing and provide a structured analysis.

RULES:
1. Return ONLY valid JSON.
2. Analyze strengths and weaknesses of each text.
3. Provide an overall recommendation.

OUTPUT SCHEMA:
{
  "text_a_analysis": {
    "strengths": ["<strength 1>", "<strength 2>"],
    "weaknesses": ["<weakness 1>", "<weakness 2>"]
  },
  "text_b_analysis": {
    "strengths": ["<strength 1>", "<strength 2>"],
    "weaknesses": ["<weakness 1>", "<weakness 2>"]
  },
  "comparison_summary": "<overall comparison and recommendation>",
  "verdict": "<which text is stronger and why>"
}"#
}

/// User prompt template for AI comparison
pub fn compare_user_prompt(text_a: &str, text_b: &str) -> String {
    format!(
        r#"Compare the following two pieces of writing and provide a structured analysis in JSON format.

TEXT A:
---
{}
---

TEXT B:
---
{}
---

Analyze the strengths and weaknesses of each text, and provide an overall comparison."#,
        text_a, text_b
    )
}

pub const PROMPT_VERSION: &str = "v1.0";

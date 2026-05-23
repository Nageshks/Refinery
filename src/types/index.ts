// ─── Approval State ─────────────────────────────────────────────────────
export type ApprovalState = 'pending' | 'approved' | 'rejected' | 'partial';
export type ViewMode = 'edit' | 'review' | 'compare' | 'settings';

// ─── Page ───────────────────────────────────────────────────────────────
export interface Page {
  id: string;
  title: string;
  content: string;
  format_type: string;
  created_at: string;
  updated_at: string;
  current_version_id: string | null;
  last_review_session_id: string | null;
  local_status: string;
}

// ─── Suggestion ─────────────────────────────────────────────────────────
export interface SuggestionItem {
  id: string;
  group_id: string;
  original_text: string;
  replacement_text: string;
  span_start: number;
  span_end: number;
  explanation: string;
  confidence: number;
  approval_state: ApprovalState;
  conflict_state: string | null;
}

export interface SuggestionGroup {
  id: string;
  page_id: string;
  review_session_id: string;
  category: string;
  label: string;
  approval_state: ApprovalState;
}

export interface SuggestionGroupWithItems {
  group: SuggestionGroup;
  items: SuggestionItem[];
}

// ─── Review Session ─────────────────────────────────────────────────────
export interface ReviewSession {
  id: string;
  page_id: string;
  provider_id: string;
  model_id: string;
  prompt_version: string;
  raw_response: string;
  status: string;
  created_at: string;
}

export interface ReviewResult {
  session: ReviewSession;
  groups: SuggestionGroupWithItems[];
}

// ─── Preview ────────────────────────────────────────────────────────────
export interface PreviewResult {
  original_content: string;
  preview_content: string;
  highlighted_content: string;
  original_highlighted: string;
  applied_count: number;
}

export interface ApplyResult {
  version_id: string;
  applied_count: number;
  new_content: string;
}

// ─── Version ────────────────────────────────────────────────────────────
export interface VersionSnapshot {
  id: string;
  page_id: string;
  content: string;
  source_review_session_id: string | null;
  applied_suggestion_ids: string[];
  created_at: string;
  name: string | null;
}

// ─── Provider ───────────────────────────────────────────────────────────
export interface ProviderConfig {
  id: string;
  name: string;
  provider_type: string;
  endpoint: string | null;
  selected_model: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

// ─── Compare ────────────────────────────────────────────────────────────
export interface DiffSegment {
  tag: string;
  old_text: string;
  new_text: string;
}

export interface CompareResult {
  diff_segments: DiffSegment[];
  ai_analysis: string | null;
}

// ─── Category Helpers ───────────────────────────────────────────────────
export const categoryLabels: Record<string, string> = {
  spelling: 'Spelling',
  grammar: 'Grammar',
  vocabulary: 'Vocabulary',
  clarity: 'Clarity',
  fluency: 'Fluency',
  rewrite: 'Rewrite',
  recommendation: 'Recommendation',
};

export const categoryIcons: Record<string, string> = {
  spelling: '✏️',
  grammar: '📝',
  vocabulary: '📖',
  clarity: '💡',
  fluency: '🌊',
  rewrite: '🔄',
  recommendation: '💬',
};

// ─── Model Config ───────────────────────────────────────────────────────
export interface ModelConfig {
  id: string;
  provider_type: string;
  name: string;
  use_case: string;
  icon: string;
  is_custom: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}


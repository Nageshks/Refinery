<script setup lang="ts">
import { computed } from 'vue';
import type { SuggestionItem as SuggestionItemType } from '../types';

const props = defineProps<{
  item: SuggestionItemType;
  categoryLabel: string;
  hasConflictApproval?: boolean;
}>();

const emit = defineEmits<{
  approve: [id: string];
  reject: [id: string];
}>();

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function tokenize(text: string): string[] {
  return text.split(/(\s+|[.,\/#!$%\^&\*;:{}=\-_`~()?"'’“”[\]])/g).filter(Boolean);
}

function diffTokens(oldStr: string, newStr: string) {
  const oldTokens = tokenize(oldStr);
  const newTokens = tokenize(newStr);

  const dp: number[][] = Array(oldTokens.length + 1)
    .fill(null)
    .map(() => Array(newTokens.length + 1).fill(0));

  for (let i = 1; i <= oldTokens.length; i++) {
    for (let j = 1; j <= newTokens.length; j++) {
      if (oldTokens[i - 1] === newTokens[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  let i = oldTokens.length;
  let j = newTokens.length;
  const originalResult: { text: string; type: 'equal' | 'removed' }[] = [];
  const replacementResult: { text: string; type: 'equal' | 'added' }[] = [];

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldTokens[i - 1] === newTokens[j - 1]) {
      const match = oldTokens[i - 1];
      originalResult.unshift({ text: match, type: 'equal' });
      replacementResult.unshift({ text: match, type: 'equal' });
      i--;
      j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      replacementResult.unshift({ text: newTokens[j - 1], type: 'added' });
      j--;
    } else {
      originalResult.unshift({ text: oldTokens[i - 1], type: 'removed' });
      i--;
    }
  }

  return { originalResult, replacementResult };
}

const formattedDiffs = computed(() => {
  const { originalResult, replacementResult } = diffTokens(
    props.item.original_text || '',
    props.item.replacement_text || ''
  );

  const formattedOriginal = originalResult
    .map(token => {
      const escaped = escapeHtml(token.text);
      if (token.type === 'removed') {
        return `<span class="word-removed">${escaped}</span>`;
      }
      return escaped;
    })
    .join('');

  const formattedReplacement = replacementResult
    .map(token => {
      const escaped = escapeHtml(token.text);
      if (token.type === 'added') {
        return `<span class="word-added">${escaped}</span>`;
      }
      return escaped;
    })
    .join('');

  return { formattedOriginal, formattedReplacement };
});
</script>

<template>
  <div 
    :class="['suggestion-card-minimal', `state-${item.approval_state}`, { 'has-conflict-border': hasConflictApproval && item.approval_state !== 'approved' }]" 
    :data-suggestion-card-id="item.id"
    @click="emit('approve', item.id)"
  >
    <div class="card-meta-row">
      <div style="display: inline-flex; align-items: center; gap: 6px;">
        <span class="category-pill-label">{{ categoryLabel }}</span>
        <span v-if="item.conflict_state === 'overlapping'" class="alternative-badge">⚡ Alternative</span>
      </div>
      <span v-if="item.approval_state === 'approved'" class="approved-badge-tick">✓ Approved</span>
    </div>

    <div class="card-body-row">
      <span class="original-inline" v-html="formattedDiffs.formattedOriginal"></span>
      <span class="inline-arrow">
        <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
      </span>
      <span class="replacement-inline" v-html="formattedDiffs.formattedReplacement"></span>
    </div>

    <p v-if="item.explanation" class="suggestion-brief-desc">{{ item.explanation }}</p>

    <div v-if="hasConflictApproval && item.approval_state !== 'approved'" class="conflict-warning-note">
      <span class="warning-icon-bulb">💡</span>
      <span class="warning-text-note">Overlaps with an approved suggestion. Accepting this will swap them.</span>
    </div>
  </div>
</template>

<style scoped>
.suggestion-card-minimal {
  padding: 12px 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  width: 100%;
  cursor: pointer;
  user-select: none;
}

.suggestion-card-minimal:hover {
  border-color: var(--border-default);
  background: var(--bg-hover);
  box-shadow: var(--shadow-md);
}

.suggestion-card-minimal:active {
  transform: scale(0.985);
  transition: transform var(--transition-fast);
}

/* State Styling */
.suggestion-card-minimal.state-approved {
  background: rgba(16, 185, 129, 0.04);
  border-color: rgba(16, 185, 129, 0.35);
  box-shadow: 0 0 0 1px rgba(16, 185, 129, 0.15), var(--shadow-sm);
}
.suggestion-card-minimal.state-approved:hover {
  background: rgba(16, 185, 129, 0.06);
  border-color: rgba(16, 185, 129, 0.5);
  box-shadow: 0 0 0 1px rgba(16, 185, 129, 0.25), var(--shadow-md);
}
.suggestion-card-minimal.state-rejected {
  opacity: 0.45;
  background: rgba(255, 255, 255, 0.01);
  border-color: var(--border-subtle);
}
html[data-theme="light"] .suggestion-card-minimal.state-approved {
  background: rgba(5, 150, 105, 0.04);
  border-color: rgba(5, 150, 105, 0.3);
  box-shadow: 0 0 0 1px rgba(5, 150, 105, 0.1), var(--shadow-sm);
}
html[data-theme="light"] .suggestion-card-minimal.state-approved:hover {
  background: rgba(5, 150, 105, 0.06);
  border-color: rgba(5, 150, 105, 0.45);
  box-shadow: 0 0 0 1px rgba(5, 150, 105, 0.2), var(--shadow-md);
}

/* Meta row */
.card-meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2px;
}

.category-pill-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted);
  letter-spacing: 0.06em;
}

/* Approved Badge Styling */
.approved-badge-tick {
  font-size: 9px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.08);
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  animation: badge-pulse 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

html[data-theme="light"] .approved-badge-tick {
  color: #059669;
  background: rgba(5, 150, 105, 0.08);
}

@keyframes badge-pulse {
  0% {
    transform: scale(0.9);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Body row for inline comparisons */
.card-body-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  font-size: var(--font-size-base);
  line-height: var(--line-height-normal);
  color: var(--text-primary);
}

.original-inline {
  color: var(--text-secondary);
  opacity: 0.85;
}

.replacement-inline {
  font-weight: 500;
}

.inline-arrow {
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  opacity: 0.65;
  font-size: 11px;
}

/* Inline Diff highlights */
:deep(.word-removed) {
  color: var(--color-error);
  background: rgba(248, 113, 113, 0.08);
  text-decoration: line-through;
  padding: 1px 4px;
  border-radius: var(--radius-xs);
  font-weight: 400;
}

:deep(.word-added) {
  color: var(--color-success);
  background: rgba(52, 211, 153, 0.08);
  padding: 1px 4px;
  border-radius: var(--radius-xs);
  font-weight: 600;
}

html[data-theme="light"] :deep(.word-removed) {
  color: #DC2626;
  background: rgba(220, 38, 38, 0.06);
}

html[data-theme="light"] :deep(.word-added) {
  color: #059669;
  background: rgba(5, 150, 105, 0.07);
}

/* Brief description */
.suggestion-brief-desc {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.4;
  font-weight: 400;
}

/* Highlight glow pulse animation (re-used for search feedback) */
@keyframes highlight-glow-pulse {
  0% {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 4px var(--accent-subtle);
  }
  50% {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 6px rgba(139, 92, 246, 0.15);
  }
  100% {
    border-color: var(--border-subtle);
    box-shadow: var(--shadow-sm);
  }
}

.suggestion-card-minimal.highlight-glow {
  animation: highlight-glow-pulse 2.2s cubic-bezier(0.25, 0.8, 0.25, 1);
  z-index: 10;
}

/* Conflict Styles */
.suggestion-card-minimal.has-conflict-border {
  border-color: rgba(217, 119, 6, 0.2);
  box-shadow: 0 0 0 1px rgba(217, 119, 6, 0.05), var(--shadow-sm);
}

.suggestion-card-minimal.has-conflict-border:hover {
  border-color: rgba(217, 119, 6, 0.35);
  background: var(--bg-hover);
}

.alternative-badge {
  font-size: 8.5px;
  font-weight: 700;
  color: #d97706; /* amber-600 */
  background: rgba(217, 119, 6, 0.08);
  padding: 1px 5px;
  border-radius: var(--radius-xs);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  animation: badge-pulse 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

html[data-theme="light"] .alternative-badge {
  color: #b45309; /* amber-700 */
  background: rgba(180, 83, 9, 0.08);
}

.conflict-warning-note {
  font-size: 10px;
  color: #d97706; /* amber-600 */
  background: rgba(217, 119, 6, 0.05);
  border: 1px solid rgba(217, 119, 6, 0.15);
  border-radius: var(--radius-sm);
  padding: 5px 8px;
  margin-top: 2px;
  line-height: 1.4;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

html[data-theme="light"] .conflict-warning-note {
  color: #b45309; /* amber-700 */
  background: rgba(180, 83, 9, 0.04);
  border-color: rgba(180, 83, 9, 0.15);
}

.warning-icon-bulb {
  font-size: 11px;
  opacity: 0.95;
}

.warning-text-note {
  flex: 1;
}
</style>

<script setup lang="ts">
import { computed } from 'vue';
import type { SuggestionItem as SuggestionItemType } from '../types';

const props = defineProps<{
  item: SuggestionItemType;
  categoryLabel: string;
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
  <div :class="['suggestion-card-minimal', `state-${item.approval_state}`]" :data-suggestion-card-id="item.id">
    <div class="card-meta-row">
      <span class="category-pill-label">{{ categoryLabel }}</span>
      <div class="suggestion-actions">
        <button
          type="button"
          :class="['btn-action btn-approve', { active: item.approval_state === 'approved' }]"
          @click="emit('approve', item.id)"
          title="Approve change"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
        <button
          type="button"
          :class="['btn-action btn-reject', { active: item.approval_state === 'rejected' }]"
          @click="emit('reject', item.id)"
          title="Reject change"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </div>

    <div class="card-body-row">
      <span class="original-inline" v-html="formattedDiffs.formattedOriginal"></span>
      <span class="inline-arrow">
        <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
      </span>
      <span class="replacement-inline" v-html="formattedDiffs.formattedReplacement"></span>
    </div>

    <p v-if="item.explanation" class="suggestion-brief-desc">{{ item.explanation }}</p>
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
}

.suggestion-card-minimal:hover {
  border-color: var(--border-default);
  background: var(--bg-hover);
  box-shadow: var(--shadow-md);
}

/* State Styling */
.suggestion-card-minimal.state-approved {
  background: rgba(52, 211, 153, 0.03);
  border-color: rgba(52, 211, 153, 0.3);
}
.suggestion-card-minimal.state-rejected {
  opacity: 0.45;
  background: rgba(255, 255, 255, 0.01);
  border-color: var(--border-subtle);
}
html[data-theme="light"] .suggestion-card-minimal.state-approved {
  background: rgba(5, 150, 105, 0.035);
  border-color: rgba(5, 150, 105, 0.25);
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

/* Actions styling */
.suggestion-actions {
  display: flex;
  gap: 6px;
  opacity: 0.35;
  transition: opacity var(--transition-fast);
}

.suggestion-card-minimal:hover .suggestion-actions {
  opacity: 1;
}

.btn-action {
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-xs);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  outline: none;
  padding: 0;
}

.btn-action:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
  border-color: var(--border-strong);
}

.btn-approve:hover,
.btn-approve.active {
  background: var(--color-success-bg) !important;
  border-color: var(--color-success-border) !important;
  color: var(--color-success) !important;
}

.btn-reject:hover,
.btn-reject.active {
  background: var(--color-error-bg) !important;
  border-color: var(--color-error-border) !important;
  color: var(--color-error) !important;
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
</style>

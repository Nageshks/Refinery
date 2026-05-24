<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { usePagesStore } from '../stores/pages';
import { useReview } from '../composables/useReview';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { Store } from '@tauri-apps/plugin-store';
import { rewriteSelection, updatePage } from '../composables/useTauri';
import { categoryIcons, categoryLabels } from '../types';

const pagesStore = usePagesStore();
const review = useReview();
const { preview } = review;
const providersStore = useProvidersStore();
const appStore = useAppStore();

const rightPane = ref<HTMLElement | null>(null);



// Watch active page id changes to fetch/update preview or load existing review session
watch(() => pagesStore.activePageId, async (newPageId) => {
  if (newPageId) {
    const page = pagesStore.activePage;
    if (page && page.last_review_session_id) {
      if (!review.reviewResult.value || review.reviewResult.value.session.id !== page.last_review_session_id) {
        await review.loadReviewSession(newPageId, page.last_review_session_id);
      } else {
        await review.refreshPreview(newPageId);
      }
    } else {
      // Clear if loaded session is for another page
      if (review.reviewResult.value && review.reviewResult.value.session.page_id !== newPageId) {
        review.clearReview();
      }
    }
  }
}, { immediate: true });

// Dynamic Class Mapper for Proposed View (adding severity & category styles)
const processedHighlightedContent = computed(() => {
  const html = preview.value?.highlighted_content || preview.value?.preview_content || pagesStore.activePage?.content || '';
  if (!html || review.groups.value.length === 0) return html;

  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const highlights = doc.querySelectorAll('.preview-highlight-inserted, .preview-highlight-original');

  highlights.forEach(el => {
    const id = el.getAttribute('data-suggestion-id');
    if (id) {
      let foundItem: any = null;
      let foundGroup: any = null;
      for (const g of review.groups.value) {
        const item = g.items.find(i => i.id === id);
        if (item) {
          foundItem = item;
          foundGroup = g.group;
          break;
        }
      }

      if (foundItem && foundGroup) {
        el.classList.add(`state-${foundItem.approval_state}`);
        el.classList.add(`category-${foundGroup.category}`);
        el.setAttribute('title', `${foundGroup.label || foundGroup.category}: ${foundItem.explanation}`);
      }
    }
  });

  return doc.body.innerHTML;
});



// Inline Popover Suggester State
const popoverOpen = ref(false);
const popoverPosition = ref({ top: 0, left: 0 });
const popoverPlacement = ref<'top' | 'bottom'>('bottom');
const activeItem = ref<any>(null);
const activeGroup = ref<any>(null); // SuggestionGroupWithItems

const activeAlternatives = computed(() => {
  if (!activeGroup.value || !activeItem.value) return [];
  return activeGroup.value.items.filter((alt: any) =>
    alt.span_start === activeItem.value.span_start &&
    alt.span_end === activeItem.value.span_end
  );
});

const handlePreviewClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  const highlight = target.closest('.preview-highlight-inserted, .preview-highlight-original') as HTMLElement;
  if (highlight) {
    e.stopPropagation();
    const suggestionId = highlight.getAttribute('data-suggestion-id');
    if (suggestionId) {
      // Find suggestion item and group
      let foundItem: any = null;
      let foundGroup: any = null;
      for (const g of review.groups.value) {
        const item = g.items.find(i => i.id === suggestionId);
        if (item) {
          foundItem = item;
          foundGroup = g;
          break;
        }
      }

      if (foundItem) {
        activeItem.value = foundItem;
        activeGroup.value = foundGroup;

        // Position the popover relative to the split-review-container
        const rect = highlight.getBoundingClientRect();
        const container = document.querySelector('.split-review-container');
        if (container) {
          const containerRect = container.getBoundingClientRect();
          const hasAlternatives = activeAlternatives.value.length > 1;
          const estimatedHeight = hasAlternatives ? 330 : 230;
          const spaceBelow = containerRect.height - (rect.bottom - containerRect.top);

          if (spaceBelow < estimatedHeight + 16) {
            popoverPlacement.value = 'top';
            popoverPosition.value = {
              top: rect.top - containerRect.top - estimatedHeight - 12,
              left: Math.max(16, Math.min(containerRect.width - 336, rect.left - containerRect.left + (rect.width / 2) - 160))
            };
          } else {
            popoverPlacement.value = 'bottom';
            popoverPosition.value = {
              top: rect.bottom - containerRect.top + 8,
              left: Math.max(16, Math.min(containerRect.width - 336, rect.left - containerRect.left + (rect.width / 2) - 160))
            };
          }
          popoverOpen.value = true;
          return;
        }
      }
    }
  }
};

const handleApprovePopover = async (item: any) => {
  if (activeGroup.value) {
    for (const alt of activeGroup.value.items) {
      if (alt.id !== item.id && alt.approval_state === 'approved') {
        await review.toggleItemApproval(alt.id, 'pending');
      }
    }
  }
  await review.toggleItemApproval(item.id, 'approved');
  if (pagesStore.activePageId) {
    await review.refreshPreview(pagesStore.activePageId);
  }
  popoverOpen.value = false;
  appStore.notify('Approved suggestion!', 'success');
};

const handleRejectPopover = async (item: any) => {
  await review.toggleItemApproval(item.id, 'rejected');
  if (pagesStore.activePageId) {
    await review.refreshPreview(pagesStore.activePageId);
  }
  popoverOpen.value = false;
  appStore.notify('Dismissed suggestion', 'success');
};

// Selection AI Rewrite State
const showRewriteToolbar = ref(false);
const rewriteToolbarPosition = ref({ top: 0, left: 0 });
const selectedText = ref('');
const loadingRewrites = ref(false);
const rewriteOptions = ref<string[]>([]);
const showRewriteDialog = ref(false);

const checkSelection = () => {
  const selection = window.getSelection();
  if (!selection || selection.isCollapsed || !selection.toString().trim()) {
    return;
  }

  const text = selection.toString().trim();
  // Valid selection constraints
  if (text.length > 2 && text.length < 500) {
    selectedText.value = text;
    try {
      const range = selection.getRangeAt(0);
      const rect = range.getBoundingClientRect();
      const container = document.querySelector('.split-review-container');
      if (container) {
        const containerRect = container.getBoundingClientRect();
        // Float 40px above selection
        rewriteToolbarPosition.value = {
          top: rect.top - containerRect.top - 40,
          left: Math.max(16, Math.min(containerRect.width - 150, rect.left - containerRect.left + (rect.width / 2) - 60))
        };
        showRewriteToolbar.value = true;
      }
    } catch (_) {}
  }
};

const handleTriggerRewrite = async () => {
  showRewriteToolbar.value = false;
  showRewriteDialog.value = true;
  loadingRewrites.value = true;
  rewriteOptions.value = [];

  const provider = providersStore.activeProvider;
  if (!provider) {
    appStore.notify('Configure an AI provider in Settings first', 'error');
    showRewriteDialog.value = false;
    return;
  }

  try {
    const store = await Store.load('credentials.json');
    const apiKey = await store.get<string>(`apikey_${provider.id}`);
    if (!apiKey) {
      appStore.notify('API key not configured in Settings.', 'error');
      showRewriteDialog.value = false;
      return;
    }

    const options = await rewriteSelection({
      apiKey,
      model: provider.selected_model,
      endpoint: provider.endpoint || undefined,
      selectedText: selectedText.value,
    });
    rewriteOptions.value = options;
  } catch (e: any) {
    appStore.notify(typeof e === 'string' ? e : 'Rewrite failed', 'error');
    showRewriteDialog.value = false;
  } finally {
    loadingRewrites.value = false;
  }
};

const handleApplyRewrite = async (option: string) => {
  const page = pagesStore.activePage;
  if (!page) return;

  const originalContent = page.content;
  if (originalContent.includes(selectedText.value)) {
    const newContent = originalContent.replace(selectedText.value, option);
    try {
      await updatePage(page.id, page.title, newContent);
      await pagesStore.selectPage(page.id); // Reload
      appStore.notify('Applied selection rewrite!', 'success');
      
      // Auto-retrigger review preview if review is active
      if (review.reviewResult.value) {
        await review.refreshPreview(page.id);
      }
    } catch (e) {
      appStore.notify('Failed to apply rewrite', 'error');
    }
  } else {
    // If exact HTML substring match fails, strip tags and try text match or notify user
    appStore.notify('Select simple text without formatting for inline rewriting.', 'info');
  }

  showRewriteDialog.value = false;
  // Clear selection
  window.getSelection()?.removeAllRanges();
};

const closePopoverAndToolbar = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.interactive-popover') && !target.closest('.preview-highlight-inserted') && !target.closest('.preview-highlight-original')) {
    popoverOpen.value = false;
  }
  if (!target.closest('.selection-rewrite-toolbar') && !target.closest('.rewrite-dialog-card')) {
    showRewriteToolbar.value = false;
  }
};

onMounted(() => {
  document.addEventListener('selectionchange', checkSelection);
  document.addEventListener('mousedown', closePopoverAndToolbar);
});

onUnmounted(() => {
  document.removeEventListener('selectionchange', checkSelection);
  document.removeEventListener('mousedown', closePopoverAndToolbar);
});
</script>

<template>
  <div class="split-review-container flex-col">
    <!-- Review Panes Area -->
    <div class="review-panes-area">
      <!-- Editing View Column (Always shows user text with highlights layered dynamically) -->
      <div class="split-column right-column">
        <div 
          ref="rightPane" 
          class="pane-scroll-area" 
        >
          <div 
            class="tiptap pane-content preview-content-interactive"
            @click="handlePreviewClick"
            v-html="processedHighlightedContent || pagesStore.activePage?.content || ''"
          ></div>
        </div>
      </div>
    </div>

    <!-- Beautiful Glassmorphic Inline Suggestions Popover (Wordtune-style) -->
    <transition name="popover-fade">
      <div 
        v-if="popoverOpen && activeItem"
        :class="['interactive-popover card', { 'placement-top': popoverPlacement === 'top', 'placement-bottom': popoverPlacement === 'bottom' }]"
        :style="{ top: popoverPosition.top + 'px', left: popoverPosition.left + 'px' }"
      >
        <div class="popover-arrow"></div>
        <div class="popover-header">
          <div class="popover-category">
            <span class="category-icon">{{ categoryIcons[activeGroup?.group.category] || '✏️' }}</span>
            <span class="category-name">{{ categoryLabels[activeGroup?.group.category] || activeGroup?.group.category }}</span>
          </div>
          <div v-if="activeItem.approval_state === 'approved'" class="popover-state-badge badge-approved">✓ Approved</div>
          <div v-else-if="activeItem.approval_state === 'rejected'" class="popover-state-badge badge-rejected">✗ Dismissed</div>
        </div>

        <p class="popover-explanation">{{ activeItem.explanation }}</p>

        <!-- Compact Comparison Diffs -->
        <div class="popover-comparison-block">
          <div class="comparison-row row-original">
            <span class="diff-indicator label-red">-</span>
            <span class="comparison-text text-strike">{{ activeItem.original_text }}</span>
          </div>
          <div class="comparison-row row-proposed">
            <span class="diff-indicator label-green">+</span>
            <span class="comparison-text text-bold">{{ activeItem.replacement_text }}</span>
          </div>
        </div>

        <!-- Alternatives Selection Picker (if group contains multiple options) -->
        <div v-if="activeAlternatives.length > 1" class="popover-alternatives">
          <div class="alternatives-title">Alternatives:</div>
          <div class="alternatives-scroll">
            <button
              v-for="alt in activeAlternatives"
              :key="alt.id"
              :class="['alt-choice-btn', { active: alt.id === activeItem.id }]"
              @click="activeItem = alt"
            >
              <div class="alt-choice-text">{{ alt.replacement_text }}</div>
              <div class="alt-choice-meta">{{ alt.explanation }}</div>
            </button>
          </div>
        </div>

        <!-- Actions Footer -->
        <div class="popover-actions">
          <button 
            type="button"
            class="btn btn-ghost btn-xs text-muted"
            @click="handleRejectPopover(activeItem)"
          >
            {{ activeItem.approval_state === 'rejected' ? 'Restore' : 'Dismiss' }}
          </button>
          <button
            type="button"
            class="btn btn-xs btn-primary"
            @click="handleApprovePopover(activeItem)"
          >
            {{ activeItem.approval_state === 'approved' ? '✓ Approved' : 'Accept' }}
          </button>
        </div>
      </div>
    </transition>

    <!-- Selection AI Rewrite Floating Toolbar Trigger Button -->
    <div 
      v-if="showRewriteToolbar"
      class="selection-rewrite-toolbar"
      :style="{ top: rewriteToolbarPosition.top + 'px', left: rewriteToolbarPosition.left + 'px' }"
    >
      <button class="rewrite-trigger-btn" @click="handleTriggerRewrite">
        <span class="btn-stars">✨</span> Rewrite selection
      </button>
    </div>

    <!-- AI Rewrite Options Dialog Backdrop & Card -->
    <transition name="popover-fade">
      <div 
        v-if="showRewriteDialog"
        class="rewrite-dialog-card card interactive-popover"
        :style="{ top: (rewriteToolbarPosition.top + 45) + 'px', left: rewriteToolbarPosition.left + 'px' }"
      >
        <div class="popover-arrow"></div>
        <div class="popover-header">
          <div class="popover-category">
            <span class="category-icon">✨</span>
            <span class="category-name">AI Rewrite Options</span>
          </div>
          <button class="btn-close-rewrite" @click="showRewriteDialog = false">×</button>
        </div>

        <div class="selected-context-preview">
          "{{ selectedText }}"
        </div>

        <!-- Loading state -->
        <div v-if="loadingRewrites" class="rewrite-loading-state">
          <div class="skeleton" style="height: 32px; margin-bottom: 6px;" />
          <div class="skeleton" style="height: 32px; margin-bottom: 6px;" />
          <div class="skeleton" style="height: 32px;" />
          <p class="loading-hint">Generating premium tone rewrites...</p>
        </div>

        <!-- Options list -->
        <div v-else class="rewrite-options-list">
          <button
            v-for="(option, index) in rewriteOptions"
            :key="index"
            class="rewrite-option-row"
            @click="handleApplyRewrite(option)"
          >
            <div class="rewrite-option-header">
              <span class="tone-tag">{{ index === 0 ? 'Professional' : index === 1 ? 'Fluent/Natural' : 'Bold/Engaging' }}</span>
            </div>
            <div class="rewrite-option-content">{{ option }}</div>
          </button>

          <div v-if="rewriteOptions.length === 0" class="rewrite-empty-options">
            No alternatives generated. Try selecting a complete sentence.
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.split-review-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
  position: relative;
}

.review-panes-area {
  display: flex;
  flex: 1;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.split-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.right-column {
  background: var(--bg-primary);
}



.pane-scroll-area {
  flex: 1;
  overflow-y: auto;
  height: 100%;
  position: relative;
}

/* Perfect editorial layout styling */
.pane-content {
  padding: var(--space-6) var(--space-8);
  min-height: 100%;
  max-width: 800px;
  margin: 0 auto;
}

.preview-content-interactive :deep(.preview-highlight-inserted) {
  background-color: rgba(52, 211, 153, 0.06);
  border-bottom: 1.5px dashed rgba(52, 211, 153, 0.35);
  padding: 1px 3px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.preview-content-interactive :deep(.preview-highlight-inserted:hover) {
  background-color: rgba(52, 211, 153, 0.15);
  border-bottom-color: rgba(52, 211, 153, 0.85);
  box-shadow: 0 0 0 1px rgba(52, 211, 153, 0.2);
}

/* Category-specific underlines and background tints for Proposed Preview (Pending States) */
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-spelling),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-grammar) {
  background-color: var(--color-error-bg);
  border-bottom: 2px dashed var(--color-error);
}
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-spelling:hover),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-grammar:hover) {
  background-color: rgba(248, 113, 113, 0.18);
  border-bottom-color: var(--color-error-muted);
  box-shadow: 0 0 0 1px rgba(248, 113, 113, 0.25);
}

.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-clarity),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-fluency),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-recommendation) {
  background-color: var(--color-warning-bg);
  border-bottom: 2px dashed var(--color-warning);
}
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-clarity:hover),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-fluency:hover),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-recommendation:hover) {
  background-color: rgba(251, 191, 36, 0.18);
  border-bottom-color: var(--color-warning);
  box-shadow: 0 0 0 1px rgba(251, 191, 36, 0.25);
}

.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-vocabulary),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-rewrite) {
  background-color: rgba(139, 92, 246, 0.06);
  border-bottom: 2px dashed var(--accent-primary);
}
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-vocabulary:hover),
.preview-content-interactive :deep(.preview-highlight-inserted.state-pending.category-rewrite:hover) {
  background-color: rgba(139, 92, 246, 0.15);
  border-bottom-color: var(--accent-muted);
  box-shadow: 0 0 0 1px rgba(139, 92, 246, 0.25);
}

/* Beautiful style for Approved highlights in the Proposed Preview */
.preview-content-interactive :deep(.preview-highlight-inserted.state-approved) {
  background-color: rgba(52, 211, 153, 0.04);
  border-bottom: 1.5px solid rgba(52, 211, 153, 0.45);
}
.preview-content-interactive :deep(.preview-highlight-inserted.state-approved:hover) {
  background-color: rgba(52, 211, 153, 0.12);
  border-bottom-color: rgba(52, 211, 153, 0.85);
  box-shadow: 0 0 0 1px rgba(52, 211, 153, 0.2);
}

.preview-content-interactive :deep(.preview-highlight-original) {
  padding: 1px 3px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
  cursor: pointer;
  position: relative;
  display: inline;
}

.preview-content-interactive :deep(.preview-original-pending) {
  background-color: rgba(139, 92, 246, 0.08);
  border-bottom: 1.5px dashed rgba(139, 92, 246, 0.4);
}

.preview-content-interactive :deep(.preview-original-pending:hover) {
  background-color: rgba(139, 92, 246, 0.18);
  border-bottom-color: rgba(139, 92, 246, 0.85);
  box-shadow: 0 0 0 1px rgba(139, 92, 246, 0.25);
}

/* Category-specific color-coding for original highlights (pending state) */
.preview-content-interactive :deep(.preview-original-pending.category-spelling),
.preview-content-interactive :deep(.preview-original-pending.category-grammar) {
  background-color: var(--color-error-bg);
  border-bottom: 1.5px dashed var(--color-error);
}
.preview-content-interactive :deep(.preview-original-pending.category-clarity),
.preview-content-interactive :deep(.preview-original-pending.category-fluency),
.preview-content-interactive :deep(.preview-original-pending.category-recommendation) {
  background-color: var(--color-warning-bg);
  border-bottom: 1.5px dashed var(--color-warning);
}
.preview-content-interactive :deep(.preview-original-pending.category-vocabulary),
.preview-content-interactive :deep(.preview-original-pending.category-rewrite) {
  background-color: rgba(139, 92, 246, 0.08);
  border-bottom: 1.5px dashed var(--accent-primary);
}

.preview-content-interactive :deep(.preview-original-approved) {
  background-color: rgba(248, 113, 113, 0.08);
  border-bottom: 1.5px dashed rgba(248, 113, 113, 0.35);
  text-decoration: line-through;
  text-decoration-color: var(--color-error-muted);
  color: var(--text-secondary);
}

.preview-content-interactive :deep(.preview-original-approved:hover) {
  background-color: rgba(248, 113, 113, 0.15);
  border-bottom-color: rgba(248, 113, 113, 0.8);
  box-shadow: 0 0 0 1px rgba(248, 113, 113, 0.2);
}

.preview-content-interactive :deep(.preview-original-rejected) {
  opacity: 0.65;
  border-bottom: 1px dotted var(--preview-rejected-border);
  text-decoration: line-through;
  text-decoration-style: dotted;
  color: var(--text-muted);
}

.preview-content-interactive :deep(.preview-original-rejected:hover) {
  opacity: 0.9;
  background-color: var(--preview-rejected-bg-hover);
  border-bottom-color: var(--preview-rejected-border-hover);
}

/* Skeleton Loading Animation Area */
.skeleton-content {
  display: flex;
  flex-direction: column;
  width: 100%;
}

/* Empty State Style */
.empty-review-split {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-8);
  height: 100%;
}

.minimalist-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: var(--space-2);
  max-width: 420px;
  animation: fade-in 0.4s ease-out;
}

.minimalist-empty-state h3 {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  letter-spacing: -0.015em;
  margin-bottom: 2px;
}

.minimalist-empty-state p {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.6;
  letter-spacing: -0.01em;
  margin: 0;
}

.minimalist-empty-state p strong {
  color: var(--text-secondary);
  font-weight: 500;
}

.minimalist-empty-state .minimal-hint {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.8;
  margin-top: var(--space-2);
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Beautiful Glassmorphic Inline Suggestions Popover (Wordtune-style) */
.interactive-popover {
  position: absolute;
  width: 330px;
  background: var(--bg-dropdown);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl), 0 0 0 1px rgba(255, 255, 255, 0.03);
  padding: var(--space-4);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.popover-arrow {
  position: absolute;
  top: -6px;
  left: 160px;
  width: 12px;
  height: 12px;
  background: var(--bg-dropdown);
  border-left: 1px solid var(--border-strong);
  border-top: 1px solid var(--border-strong);
  transform: rotate(45deg);
  pointer-events: none;
}

.interactive-popover.placement-top .popover-arrow {
  top: auto;
  bottom: -6px;
  border-left: none;
  border-top: none;
  border-right: 1px solid var(--border-strong);
  border-bottom: 1px solid var(--border-strong);
}

.popover-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.popover-category {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.category-icon {
  font-size: 14px;
}

.category-name {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--accent-muted);
}

.popover-state-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.popover-state-badge.badge-approved {
  background: var(--color-success-bg);
  color: var(--color-success);
  border: 1px solid var(--color-success-border);
}

.popover-state-badge.badge-rejected {
  background: var(--color-error-bg);
  color: var(--color-error);
  border: 1px solid var(--color-error-border);
}

.popover-explanation {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.4;
}

/* Compact Comparison Diffs */
.popover-comparison-block {
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.comparison-row {
  display: flex;
  gap: var(--space-2);
  align-items: flex-start;
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
}

.diff-indicator {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 13px;
  width: 10px;
  display: inline-block;
}

.diff-indicator.label-red {
  color: var(--color-error);
}

.diff-indicator.label-green {
  color: var(--color-success);
}

.comparison-text {
  flex: 1;
  word-break: break-word;
}

.comparison-text.text-strike {
  text-decoration: line-through;
  color: var(--text-muted);
}

.comparison-text.text-bold {
  font-weight: 500;
  color: var(--text-primary);
}

/* Alternatives Selection Picker */
.popover-alternatives {
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-2);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.alternatives-title {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.alternatives-scroll {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  max-height: 140px;
  overflow-y: auto;
  padding-right: 2px;
}

.alt-choice-btn {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  text-align: left;
  padding: 6px var(--space-3);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  width: 100%;
}

.alt-choice-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-default);
}

.alt-choice-btn.active {
  background: rgba(139, 92, 246, 0.08);
  border-color: var(--accent-primary);
}

.alt-choice-text {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.alt-choice-meta {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* Actions Footer */
.popover-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-3);
  margin-top: var(--space-1);
}

/* Selection AI Rewrite Floating Toolbar */
.selection-rewrite-toolbar {
  position: absolute;
  z-index: 1000;
}

.rewrite-trigger-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 6px var(--space-3);
  background: var(--bg-dropdown);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-xl);
  color: var(--text-primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: var(--shadow-lg), 0 0 0 1px rgba(255, 255, 255, 0.02);
  transition: all var(--transition-fast);
}

.rewrite-trigger-btn:hover {
  transform: translateY(-1px);
  border-color: var(--accent-primary);
  background: var(--bg-hover);
}

.btn-stars {
  color: var(--accent-muted);
}

/* AI Rewrite Options Dialog Backdrop & Card */
.rewrite-dialog-card {
  position: absolute;
  width: 340px;
  background: var(--bg-dropdown);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  padding: var(--space-4);
  z-index: 1001;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.btn-close-rewrite {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
  padding: 0 4px;
}

.btn-close-rewrite:hover {
  color: var(--text-primary);
}

.selected-context-preview {
  font-style: italic;
  font-size: 11px;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.02);
  padding: 6px var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  max-height: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rewrite-loading-state {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0;
}

.loading-hint {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
  margin-top: 4px;
}

.rewrite-options-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.rewrite-option-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  text-align: left;
  padding: var(--space-3);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  width: 100%;
}

.rewrite-option-row:hover {
  background: var(--bg-hover);
  border-color: var(--accent-primary);
  transform: translateY(-1px);
}

.rewrite-option-header {
  margin-bottom: var(--space-1);
}

.tone-tag {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 1px 5px;
  border-radius: var(--radius-xs);
  background: rgba(139, 92, 246, 0.12);
  color: var(--accent-muted);
}

.rewrite-option-content {
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-primary);
  word-break: break-word;
}

.rewrite-empty-options {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  text-align: center;
  padding: var(--space-4) 0;
}

/* Vue transitions */
.popover-fade-enter-active,
.popover-fade-leave-active {
  transition: opacity var(--transition-fast), transform var(--transition-fast);
}

.popover-fade-enter-from,
.popover-fade-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.98);
}
</style>

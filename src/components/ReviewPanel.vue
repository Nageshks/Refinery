<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { usePagesStore } from '../stores/pages';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { useReviewStore } from '../stores/review';
import { useReview } from '../composables/useReview';
import { Store } from '@tauri-apps/plugin-store';
import SuggestionItemComp from './SuggestionItem.vue';
import { categoryLabels } from '../types';
import { setActiveProvider } from '../composables/useTauri';

const pagesStore = usePagesStore();
const providersStore = useProvidersStore();
const appStore = useAppStore();
const reviewStore = useReviewStore();
const review = useReview();

const isDragging = ref(false);
const providerKeys = ref<Record<string, string>>({});

const initResize = (e: MouseEvent) => {
  e.preventDefault();
  isDragging.value = true;
  document.body.classList.add('is-resizing');

  const startWidth = appStore.reviewPanelWidth;
  const startX = e.clientX;

  const doResize = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX;
    appStore.reviewPanelWidth = Math.max(300, Math.min(600, startWidth - deltaX));
  };

  const stopResize = () => {
    isDragging.value = false;
    document.body.classList.remove('is-resizing');
    window.removeEventListener('mousemove', doResize);
    window.removeEventListener('mouseup', stopResize);
  };

  window.addEventListener('mousemove', doResize);
  window.addEventListener('mouseup', stopResize);
};

// Model selector state & options
const showDropdown = ref(false);
const dropdownEl = ref<HTMLElement | null>(null);

const loadApiKeys = async () => {
  try {
    const store = await Store.load('credentials.json');
    for (const provider of providersStore.providers) {
      const key = await store.get<string>(`apikey_${provider.id}`);
      if (key) {
        providerKeys.value[provider.id] = key;
      } else {
        providerKeys.value[provider.id] = '';
      }
    }
  } catch (e) {
    console.error('Failed to load api keys in review panel:', e);
  }
};

const groupedModelsList = computed(() => {
  const list = providersStore.models.filter(m => m.enabled);
  const groups: Record<string, { providerName: string; providerId: string; models: any[] }> = {};
  for (const model of list) {
    const provider = providersStore.providers.find(p => p.provider_type === model.provider_type);
    if (!provider) continue;
    const key = providerKeys.value[provider.id];
    if (!key) continue; // Only show if token/API key is entered!
    
    if (!groups[model.provider_type]) {
      groups[model.provider_type] = {
        providerName: provider.name,
        providerId: provider.id,
        models: []
      };
    }
    groups[model.provider_type].models.push(model);
  }
  return Object.values(groups);
});

const selectedModelDetail = computed(() => {
  const active = providersStore.activeProvider;
  if (!active) return null;
  
  // Find in all provider models
  return providersStore.models.find(m => m.id === active.selected_model && m.provider_type === active.provider_type) || {
    id: active.selected_model,
    name: active.selected_model.split('/').pop() || active.selected_model,
    useCase: 'Custom Model',
    icon: '🤖'
  };
});

const selectModel = async (model: any) => {
  try {
    const provider = providersStore.providers.find(p => p.provider_type === model.provider_type);
    if (!provider) return;

    // 1. Update this provider's default model and mark enabled
    await providersStore.saveProvider({
      id: provider.id,
      name: provider.name,
      providerType: provider.provider_type,
      endpoint: provider.endpoint || undefined,
      selectedModel: model.id,
      enabled: true
    });

    // 2. Set active in backend
    await setActiveProvider(provider.id);

    // 3. Refresh providers
    await providersStore.fetchProviders();

    appStore.notify(`Switched to ${model.name} (${provider.name})`, 'success');
  } catch (e) {
    console.error('Failed to switch active model:', e);
    appStore.notify('Failed to switch active model', 'error');
  }
  showDropdown.value = false;
};

const closeDropdown = (e: MouseEvent) => {
  if (dropdownEl.value && !dropdownEl.value.contains(e.target as Node)) {
    showDropdown.value = false;
  }
};

onMounted(async () => {
  document.addEventListener('click', closeDropdown);
  await providersStore.fetchProviders();
  await loadApiKeys();
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
});

const handleReview = async () => {
  const page = pagesStore.activePage;
  const provider = providersStore.activeProvider;
  if (!page || !provider) {
    appStore.notify('Configure a provider in Settings first', 'error');
    return;
  }
  try {
    const store = await Store.load('credentials.json');
    const apiKey = await store.get<string>(`apikey_${provider.id}`);
    if (!apiKey) {
      appStore.notify('API key not configured. Go to Settings.', 'error');
      return;
    }
    const context = appStore.activeSidebarTab === 'polish' ? reviewStore.editorialContext : '';
    await review.doReview(page.id, apiKey, provider.selected_model, provider.endpoint || undefined, context);
    appStore.notify(`Review complete: ${review.totalCount()} suggestions`, 'success');
  } catch (e: any) {
    appStore.notify(typeof e === 'string' ? e : 'Review failed', 'error');
  }
};

const handleApprove = async (itemId: string) => {
  const item = review.groups.value.flatMap(g => g.items).find(i => i.id === itemId);
  const newState = item?.approval_state === 'approved' ? 'pending' : 'approved';
  await review.toggleItemApproval(itemId, newState);
  if (pagesStore.activePageId) await review.refreshPreview(pagesStore.activePageId);
};

const handleReject = async (itemId: string) => {
  const item = review.groups.value.flatMap(g => g.items).find(i => i.id === itemId);
  const newState = item?.approval_state === 'rejected' ? 'pending' : 'rejected';
  await review.toggleItemApproval(itemId, newState);
  if (pagesStore.activePageId) await review.refreshPreview(pagesStore.activePageId);
};

const handleApply = async () => {
  if (!pagesStore.activePageId) return;
  try {
    const result = await review.applyChanges(pagesStore.activePageId);
    await pagesStore.selectPage(pagesStore.activePageId);
    appStore.notify(`Applied ${result.applied_count} changes`, 'success');
    appStore.setView('edit');
  } catch (e: any) {
    appStore.notify(typeof e === 'string' ? e : 'Apply failed', 'error');
  }
};

const handleDismissReview = async () => {
  if (pagesStore.activePageId) {
    await review.clearReview(pagesStore.activePageId);
  } else {
    review.clearReview();
  }
  appStore.setView('edit');
  appStore.notify('Polishing session cleared', 'success');
};

// Flattened list of suggestions with injected category label and category tag
const allSuggestions = computed(() => {
  return review.groups.value.flatMap(gwi => {
    const categoryLabel = gwi.group.label || categoryLabels[gwi.group.category] || gwi.group.category;
    const category = gwi.group.category;
    return gwi.items.map(item => ({
      ...item,
      category,
      categoryLabel
    }));
  });
});

const isQuickFix = (item: any) => {
  const category = (item.category || '').toLowerCase();
  const label = (item.categoryLabel || '').toLowerCase();
  return category === 'spelling' || 
         label.includes('typo') || 
         label.includes('spelling') || 
         ((category === 'grammar' || label.includes('grammar')) && item.original_text.length < 15);
};

const quickFixes = computed(() => {
  return allSuggestions.value.filter(isQuickFix);
});

const groupActiveIndices = ref<Record<string, number>>({});

const stylisticGroups = computed(() => {
  const stylisticItems = allSuggestions.value.filter(item => !isQuickFix(item));
  
  // Sort by span_start
  const sorted = [...stylisticItems].sort((a, b) => a.span_start - b.span_start);
  
  const groupsList: any[][] = [];
  for (const item of sorted) {
    let added = false;
    for (const group of groupsList) {
      const overlaps = group.some(other => 
        !(item.span_end <= other.span_start || other.span_end <= item.span_start)
      );
      if (overlaps) {
        group.push(item);
        added = true;
        break;
      }
    }
    if (!added) {
      groupsList.push([item]);
    }
  }
  
  return groupsList.map((items) => {
    const groupKey = items.map(i => i.id).sort().join(',');
    const approvedIndex = items.findIndex(i => i.approval_state === 'approved');
    
    let activeIndex = 0;
    if (approvedIndex !== -1) {
      activeIndex = approvedIndex;
    } else if (groupActiveIndices.value[groupKey] !== undefined) {
      if (groupActiveIndices.value[groupKey] < items.length) {
        activeIndex = groupActiveIndices.value[groupKey];
      }
    }
    
    return {
      key: groupKey,
      items,
      activeIndex
    };
  });
});

const setGroupActiveIndex = (groupKey: string, idx: number) => {
  groupActiveIndices.value[groupKey] = idx;
};

const approveAllQuickFixes = async () => {
  const pendingFixes = quickFixes.value.filter(i => i.approval_state !== 'approved');
  if (pendingFixes.length === 0) return;
  
  for (const item of pendingFixes) {
    await review.toggleItemApproval(item.id, 'approved');
  }
  if (pagesStore.activePageId) {
    await review.refreshPreview(pagesStore.activePageId);
  }
  appStore.notify(`Approved all ${pendingFixes.length} quick fixes!`, 'success');
};

const getShortDiffText = (item: any) => {
  const oldStr = item.original_text;
  const newStr = item.replacement_text;
  
  if (!oldStr || !newStr) return { original: oldStr, replacement: newStr };
  
  const oldWords = oldStr.split(/\s+/);
  const newWords = newStr.split(/\s+/);
  
  let start = 0;
  while (start < oldWords.length && start < newWords.length && oldWords[start] === newWords[start]) {
    start++;
  }
  
  let oldEnd = oldWords.length - 1;
  let newEnd = newWords.length - 1;
  while (oldEnd >= start && newEnd >= start && oldWords[oldEnd] === newWords[newEnd]) {
    oldEnd--;
    newEnd--;
  }
  
  const contextBefore = 1;
  const contextAfter = 1;
  
  const displayStart = Math.max(0, start - contextBefore);
  const displayOldEnd = Math.min(oldWords.length - 1, oldEnd + contextAfter);
  const displayNewEnd = Math.min(newWords.length - 1, newEnd + contextAfter);
  
  let finalOld = oldWords.slice(displayStart, displayOldEnd + 1).join(' ');
  let finalNew = newWords.slice(displayStart, displayNewEnd + 1).join(' ');
  
  if (displayStart > 0) {
    finalOld = '...' + finalOld;
    finalNew = '...' + finalNew;
  }
  if (displayOldEnd < oldWords.length - 1) {
    finalOld = finalOld + '...';
  }
  if (displayNewEnd < newWords.length - 1) {
    finalNew = finalNew + '...';
  }
  
  return {
    original: finalOld,
    replacement: finalNew
  };
};
</script>

<template>
  <div class="review-panel panel" :style="{ width: appStore.reviewPanelWidth + 'px' }">
    <!-- Panel resizer handle -->
    <div 
      :class="['panel-resizer', { 'is-dragging': isDragging }]" 
      @mousedown="initResize"
    ></div>

    <div class="panel-header" style="display: flex; justify-content: space-between; align-items: center; padding: var(--space-3) var(--space-4); gap: var(--space-2); min-height: 48px;">
      <!-- Left side: Assisting Voice Switcher & Re-submit wrapper -->
      <div style="display: flex; align-items: center; gap: 6px;">
        <div v-if="providersStore.activeProvider" class="header-model-selector" ref="dropdownEl" style="position: relative;">
          <button 
            type="button" 
            class="btn btn-xs btn-outline dropdown-trigger" 
            @click.stop="showDropdown = !showDropdown"
            :class="{ active: showDropdown }"
            style="padding: 4px 8px; font-size: 11px; display: flex; align-items: center; gap: 4px;"
          >
            <span class="model-icon">{{ selectedModelDetail?.icon || '🤖' }}</span>
            <span class="model-name" style="font-weight: 600;">{{ selectedModelDetail?.name }}</span>
            <span class="chevron-icon" style="font-size: 8px;">▼</span>
          </button>
          
          <!-- Grouped Advanced Dropdown Menu -->
          <transition name="dropdown-fade">
            <div v-if="showDropdown" class="dropdown-menu advanced-dropdown-menu" style="top: calc(100% + 6px); bottom: auto; left: 0; min-width: 240px; transform-origin: top left;">
              <div class="dropdown-header-title">Select Assisting Voice</div>
              <div class="dropdown-scroll-area">
                <div 
                  v-for="group in groupedModelsList" 
                  :key="group.providerId" 
                  class="dropdown-group"
                >
                  <div class="dropdown-group-header">
                    <span class="group-header-icon">
                      {{ group.providerId === 'openrouter' ? '🔗' : group.providerId === 'groq' ? '⚡' : '💚' }}
                    </span>
                    <span class="group-header-text">{{ group.providerName }}</span>
                  </div>
                  <button
                    v-for="model in group.models"
                    :key="model.id"
                    type="button"
                    class="dropdown-item"
                    :class="{ selected: providersStore.activeProvider?.selected_model === model.id && providersStore.activeProvider?.provider_type === group.providerId }"
                    @click="selectModel(model)"
                  >
                    <div class="item-icon-col">
                      <span class="model-icon">{{ model.icon || '🤖' }}</span>
                    </div>
                    <div class="item-text-col">
                      <div class="item-name-row">
                        <span class="item-name">{{ model.name }}</span>
                        <span v-if="providersStore.activeProvider?.selected_model === model.id && providersStore.activeProvider?.provider_type === group.providerId" class="selected-check">✓</span>
                      </div>
                      <div class="item-desc" v-if="model.use_case">{{ model.use_case }}</div>
                    </div>
                  </button>
                </div>
                
                <div v-if="groupedModelsList.length === 0" class="dropdown-empty-state" style="padding: var(--space-4) var(--space-3); text-align: center;">
                  <p class="text-xs text-muted" style="margin: 0; line-height: 1.4;">
                    No API keys configured. Set them up in settings first.
                  </p>
                </div>
              </div>
            </div>
          </transition>
        </div>

        <!-- Regenerate / Re-submit Send Button -->
        <button 
          v-if="review.groups.value.length > 0 && !review.loading.value"
          type="button" 
          class="btn btn-xs btn-outline resubmit-btn" 
          @click="handleReview"
          title="Regenerate suggestions"
          style="padding: 4px 6px; display: inline-flex; align-items: center; justify-content: center; height: 23px; width: 23px; border-radius: var(--radius-sm);"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
          </svg>
        </button>
      </div>

      <!-- Right side: Submit/Apply Changes button & Close Button -->
      <div class="flex items-center gap-2">
        <button 
          v-if="review.groups.value.length > 0"
          class="btn btn-xs btn-primary btn-submit-changes" 
          @click="handleApply" 
          :disabled="review.approvedCount() === 0"
          style="padding: 4px 10px; font-size: 11px; font-weight: 600;"
        >
          ✓ Apply {{ review.approvedCount() }} Changes
        </button>
        <button 
          class="btn btn-ghost btn-icon sm close-panel-btn" 
          @click="handleDismissReview" 
          title="Close Polishing"
          style="font-size: 16px; width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; border-radius: 50%; padding: 0; line-height: 1;"
        >
          ×
        </button>
      </div>
    </div>

    <div class="review-body">
      <!-- No review yet -->
      <div v-if="review.groups.value.length === 0 && !review.loading.value" class="review-empty">
        <template v-if="appStore.activeSidebarTab === 'speller'">
          <p class="minimal-sidebar-instruction">Choose an assisting voice at the top to begin proofreading your draft.</p>
          
          <button class="btn-polish-trigger btn-speller-trigger" @click="handleReview" :disabled="!pagesStore.activePage">
            Begin Proofreading
          </button>
        </template>

        <template v-else-if="appStore.activeSidebarTab === 'polish'">
          <p class="minimal-sidebar-instruction">Provide editorial details below to personalize the style revisions.</p>
          
          <!-- Editorial Context Card -->
          <div class="editorial-context-card">
            <label for="polish-context-textarea" class="context-label">Editorial Context & Goals</label>
            <textarea
              id="polish-context-textarea"
              v-model="reviewStore.editorialContext"
              placeholder="e.g., Target audience (developers), publication channel (blog post), desired tone (witty & clear), style guidelines, or specific depth..."
              class="context-textarea"
              rows="4"
            ></textarea>
          </div>

          <button class="btn-polish-trigger" @click="handleReview" :disabled="!pagesStore.activePage">
            Begin Polishing
          </button>
        </template>

        <template v-else>
          <p class="minimal-sidebar-instruction">Choose an assisting voice at the top to begin refining your draft.</p>
          
          <button class="btn-polish-trigger" @click="handleReview" :disabled="!pagesStore.activePage">
            Begin Polishing
          </button>
        </template>

        <p v-if="review.error.value" class="text-xs" style="color: var(--color-error); margin-top: 8px;">
          {{ review.error.value }}
        </p>
      </div>
 
      <!-- Loading -->
      <div v-else-if="review.loading.value" class="review-loading" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; padding: 24px;">
        <div class="skeleton" style="height: 60px; width: 100%; margin-bottom: 4px;" />
        <div class="skeleton" style="height: 80px; width: 100%; margin-bottom: 4px;" />
        <div class="skeleton" style="height: 60px; width: 100%;" />
        <p class="text-xs text-muted" style="text-align: center; margin-top: 4px; margin-bottom: 8px;">Analyzing your writing...</p>
        <button class="btn btn-outline" style="width: 100%; max-width: 280px; padding: 10px;" @click="review.cancelReview()">
          Cancel Polishing
        </button>
      </div>
 
      <!-- 1. SPELLER VIEW: Compact Typo Fixes Checklist -->
      <template v-else-if="appStore.activeSidebarTab === 'speller'">
        <div v-if="quickFixes.length > 0" class="speller-container">
          <div class="speller-header-row">
            <h4>Mechanical Corrections ({{ quickFixes.length }})</h4>
            <button 
              type="button"
              class="btn btn-xs btn-outline btn-accept-all-quick"
              @click.stop="approveAllQuickFixes"
            >
              ✓ Accept All
            </button>
          </div>
          
          <div class="quick-fixes-rows-list flat-checklist">
            <div 
              v-for="item in quickFixes" 
              :key="item.id"
              :class="['quick-fix-row', `state-${item.approval_state}`]"
              @click="handleApprove(item.id)"
            >
              <div class="quick-fix-info-col">
                <span class="quick-fix-pill">{{ item.categoryLabel }}</span>
                <div class="quick-fix-diff-preview">
                  <span class="diff-original">{{ getShortDiffText(item).original }}</span>
                  <span class="diff-arrow">➔</span>
                  <span class="diff-replacement">{{ getShortDiffText(item).replacement }}</span>
                </div>
              </div>
              <div class="quick-fix-status-icon">
                <span v-if="item.approval_state === 'approved'" class="tick-icon">✓</span>
                <span v-else class="bullet-icon"></span>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="review-empty" style="padding: 32px 16px;">
          <p class="minimal-sidebar-instruction">No spelling or mechanical typos found in this draft!</p>
        </div>
      </template>

      <!-- 2. POLISH VIEW: Stylistic Revisions Card List -->
      <template v-else-if="appStore.activeSidebarTab === 'polish'">
        <div v-if="stylisticGroups.length > 0" class="stylistic-revisions-section">
          <div class="corrections-header" style="margin-bottom: 12px; padding: 0 4px;">
            <h4 style="font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); font-weight: 700; margin: 0;">
              Stylistic Revisions ({{ stylisticGroups.length }})
            </h4>
          </div>

          <div class="stylistic-groups-list">
            <div 
              v-for="group in stylisticGroups" 
              :key="group.key"
              class="stylistic-card-wrapper"
            >
              <!-- Horizontal choices selector tab bar for overlapping alternatives -->
              <div v-if="group.items.length > 1" class="alternatives-tabs-bar">
                <button 
                  v-for="(alt, idx) in group.items" 
                  :key="alt.id"
                  type="button"
                  :class="['alt-tab-btn', { active: group.activeIndex === idx, approved: alt.approval_state === 'approved' }]"
                  @click="setGroupActiveIndex(group.key, idx)"
                >
                  <span class="tab-indicator-dot"></span>
                  <span class="tab-label-text">{{ alt.categoryLabel }}</span>
                  <span v-if="alt.approval_state === 'approved'" class="tab-check-tick">✓</span>
                </button>
              </div>

              <!-- Main Suggestion Card -->
              <SuggestionItemComp
                :item="group.items[group.activeIndex]"
                :categoryLabel="group.items[group.activeIndex].categoryLabel"
                :hasConflictApproval="false"
                @approve="handleApprove"
                @reject="handleReject"
              />
            </div>
          </div>
        </div>
        <div v-else class="review-empty" style="padding: 32px 16px;">
          <p class="minimal-sidebar-instruction">All flow improvements and style edits resolved!</p>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.review-panel {
  width: var(--panel-width);
  border-left: 1px solid var(--border-subtle);
  border-right: none;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  position: relative;
}
.review-body { flex: 1; overflow-y: auto; padding: var(--space-3); display: flex; flex-direction: column; gap: var(--space-3); }

.review-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-4);
  text-align: center;
  padding: var(--space-6) var(--space-4);
  background: transparent;
}
.minimal-sidebar-instruction {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  max-width: 240px;
  line-height: 1.5;
  letter-spacing: -0.01em;
  margin: 0 0 var(--space-2) 0;
}

.btn-polish-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 10px 12px;
  background: var(--accent-primary);
  color: #ffffff;
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 600;
  border-radius: var(--radius-md);
  border: 1px solid rgba(255, 255, 255, 0.05);
  box-shadow: 0 2px 8px -2px rgba(139, 92, 246, 0.25);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  margin-top: var(--space-3);
  width: 100%;
  max-width: 280px;
  align-self: center;
}

.btn-polish-trigger:hover:not(:disabled) {
  background: #7c3aed;
  box-shadow: 0 4px 12px -3px rgba(139, 92, 246, 0.35);
  transform: translateY(-1px);
}

.btn-polish-trigger:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: 0 2px 6px -2px rgba(139, 92, 246, 0.2);
}

.btn-polish-trigger:disabled {
  opacity: 0.35;
  cursor: not-allowed;
  background: var(--bg-tertiary);
  border-color: var(--border-subtle);
  color: var(--text-muted);
  box-shadow: none;
}

.review-loading { padding: var(--space-4); }

.suggestions-flat-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: var(--space-4);
}

.corrections-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
  padding: 0 4px;
}

.corrections-header h4 {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin: 0;
}

.corrections-header .badge {
  font-size: 10px;
  font-weight: bold;
  padding: 1px 5px;
  background: var(--bg-hover);
  border-radius: 8px;
  color: var(--text-secondary);
}

/* Model Selector Custom Dropdown styles */
.model-selector-container {
  width: 100%;
  max-width: 280px;
  margin: var(--space-1) 0 var(--space-3) 0;
  text-align: left;
}

.selector-label {
  text-align: left;
  font-size: 10px;
  letter-spacing: 0.08em;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.custom-dropdown {
  position: relative;
  width: 100%;
}

.dropdown-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
  box-shadow: var(--shadow-sm);
}

.dropdown-trigger:hover {
  border-color: var(--accent-border);
  background: var(--bg-hover);
}

.dropdown-trigger.active {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.trigger-content {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.model-icon {
  font-size: 14px;
}

.model-name {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
}

.chevron-icon {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  transition: transform var(--transition-fast);
}

.dropdown-trigger.active .chevron-icon {
  transform: rotate(180deg);
}

.dropdown-menu {
  position: absolute;
  bottom: calc(100% + 8px); /* Open upwards inside the empty state box */
  left: 0;
  width: 100%;
  min-width: 280px;
  background: var(--bg-dropdown);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  z-index: 100;
  overflow: hidden;
  padding: 6px;
}

.dropdown-header-title {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-subtle);
  font-weight: 600;
}

.dropdown-scroll-area {
  max-height: 240px;
  overflow-y: auto;
  padding-top: 4px;
}

.dropdown-item {
  width: 100%;
  display: flex;
  gap: var(--space-2);
  padding: 8px 10px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
  font-family: var(--font-sans);
}

.dropdown-item:hover {
  background: var(--bg-hover);
}

.dropdown-item.selected {
  background: var(--accent-subtle);
}

.dropdown-group {
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--border-subtle);
  padding-bottom: var(--space-2);
  margin-bottom: var(--space-2);
}
.dropdown-group:last-child {
  border-bottom: none;
  padding-bottom: 0;
  margin-bottom: 0;
}
.dropdown-group-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3) var(--space-1) var(--space-3);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--accent-muted);
  letter-spacing: 0.05em;
}
.group-header-icon {
  font-size: 12px;
}
.group-header-text {
  opacity: 0.85;
}

.item-icon-col {
  display: flex;
  align-items: flex-start;
  margin-top: 2px;
}

.item-text-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-name-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.item-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.selected-check {
  font-size: 12px;
  color: var(--accent-primary);
  font-weight: bold;
}

.item-desc {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.3;
}

/* Transitions */
.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

/* Re-submit Button Premium Styles */
.resubmit-btn {
  background: var(--bg-secondary);
  border-color: var(--border-subtle);
  color: var(--text-muted);
  transition: all var(--transition-fast);
}

.resubmit-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-strong);
  color: var(--text-primary);
}

.resubmit-btn svg {
  transition: transform var(--transition-fast);
}

.resubmit-btn:hover svg {
  transform: translate(1px, -1px) scale(1.05);
}

/* Quick Fixes Section Styles */
.quick-fixes-section {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  margin-bottom: var(--space-2);
}

.quick-fixes-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg-tertiary);
  cursor: pointer;
  user-select: none;
  border-bottom: 1px solid var(--border-subtle);
}

.header-left-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chevron-indicator {
  font-size: 8px;
  color: var(--text-muted);
  transition: transform var(--transition-fast);
}

.chevron-indicator.collapsed {
  transform: rotate(-90deg);
}

.header-left-title h4 {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  margin: 0;
}

.quick-fixes-count-badge {
  font-size: 9px;
  font-weight: 700;
  background: var(--bg-primary);
  color: var(--text-muted);
  padding: 1px 5px;
  border-radius: 8px;
  border: 1px solid var(--border-subtle);
}

.btn-accept-all-quick {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  color: #10b981;
  border-color: rgba(16, 185, 129, 0.35);
  background: rgba(16, 185, 129, 0.04);
}

.btn-accept-all-quick:hover {
  background: rgba(16, 185, 129, 0.08);
  border-color: rgba(16, 185, 129, 0.5);
  color: #10b981;
}

html[data-theme="light"] .btn-accept-all-quick {
  color: #059669;
  border-color: rgba(5, 150, 105, 0.3);
  background: rgba(5, 150, 105, 0.04);
}

html[data-theme="light"] .btn-accept-all-quick:hover {
  background: rgba(5, 150, 105, 0.08);
  border-color: rgba(5, 150, 105, 0.45);
}

.quick-fixes-rows-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow-y: auto;
}

.quick-fix-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border-subtle);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: var(--bg-secondary);
}

.quick-fix-row:last-child {
  border-bottom: none;
}

.quick-fix-row:hover {
  background: var(--bg-hover);
}

.quick-fix-row.state-approved {
  background: rgba(16, 185, 129, 0.02);
}

.quick-fix-row.state-approved:hover {
  background: rgba(16, 185, 129, 0.04);
}

.quick-fix-info-col {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  flex: 1;
  min-width: 0;
  padding: 2px 0;
}

.quick-fix-pill {
  font-size: 8px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: var(--radius-xs);
  letter-spacing: 0.02em;
  flex-shrink: 0;
  margin-top: 1.5px;
}

.quick-fix-diff-preview {
  display: flex;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 4px 6px;
  font-size: 11px;
  color: var(--text-primary);
  line-height: 1.4;
  word-break: break-word;
}

.diff-original {
  color: var(--color-error);
  text-decoration: line-through;
  opacity: 0.8;
}

.diff-arrow {
  color: var(--text-muted);
  font-size: 9px;
  opacity: 0.6;
}

.diff-replacement {
  color: var(--color-success);
  font-weight: 600;
}

.quick-fix-status-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.tick-icon {
  color: #10b981;
  font-weight: 800;
  font-size: 11px;
}

html[data-theme="light"] .tick-icon {
  color: #059669;
}

.bullet-icon {
  width: 5px;
  height: 5px;
  background: var(--border-default);
  border-radius: 50%;
  opacity: 0.5;
}

.quick-fix-row:hover .bullet-icon {
  opacity: 0.8;
  background: var(--accent-primary);
}

.sections-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: var(--space-2) 0;
  opacity: 0.5;
}

/* Stylistic Revisions Card Layout Styles */
.stylistic-card-wrapper {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  margin-bottom: 12px;
}

.stylistic-card-wrapper :deep(.suggestion-card-minimal) {
  border: none;
  box-shadow: none;
  border-radius: 0;
  background: transparent;
}

.alternatives-tabs-bar {
  display: flex;
  gap: 3px;
  background: var(--bg-tertiary);
  padding: 6px 10px;
  border-bottom: 1px solid var(--border-subtle);
  overflow-x: auto;
  scrollbar-width: none;
}

.alternatives-tabs-bar::-webkit-scrollbar {
  display: none;
}

.alt-tab-btn {
  background: transparent;
  border: none;
  font-family: var(--font-sans);
  font-size: 8.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  padding: 4px 8px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.alt-tab-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.alt-tab-btn.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.tab-indicator-dot {
  width: 4px;
  height: 4px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: all var(--transition-fast);
}

.alt-tab-btn.active .tab-indicator-dot {
  background: var(--accent-primary);
}

.alt-tab-btn.approved {
  color: #10b981;
}

html[data-theme="light"] .alt-tab-btn.approved {
  color: #059669;
}

.alt-tab-btn.approved .tab-indicator-dot {
  background: #10b981;
}

html[data-theme="light"] .alt-tab-btn.approved .tab-indicator-dot {
  background: #059669;
}

.tab-check-tick {
  font-size: 8px;
  font-weight: 800;
}

/* Animations */
.collapse-enter-active,
.collapse-leave-active {
  transition: max-height 0.2s ease-in-out, opacity 0.2s ease-in-out;
  max-height: 240px;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  max-height: 0;
  opacity: 0;
}

/* Speller View Custom Styles */
.speller-container {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  width: 100%;
}

.speller-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
}

.speller-header-row h4 {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  margin: 0;
}

.flat-checklist {
  max-height: none !important; /* Speller has no max height limit, takes full panel scroll! */
}

/* Editorial Context UI Premium Styles */
.editorial-context-card {
  width: 100%;
  max-width: 280px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  box-shadow: var(--shadow-sm);
  margin-top: 4px;
  margin-bottom: 2px;
  text-align: left;
  transition: all 0.2s ease;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.editorial-context-card:focus-within {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.context-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.context-textarea {
  width: 100%;
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  padding: 8px;
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  line-height: 1.4;
  resize: vertical;
  min-height: 80px;
  transition: border-color var(--transition-fast);
}

.context-textarea:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.context-textarea::placeholder {
  color: var(--text-muted);
  opacity: 0.6;
}

/* Proofreader Tab Specific Colors */
.btn-speller-trigger {
  background: #10b981 !important;
  box-shadow: 0 2px 8px -2px rgba(16, 185, 129, 0.25) !important;
}

.btn-speller-trigger:hover:not(:disabled) {
  background: #059669 !important;
  box-shadow: 0 4px 12px -3px rgba(16, 185, 129, 0.35) !important;
}
</style>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { usePagesStore } from '../stores/pages';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { useReview } from '../composables/useReview';
import { Store } from '@tauri-apps/plugin-store';
import SuggestionItemComp from './SuggestionItem.vue';
import { categoryLabels } from '../types';

const pagesStore = usePagesStore();
const providersStore = useProvidersStore();
const appStore = useAppStore();
const review = useReview();

const isDragging = ref(false);

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

interface ModelDetail {
  id: string;
  name: string;
  useCase: string;
  icon: string;
}

const openrouterModelsList: ModelDetail[] = [
  { id: 'openai/gpt-4o-mini', name: 'GPT-4o Mini', useCase: 'Fast proofreading, grammar & syntax polishing', icon: '⚡' },
  { id: 'meta-llama/llama-3.3-70b-instruct', name: 'Llama 3.3 70B', useCase: 'General prose editing & standard revision', icon: '🦙' },
  { id: 'z-ai/glm-4.5-air', name: 'GLM 4.5 Air', useCase: 'Creative novels, character dialogue & SEO', icon: '🎨' },
  { id: 'openrouter/owl-alpha', name: 'Owl Alpha', useCase: 'Heavy content synthesis & professional drafts', icon: '🦉' },
  { id: 'google/gemma-4-31b-it', name: 'Gemma 4 31B', useCase: 'Grammar review & high-accuracy translation', icon: '💎' },
  { id: 'openai/gpt-oss-120b', name: 'GPT-OSS 120B', useCase: 'Academic papers, logic flow & structured essays', icon: '🎓' },
  { id: 'deepseek/deepseek-v4-flash', name: 'DeepSeek V4 Flash', useCase: 'Long-form narrative & fast token translation', icon: '🌀' }
];

const groqModelsList: ModelDetail[] = [
  { id: 'openai/gpt-oss-120b', name: 'GPT-OSS 120B', useCase: 'Academic papers, logic flow & structured essays', icon: '🎓' },
  { id: 'llama-3.1-8b-instant', name: 'Llama 3.1 8B Instant', useCase: 'Ultra-fast grammar corrections & simple revisions', icon: '⚡' },
  { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B Versatile', useCase: 'General prose editing & standard revision', icon: '🦙' },
  { id: 'meta-llama/llama-4-scout-17b-16e-instruct', name: 'Llama 4 Scout 17B (16e)', useCase: 'Highly creative, nuance & character dialog', icon: '🔍' }
];

const modelsList = computed(() => {
  const active = providersStore.activeProvider;
  if (active && active.provider_type === 'groq') {
    return groqModelsList;
  }
  return openrouterModelsList;
});

const selectedModelDetail = computed(() => {
  const active = providersStore.activeProvider;
  if (!active) return null;
  return modelsList.value.find(m => m.id === active.selected_model) || {
    id: active.selected_model,
    name: active.selected_model.split('/').pop() || active.selected_model,
    useCase: 'Custom Model',
    icon: '🤖'
  };
});

const selectModel = async (modelId: string) => {
  const active = providersStore.activeProvider;
  if (!active) return;
  try {
    await providersStore.saveProvider({
      id: active.id,
      name: active.name,
      providerType: active.provider_type,
      endpoint: active.endpoint || undefined,
      selectedModel: modelId,
      enabled: active.enabled
    });
    appStore.notify(`Switched to ${selectedModelDetail.value?.name}`, 'success');
  } catch (e) {
    appStore.notify('Failed to switch model', 'error');
  }
  showDropdown.value = false;
};

const closeDropdown = (e: MouseEvent) => {
  if (dropdownEl.value && !dropdownEl.value.contains(e.target as Node)) {
    showDropdown.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', closeDropdown);
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
    await review.doReview(page.id, apiKey, provider.selected_model, provider.endpoint || undefined);
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

// Flattened list of suggestions with injected category label
const allSuggestions = computed(() => {
  return review.groups.value.flatMap(gwi => {
    const categoryLabel = gwi.group.label || categoryLabels[gwi.group.category] || gwi.group.category;
    return gwi.items.map(item => ({
      ...item,
      categoryLabel
    }));
  });
});
</script>

<template>
  <div class="review-panel panel" :style="{ width: appStore.reviewPanelWidth + 'px' }">
    <!-- Panel resizer handle -->
    <div 
      :class="['panel-resizer', { 'is-dragging': isDragging }]" 
      @mousedown="initResize"
    ></div>
    <div class="panel-header">
      <h3>Polishing Desk</h3>
      <div class="flex items-center gap-2" v-if="review.groups.value.length > 0">
        <span class="badge badge-neutral">{{ review.totalCount() }} suggestions</span>
      </div>
    </div>

    <div class="review-body">
      <!-- No review yet -->
      <div v-if="review.groups.value.length === 0 && !review.loading.value" class="review-empty">
        <p class="minimal-sidebar-instruction">Choose an assisting voice below to begin refining your draft.</p>
        
        <!-- Premium Model Selector -->
        <div v-if="providersStore.activeProvider" class="model-selector-container" ref="dropdownEl">
          <label class="label selector-label">Assisting Voice</label>
          <div class="custom-dropdown">
            <button 
              type="button" 
              class="dropdown-trigger" 
              @click.stop="showDropdown = !showDropdown"
              :class="{ active: showDropdown }"
            >
              <div class="trigger-content" v-if="selectedModelDetail">
                <span class="model-icon">{{ selectedModelDetail.icon }}</span>
                <span class="model-name">{{ selectedModelDetail.name }}</span>
              </div>
              <span class="chevron-icon">▾</span>
            </button>
            
            <transition name="dropdown-fade">
              <div v-if="showDropdown" class="dropdown-menu">
                <div class="dropdown-header-title">Select Assisting Voice</div>
                <div class="dropdown-scroll-area">
                  <button
                    v-for="model in modelsList"
                    :key="model.id"
                    type="button"
                    class="dropdown-item"
                    :class="{ selected: providersStore.activeProvider.selected_model === model.id }"
                    @click="selectModel(model.id)"
                  >
                    <div class="item-icon-col">
                      <span class="model-icon">{{ model.icon }}</span>
                    </div>
                    <div class="item-text-col">
                      <div class="item-name-row">
                        <span class="item-name">{{ model.name }}</span>
                        <span v-if="providersStore.activeProvider.selected_model === model.id" class="selected-check">✓</span>
                      </div>
                      <div class="item-desc">{{ model.useCase }}</div>
                    </div>
                  </button>
                </div>
              </div>
            </transition>
          </div>
        </div>

        <button class="btn-polish-trigger" @click="handleReview" :disabled="!pagesStore.activePage">
          Begin Polishing
        </button>
        <p v-if="review.error.value" class="text-xs" style="color: var(--color-error); margin-top: 8px;">
          {{ review.error.value }}
        </p>
      </div>

      <!-- Loading -->
      <div v-else-if="review.loading.value" class="review-loading">
        <div class="skeleton" style="height: 60px; margin-bottom: 8px;" />
        <div class="skeleton" style="height: 80px; margin-bottom: 8px;" />
        <div class="skeleton" style="height: 60px;" />
        <p class="text-xs text-muted" style="text-align: center; margin-top: 12px;">Analyzing your writing...</p>
      </div>

      <!-- Suggestions Flat List -->
      <template v-else>
        <div class="corrections-header">
          <h4>Corrections</h4>
          <span class="badge badge-neutral">{{ review.totalCount() }}</span>
        </div>

        <div class="suggestions-flat-list">
          <SuggestionItemComp
            v-for="item in allSuggestions"
            :key="item.id"
            :item="item"
            :categoryLabel="item.categoryLabel"
            @approve="handleApprove"
            @reject="handleReject"
          />
        </div>

        <!-- Apply Button -->
        <div class="review-footer">
          <button class="btn btn-ghost" @click="handleDismissReview">Dismiss</button>
          <button class="btn btn-primary" @click="handleApply" :disabled="review.approvedCount() === 0">
            Apply {{ review.approvedCount() }} Changes
          </button>
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

.review-footer {
  padding: var(--space-3);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  justify-content: space-between;
  gap: var(--space-2);
  flex-shrink: 0;
  background: var(--bg-secondary);
  position: sticky;
  bottom: 0;
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
</style>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { VersionSnapshot, CompareResult } from '../types';
import { compareTexts } from '../composables/useTauri';

const props = defineProps<{
  version: VersionSnapshot;
  activePageContent: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'restore', version: VersionSnapshot): void;
}>();

const activeTab = ref<'diff' | 'full'>('diff');
const diffResult = ref<CompareResult | null>(null);
const loadingDiff = ref(false);

const convertHtmlToText = (html: string) => {
  if (!html) return '';
  const tempDiv = document.createElement('div');
  tempDiv.innerHTML = html;
  
  // Replace block elements with newlines to preserve spacing
  const blockTags = ['p', 'div', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'li', 'br'];
  blockTags.forEach(tag => {
    const elements = tempDiv.getElementsByTagName(tag);
    for (let i = elements.length - 1; i >= 0; i--) {
      const el = elements[i];
      const newline = document.createTextNode('\n');
      el.parentNode?.insertBefore(newline, el.nextSibling);
    }
  });
  
  return tempDiv.textContent || tempDiv.innerText || '';
};

const loadDiff = async () => {
  loadingDiff.value = true;
  try {
    const cleanActive = convertHtmlToText(props.activePageContent);
    const cleanVersion = convertHtmlToText(props.version.content);
    diffResult.value = await compareTexts(cleanActive, cleanVersion);
  } catch (e) {
    console.error('Failed to calculate diff:', e);
  } finally {
    loadingDiff.value = false;
  }
};

onMounted(() => {
  loadDiff();
});

const formatDate = (dateStr: string) => {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' }) + ', ' + 
           d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: true });
  } catch { return dateStr; }
};
</script>

<template>
  <div class="modal-backdrop" @click="emit('close')">
    <div class="modal-content version-preview-modal" @click.stop>
      <div class="modal-header">
        <div class="modal-title-area">
          <span class="badge badge-pending text-xs" style="margin-bottom: var(--space-1); width: fit-content;">
            Version Snapshot
          </span>
          <h2 class="version-title">
            {{ version.name || 'Auto-created Draft' }}
          </h2>
          <p class="text-xs text-secondary">
            Created on {{ formatDate(version.created_at) }}
          </p>
        </div>
        <button class="btn btn-ghost btn-icon close-btn" @click="emit('close')">×</button>
      </div>

      <div class="modal-tabs">
        <button 
          :class="['tab-btn', { active: activeTab === 'diff' }]" 
          @click="activeTab = 'diff'"
        >
          Unified Diff
        </button>
        <button 
          :class="['tab-btn', { active: activeTab === 'full' }]" 
          @click="activeTab = 'full'"
        >
          Full Snapshot Content
        </button>
      </div>

      <div class="modal-body">
        <div v-if="activeTab === 'diff'" class="diff-container">
          <div v-if="loadingDiff" class="loading-state">
            <div class="spinner"></div>
            <span>Calculating differences...</span>
          </div>
          <div v-else-if="diffResult" class="diff-view card">
            <div class="diff-content">
              <template v-for="(seg, i) in diffResult.diff_segments" :key="i">
                <span v-if="seg.tag === 'equal'">{{ seg.old_text }}</span>
                <span v-else-if="seg.tag === 'delete'" class="diff-delete">{{ seg.old_text }}</span>
                <span v-else-if="seg.tag === 'insert'" class="diff-insert">{{ seg.new_text }}</span>
              </template>
            </div>
          </div>
          <div v-else class="error-state">
            <span>Unable to render diff. Please view the full snapshot text instead.</span>
          </div>
        </div>

        <div v-else class="full-content-container">
          <div class="full-content-rendered">
            <div class="tiptap prose-preview" v-html="version.content || '<p>(Empty snapshot)</p>'"></div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <div class="footer-meta">
          <span class="badge badge-neutral text-xs">
            {{ version.applied_suggestion_ids.length }} suggestions applied
          </span>
        </div>
        <div class="footer-actions">
          <button class="btn btn-outline" @click="emit('close')">Close</button>
          <button class="btn btn-primary" @click="emit('restore', version)">
            Restore Version
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(8, 9, 12, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn var(--transition-fast);
}

.modal-content {
  background: var(--bg-primary);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  width: 80vw;
  max-width: 850px;
  height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
  animation: scaleIn var(--transition-fast);
}

.modal-header {
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  background: var(--bg-secondary);
}

.modal-title-area {
  display: flex;
  flex-direction: column;
}

.version-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
  margin: 2px 0;
}

.close-btn {
  font-size: 20px;
  color: var(--text-muted);
  width: var(--space-8);
  height: var(--space-8);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: none;
}
.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.modal-tabs {
  display: flex;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  padding: 0 var(--space-6);
  gap: var(--space-4);
}

.tab-btn {
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  padding: var(--space-3) 2px;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.tab-btn:hover {
  color: var(--text-primary);
}
.tab-btn.active {
  color: var(--accent-primary);
  border-bottom-color: var(--accent-primary);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  background: var(--bg-primary);
}

.diff-container, .full-content-container {
  height: 100%;
}

.diff-view {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  height: 100%;
  overflow-y: auto;
  padding: var(--space-4);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

.diff-content {
  white-space: pre-wrap;
  word-break: break-all;
  line-height: var(--line-height-relaxed);
}

.full-content-rendered {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  height: 100%;
  overflow-y: auto;
  padding: var(--space-4);
}

.prose-preview {
  color: var(--text-primary);
  max-width: 100%;
  padding: 0;
  min-height: auto;
  margin: 0;
}

.loading-state, .error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-subtle);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.modal-footer {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-actions {
  display: flex;
  gap: var(--space-2);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleIn {
  from { transform: scale(0.97); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>

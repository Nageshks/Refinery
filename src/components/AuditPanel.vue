<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useAppStore } from '../stores/app';
import { usePagesStore } from '../stores/pages';
import { useProvidersStore } from '../stores/providers';
import { useAuditStore } from '../stores/audit';
import type { AuditResult } from '../stores/audit';
import { Store } from '@tauri-apps/plugin-store';

const appStore = useAppStore();
const pagesStore = usePagesStore();
const providersStore = useProvidersStore();
const auditStore = useAuditStore();

// ─── Panel State ────────────────────────────────────────────────────────

const panelWidth = ref(380);
const activeResult = ref<AuditResult | null>(null);

// ─── Computed ───────────────────────────────────────────────────────────

const activePage = computed(() => pagesStore.activePage);
const pageFormatType = computed(() => activePage.value?.format_type || 'markdown');
const hasPageType = computed(() => pageFormatType.value !== 'markdown' && pageFormatType.value !== '');
const assignedPageType = computed(() => {
  if (!hasPageType.value) return null;
  return auditStore.getPageType(pageFormatType.value);
});
const resolvedChecks = computed(() => {
  if (!assignedPageType.value) return [];
  return auditStore.getResolvedChecks(assignedPageType.value.id);
});
const availablePageTypes = computed(() => auditStore.pageTypes);

const isStale = computed(() => {
  if (!activePage.value || !activeResult.value) return false;
  return auditStore.isAuditStale(activePage.value.id, activePage.value.content || '');
});

const readinessScore = computed(() => {
  if (!activeResult.value || !activeResult.value.checks || activeResult.value.checks.length === 0) return null;

  let totalWeight = 0;
  let scoredPoints = 0;

  activeResult.value.checks.forEach(chk => {
    if (chk.widgetType === 'scale_1_5') {
      const score = Number(chk.value);
      if (!isNaN(score)) {
        scoredPoints += (score / 5) * 100;
        totalWeight += 1;
      }
    } else if (chk.widgetType === 'binary') {
      const pass = String(chk.value).trim().toLowerCase() === 'pass';
      scoredPoints += pass ? 100 : 0;
      totalWeight += 1;
    }
  });

  if (totalWeight === 0) return null;

  const percentage = Math.round(scoredPoints / totalWeight);
  
  let label = 'Draft Unready';
  let color = 'var(--color-error, #ef4444)';
  
  if (percentage >= 90) {
    label = 'Ready to Publish 🚀';
    color = 'var(--color-success, #22c55e)';
  } else if (percentage >= 70) {
    label = 'Ready with Minor Polish ✨';
    color = '#84cc16'; // Yellow green
  } else if (percentage >= 50) {
    label = 'Needs Substantial Revision ⚠️';
    color = 'var(--color-warning, #f59e0b)';
  } else {
    label = 'Unsuitable to Publish ❌';
    color = 'var(--color-error, #ef4444)';
  }

  return {
    percentage,
    label,
    color
  };
});

// ─── Load cached audit when page changes ────────────────────────────────

watch(() => activePage.value?.id, (newId) => {
  if (newId) {
    activeResult.value = auditStore.getCachedAudit(newId);
  } else {
    activeResult.value = null;
  }
}, { immediate: true });

// ─── Page Type Assignment ───────────────────────────────────────────────

const handleAssignPageType = async (typeId: string) => {
  if (!activePage.value) return;
  try {
    if (typeId === '__none__') {
      await pagesStore.updatePageFormat(activePage.value.id, 'markdown');
      activeResult.value = null;
      auditStore.clearCachedAudit(activePage.value.id);
      appStore.notify('Page type removed', 'success');
    } else {
      await pagesStore.updatePageFormat(activePage.value.id, typeId);
      activeResult.value = null;
      auditStore.clearCachedAudit(activePage.value.id);
      const pt = auditStore.getPageType(typeId);
      appStore.notify(`Page type set to "${pt?.name || typeId}"`, 'success');
    }
  } catch (e) {
    appStore.notify('Failed to update page type', 'error');
  }
};

// ─── Run Audit ──────────────────────────────────────────────────────────

const handleRunAudit = async () => {
  if (!activePage.value || !assignedPageType.value) return;

  try {
    // Load API key from active provider
    const providers = providersStore.providers;
    const activeProvider = providers.find(p => p.enabled);
    if (!activeProvider) {
      appStore.notify('No active AI provider configured. Open Settings → AI Providers.', 'error');
      return;
    }

    const store = await Store.load('credentials.json');
    const apiKey = await store.get<string>(`apikey_${activeProvider.id}`);
    if (!apiKey) {
      appStore.notify('No API key found for active provider. Open Settings → AI Providers.', 'error');
      return;
    }

    const result = await auditStore.runAudit(
      activePage.value.id,
      assignedPageType.value.id,
      activePage.value.content || '',
      apiKey,
      activeProvider.selected_model,
      activeProvider.endpoint || undefined
    );

    activeResult.value = result;

    if (result.rawFallback) {
      appStore.notify('Analysis complete (raw text mode — AI response format issue)', 'info');
    } else {
      appStore.notify('Draft analysis complete', 'success');
    }
  } catch (err: any) {
    appStore.notify(err.message || 'Audit failed', 'error');
  }
};

// ─── Score Color Helper ─────────────────────────────────────────────────

const getScoreColor = (score: number): string => {
  if (score >= 5) return 'var(--color-success, #22c55e)';
  if (score >= 4) return '#84cc16';
  if (score >= 3) return 'var(--color-warning, #f59e0b)';
  if (score >= 2) return '#f97316';
  return 'var(--color-error, #ef4444)';
};

const getBinaryColor = (value: string): string => {
  return value === 'pass' ? 'var(--color-success, #22c55e)' : 'var(--color-error, #ef4444)';
};

// ─── Resize ─────────────────────────────────────────────────────────────

const initResize = (e: MouseEvent) => {
  e.preventDefault();
  const startX = e.clientX;
  const startWidth = panelWidth.value;
  const onMove = (ev: MouseEvent) => {
    const delta = startX - ev.clientX;
    panelWidth.value = Math.min(600, Math.max(300, startWidth + delta));
  };
  const onUp = () => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
};
</script>

<template>
  <div class="audit-panel panel" :style="{ width: panelWidth + 'px' }">
    <div class="panel-resizer" @mousedown="initResize"></div>

    <!-- Header -->
    <div class="audit-header">
      <div class="audit-header-top">
        <h3 class="audit-title">📋 Draft Auditor</h3>
        <button class="close-audit-btn" @click="appStore.auditPanelVisible = false" title="Close">×</button>
      </div>

      <!-- Page Type Selector -->
      <div class="audit-type-selector">
        <label class="selector-label">Page Type</label>
        <select
          class="type-dropdown"
          :value="hasPageType ? pageFormatType : '__none__'"
          @change="handleAssignPageType(($event.target as HTMLSelectElement).value)"
        >
          <option value="__none__">None (No Audit)</option>
          <option
            v-for="pt in availablePageTypes"
            :key="pt.id"
            :value="pt.id"
          >
            {{ pt.icon }} {{ pt.name }}
          </option>
        </select>
      </div>
    </div>

    <!-- Body -->
    <div class="audit-body">

      <!-- No Page Type Assigned -->
      <div v-if="!hasPageType" class="audit-empty">
        <span class="empty-icon">📝</span>
        <p class="empty-text">No Page Type Assigned</p>
        <p class="empty-subtext">
          Select a page type above to enable structured AI audits for this draft.
        </p>
        <p class="empty-subtext" v-if="availablePageTypes.length === 0">
          No page types configured yet. Go to <strong>Settings → Page Types</strong> to create one.
        </p>
      </div>

      <!-- Page Type Assigned but No Page Types Configured -->
      <div v-else-if="!assignedPageType" class="audit-empty">
        <span class="empty-icon">⚠️</span>
        <p class="empty-text">Page Type Not Found</p>
        <p class="empty-subtext">
          The assigned page type "{{ pageFormatType }}" was not found. It may have been deleted. Please select a different type.
        </p>
      </div>

      <!-- Page Type Assigned — Ready to Audit -->
      <template v-else>

        <!-- Stale Warning -->
        <div v-if="isStale && activeResult" class="stale-warning">
          <span>⚠️</span>
          <span>Draft modified since last analysis. Re-run to update.</span>
        </div>

        <!-- Checks Preview (before running) -->
        <div v-if="!activeResult && !auditStore.auditing" class="checks-preview">
          <div class="checks-preview-header">
            <span class="preview-badge">{{ resolvedChecks.length }}</span>
            <span>Active Checks</span>
          </div>
          <div class="checks-preview-list">
            <div v-for="check in resolvedChecks" :key="check.id" class="preview-check-item">
              <span class="preview-check-icon">
                {{ check.widgetType === 'binary' ? '✓' : check.widgetType === 'scale_1_5' ? '📊' : '📝' }}
              </span>
              <span class="preview-check-name">{{ check.name }}</span>
              <span class="preview-check-type">{{ check.widgetType === 'binary' ? 'Pass/Fail' : check.widgetType === 'scale_1_5' ? '1–5' : 'Extract' }}</span>
            </div>
          </div>
        </div>

        <!-- Loading State -->
        <div v-if="auditStore.auditing" class="audit-loading">
          <div class="skeleton-pulse"></div>
          <div class="skeleton-pulse short"></div>
          <div class="skeleton-pulse"></div>
          <p class="loading-text">Analyzing draft against {{ resolvedChecks.length }} parameters...</p>
        </div>

        <!-- Error State -->
        <div v-if="auditStore.auditError && !auditStore.auditing" class="audit-error">
          <span class="error-icon">❌</span>
          <p class="error-text">{{ auditStore.auditError }}</p>
          <button class="btn-retry" @click="handleRunAudit">🔄 Retry Analysis</button>
        </div>

        <!-- Results -->
        <div v-if="activeResult && !auditStore.auditing" class="audit-results">

          <!-- Aggregate Readiness Score Card -->
          <div v-if="readinessScore" class="readiness-score-card">
            <div class="readiness-score-header flex-between">
              <span class="readiness-score-title">🎯 Draft Readiness Score</span>
              <span class="readiness-score-badge" :style="{ color: readinessScore.color, border: `1px solid ${readinessScore.color}`, background: readinessScore.color + '0e' }">
                {{ readinessScore.label }}
              </span>
            </div>
            
            <div class="readiness-score-body flex items-center gap-4">
              <div class="gauge-percentage" :style="{ color: readinessScore.color }">
                {{ readinessScore.percentage }}%
              </div>
              
              <div class="flex-grow progress-track-layout">
                <div class="progress-bar-container">
                  <div class="progress-bar-track">
                    <div 
                      class="progress-bar-fill" 
                      :style="{ width: readinessScore.percentage + '%', background: readinessScore.color }"
                    ></div>
                  </div>
                </div>
                <span class="score-helper-text">
                  Average score evaluated across all criteria.
                </span>
              </div>
            </div>
          </div>

          <!-- Raw Fallback (malformed JSON) -->
          <div v-if="activeResult.rawFallback" class="fallback-card">
            <div class="fallback-header">
              <span>⚠️</span>
              <span>Raw Feedback (format issue)</span>
            </div>
            <pre class="fallback-text">{{ activeResult.rawFallback }}</pre>
            <button class="btn-retry" @click="handleRunAudit">🔄 Retry Analysis</button>
          </div>

          <!-- Structured Results -->
          <template v-else>
            <div
              v-for="check in activeResult.checks"
              :key="check.checkId"
              class="audit-check-card"
            >
              <!-- Binary Widget -->
              <template v-if="check.widgetType === 'binary'">
                <div class="check-header">
                  <span class="check-name">{{ check.name }}</span>
                  <span
                    class="binary-badge"
                    :style="{ background: getBinaryColor(String(check.value)), color: '#fff' }"
                  >
                    {{ String(check.value).toUpperCase() }}
                  </span>
                </div>
                <p class="check-feedback">{{ check.feedback }}</p>
                <p v-if="check.suggestion" class="check-suggestion">💡 {{ check.suggestion }}</p>
              </template>

              <!-- Scale 1-5 Widget -->
              <template v-else-if="check.widgetType === 'scale_1_5'">
                <div class="check-header">
                  <span class="check-name">{{ check.name }}</span>
                  <span class="scale-value" :style="{ color: getScoreColor(Number(check.value)) }">
                    {{ check.value }}/5
                  </span>
                </div>
                <div class="scale-bar-container">
                  <div class="scale-bar-track">
                    <div
                      class="scale-bar-fill"
                      :style="{ width: (Number(check.value) / 5) * 100 + '%', background: getScoreColor(Number(check.value)) }"
                    ></div>
                  </div>
                  <div class="scale-segments">
                    <span v-for="n in 5" :key="n" class="scale-segment" :class="{ active: n <= Number(check.value) }"></span>
                  </div>
                </div>
                <p class="check-feedback">{{ check.feedback }}</p>
                <p v-if="check.suggestion" class="check-suggestion">💡 {{ check.suggestion }}</p>
              </template>

              <!-- Text Match Widget -->
              <template v-else-if="check.widgetType === 'text_match'">
                <div class="check-header">
                  <span class="check-name">{{ check.name }}</span>
                  <span class="text-match-badge">📝 Extracted</span>
                </div>
                <div class="text-match-extraction">
                  <blockquote class="extracted-text">"{{ check.value }}"</blockquote>
                </div>
                <p v-if="check.evaluation" class="check-evaluation">{{ check.evaluation }}</p>
                <p class="check-feedback">{{ check.feedback }}</p>
                <p v-if="check.suggestion" class="check-suggestion">💡 {{ check.suggestion }}</p>
              </template>

              <!-- Snippet link -->
              <div v-if="check.snippet" class="check-snippet">
                <span class="snippet-label">📍 Related text:</span>
                <span class="snippet-text">"{{ check.snippet.substring(0, 80) }}{{ check.snippet.length > 80 ? '…' : '' }}"</span>
              </div>
            </div>
          </template>
        </div>

        <!-- Action Buttons -->
        <div v-if="assignedPageType && !auditStore.auditing" class="audit-actions">
          <button
            class="btn-audit-primary"
            @click="handleRunAudit"
            :disabled="auditStore.auditing"
          >
            ⚡ {{ activeResult ? 'Re-Analyze Draft' : 'Analyze Draft' }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.audit-panel {
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  flex-shrink: 0;
  position: relative;
  height: 100%;
  overflow: hidden;
}

.panel-resizer {
  position: absolute;
  left: -3px;
  top: 0;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
  transition: background var(--transition-fast);
}
.panel-resizer:hover {
  background: var(--accent-primary);
}

/* ─── Header ─── */
.audit-header {
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.audit-header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.audit-title {
  font-size: var(--font-size-sm);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.01em;
}
.close-audit-btn {
  font-size: 20px;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  color: var(--text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
  line-height: 1;
}
.close-audit-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* ─── Type Selector ─── */
.audit-type-selector {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.selector-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.type-dropdown {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}
.type-dropdown:focus {
  border-color: var(--accent-primary);
  outline: none;
  box-shadow: 0 0 0 3px var(--accent-subtle);
}
.type-dropdown option {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

/* ─── Body ─── */
.audit-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4) var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ─── Empty State ─── */
.audit-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
  gap: var(--space-2);
  text-align: center;
  flex: 1;
}
.audit-empty .empty-icon { font-size: 32px; opacity: 0.6; }
.audit-empty .empty-text {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0;
}
.audit-empty .empty-subtext {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  line-height: 1.5;
  max-width: 260px;
  margin: 0;
}

/* ─── Stale Warning ─── */
.stale-warning {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 10px 14px;
  background: rgba(245, 158, 11, 0.08);
  border: 1px solid rgba(245, 158, 11, 0.2);
  border-radius: var(--radius-md);
  font-size: var(--font-size-xs);
  color: #f59e0b;
  font-weight: 500;
}

/* ─── Checks Preview ─── */
.checks-preview {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.checks-preview-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.preview-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: var(--accent-subtle);
  color: var(--accent-primary);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 700;
}
.checks-preview-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.preview-check-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
}
.preview-check-icon {
  font-size: 12px;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}
.preview-check-name {
  flex: 1;
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  font-weight: 500;
}
.preview-check-type {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  padding: 2px 6px;
  background: var(--bg-hover);
  border-radius: var(--radius-xs);
}

/* ─── Loading ─── */
.audit-loading {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4) 0;
}
.skeleton-pulse {
  height: 48px;
  background: linear-gradient(90deg, var(--bg-hover) 25%, var(--bg-tertiary) 50%, var(--bg-hover) 75%);
  background-size: 200% 100%;
  border-radius: var(--radius-md);
  animation: shimmer 1.5s infinite;
}
.skeleton-pulse.short { width: 60%; }
.loading-text {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-align: center;
  margin: 0;
}
@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* ─── Error ─── */
.audit-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-6) var(--space-4);
  text-align: center;
}
.error-icon { font-size: 28px; }
.error-text {
  font-size: var(--font-size-sm);
  color: var(--color-error, #ef4444);
  margin: 0;
}
.btn-retry {
  padding: 8px 16px;
  background: var(--bg-hover);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.btn-retry:hover {
  background: var(--bg-active);
}

/* ─── Result Cards ─── */
.audit-results {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.audit-check-card {
  padding: 14px 16px;
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  transition: border-color var(--transition-fast);
}
.audit-check-card:hover {
  border-color: var(--border-default);
}
.check-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}
.check-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.binary-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  letter-spacing: 0.05em;
}
.scale-value {
  font-size: var(--font-size-md);
  font-weight: 700;
}

/* Scale bar */
.scale-bar-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.scale-bar-track {
  width: 100%;
  height: 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  overflow: hidden;
}
.scale-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}
.scale-segments {
  display: flex;
  gap: 3px;
}
.scale-segment {
  flex: 1;
  height: 3px;
  background: var(--bg-hover);
  border-radius: 2px;
  transition: background var(--transition-fast);
}
.scale-segment.active {
  background: var(--accent-primary);
}

.text-match-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 8px;
  background: var(--accent-subtle);
  color: var(--accent-primary);
  border-radius: var(--radius-sm);
}
.text-match-extraction {
  padding: 0;
}
.extracted-text {
  margin: 0;
  padding: 10px 14px;
  background: var(--bg-tertiary);
  border-left: 3px solid var(--accent-primary);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  font-style: italic;
  line-height: 1.5;
}
.check-evaluation {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.4;
  font-weight: 500;
}
.check-feedback {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin: 0;
  line-height: 1.4;
}
.check-suggestion {
  font-size: var(--font-size-xs);
  color: var(--accent-primary);
  margin: 0;
  line-height: 1.4;
  font-weight: 500;
  padding: 6px 10px;
  background: var(--accent-subtle);
  border-radius: var(--radius-sm);
}
.check-snippet {
  display: flex;
  align-items: flex-start;
  gap: var(--space-1);
  font-size: 11px;
  color: var(--text-muted);
  padding-top: var(--space-1);
  border-top: 1px solid var(--border-subtle);
}
.snippet-label {
  flex-shrink: 0;
  font-weight: 600;
}
.snippet-text {
  font-style: italic;
  opacity: 0.8;
}

/* Fallback */
.fallback-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: 14px 16px;
  background: var(--bg-primary);
  border: 1px solid rgba(245, 158, 11, 0.2);
  border-radius: var(--radius-md);
}
.fallback-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: #f59e0b;
}
.fallback-text {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 300px;
  overflow-y: auto;
  margin: 0;
  padding: 10px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  line-height: 1.5;
}

/* ─── Actions ─── */
.audit-actions {
  padding: var(--space-4) 0 0;
  border-top: 1px solid var(--border-subtle);
}
.btn-audit-primary {
  width: 100%;
  padding: 12px 16px;
  background: var(--accent-primary);
  color: #ffffff;
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  letter-spacing: -0.01em;
}
.btn-audit-primary:hover {
  filter: brightness(1.1);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}
.btn-audit-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ─── Premium Aggregate Readiness Score Card ─── */
.readiness-score-card {
  background: linear-gradient(135deg, var(--bg-primary), var(--bg-tertiary));
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-4) var(--space-5);
  margin-bottom: var(--space-4);
  box-shadow: var(--shadow-sm), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  transition: border-color var(--transition-fast);
}
.readiness-score-card:hover {
  border-color: var(--border-default);
}
.readiness-score-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.readiness-score-title {
  font-size: var(--font-size-xs);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
}
.readiness-score-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 10px;
  border-radius: 12px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}
.gauge-percentage {
  font-size: 2.2rem;
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1;
  font-family: var(--font-mono);
  min-width: 72px;
}
.progress-track-layout {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.progress-bar-track {
  width: 100%;
  height: 8px;
  background: var(--bg-hover);
  border-radius: 4px;
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 1s cubic-bezier(0.16, 1, 0.3, 1);
}
.score-helper-text {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.2;
}
</style>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { compareTexts } from '../composables/useTauri';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { Store } from '@tauri-apps/plugin-store';
import type { CompareResult } from '../types';

const providersStore = useProvidersStore();
const appStore = useAppStore();

const textA = ref('');
const textB = ref('');
const loading = ref(false);
const result = ref<CompareResult | null>(null);

const parsedAnalysis = computed(() => {
  if (!result.value?.ai_analysis) return null;
  try {
    // Sometimes the AI returns markdown code fences around JSON
    let cleanJson = result.value.ai_analysis.trim();
    if (cleanJson.startsWith('```json')) {
      cleanJson = cleanJson.replace(/^```json\s*/, '');
    }
    if (cleanJson.startsWith('```')) {
      cleanJson = cleanJson.replace(/^```\s*/, '');
    }
    if (cleanJson.endsWith('```')) {
      cleanJson = cleanJson.replace(/\s*```$/, '');
    }
    return JSON.parse(cleanJson);
  } catch (e) {
    return null;
  }
});

const handleCompare = async () => {
  if (!textA.value.trim() || !textB.value.trim()) {
    appStore.notify('Both text fields must have content', 'error');
    return;
  }
  loading.value = true;
  try {
    let apiKey: string | undefined;
    let model: string | undefined;
    let endpoint: string | undefined;
    const provider = providersStore.activeProvider;
    if (provider) {
      const store = await Store.load('credentials.json');
      apiKey = (await store.get<string>(`apikey_${provider.id}`)) || undefined;
      model = provider.selected_model;
      endpoint = provider.endpoint || undefined;
    }
    result.value = await compareTexts(textA.value, textB.value, apiKey, model, endpoint);
  } catch (e: any) {
    appStore.notify('Comparison failed', 'error');
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="modal-backdrop" @click="appStore.showCompareModal = false">
    <div class="modal-content compare-modal" @click.stop>
      <div class="modal-header">
        <div class="compare-header">
          <h2>Compare Texts</h2>
          <p class="text-sm text-secondary">Paste two texts to see a side-by-side diff and AI analysis</p>
        </div>
        <button class="btn btn-ghost btn-icon" @click="appStore.showCompareModal = false">×</button>
      </div>
      
      <div class="modal-body compare-view">
        <div class="compare-editors">
          <div class="compare-pane">
            <label class="label">Text A</label>
            <textarea class="input compare-textarea" v-model="textA" placeholder="Paste first text here..." />
          </div>
          <div class="compare-divider">
            <button class="btn btn-primary" @click="handleCompare" :disabled="loading">
              {{ loading ? '...' : 'Compare' }}
            </button>
          </div>
          <div class="compare-pane">
            <label class="label">Text B</label>
            <textarea class="input compare-textarea" v-model="textB" placeholder="Paste second text here..." />
          </div>
        </div>
        <div v-if="result" class="compare-results">
          <div class="diff-view card">
            <h3 class="text-sm font-semibold" style="margin-bottom: 12px;">Diff</h3>
            <div class="diff-content">
              <template v-for="(seg, i) in result.diff_segments" :key="i">
                <span v-if="seg.tag === 'equal'">{{ seg.old_text }}</span>
                <span v-else-if="seg.tag === 'delete'" class="diff-delete">{{ seg.old_text }}</span>
                <span v-else-if="seg.tag === 'insert'" class="diff-insert">{{ seg.new_text }}</span>
              </template>
            </div>
          </div>
          <div v-if="result.ai_analysis" class="analysis-view card">
            <h3 class="text-sm font-semibold" style="margin-bottom: 12px;">AI Analysis</h3>
            <div v-if="parsedAnalysis" class="analysis-parsed">
              <div class="analysis-summary">
                <p style="margin-bottom: 8px;"><strong>Summary:</strong> {{ parsedAnalysis.comparison_summary }}</p>
                <p class="verdict"><strong>Verdict:</strong> {{ parsedAnalysis.verdict }}</p>
              </div>
              <div class="analysis-columns">
                <div class="analysis-col">
                  <h4>Text A</h4>
                  <div class="strengths">
                    <h5>Strengths</h5>
                    <ul>
                      <li v-for="(item, idx) in parsedAnalysis.text_a_analysis?.strengths" :key="'a-s-'+idx">{{ item }}</li>
                    </ul>
                  </div>
                  <div class="weaknesses">
                    <h5>Weaknesses</h5>
                    <ul>
                      <li v-for="(item, idx) in parsedAnalysis.text_a_analysis?.weaknesses" :key="'a-w-'+idx">{{ item }}</li>
                    </ul>
                  </div>
                </div>
                <div class="analysis-col">
                  <h4>Text B</h4>
                  <div class="strengths">
                    <h5>Strengths</h5>
                    <ul>
                      <li v-for="(item, idx) in parsedAnalysis.text_b_analysis?.strengths" :key="'b-s-'+idx">{{ item }}</li>
                    </ul>
                  </div>
                  <div class="weaknesses">
                    <h5>Weaknesses</h5>
                    <ul>
                      <li v-for="(item, idx) in parsedAnalysis.text_b_analysis?.weaknesses" :key="'b-w-'+idx">{{ item }}</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
            <pre v-else class="analysis-content">{{ result.ai_analysis }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { background: var(--bg-primary); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); width: 90vw; max-width: 1200px; height: 90vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: var(--shadow-xl); }
.modal-header { padding: var(--space-4); border-bottom: 1px solid var(--border-subtle); display: flex; align-items: flex-start; justify-content: space-between; flex-shrink: 0; background: var(--bg-secondary); }
.modal-body { flex: 1; overflow-y: auto; padding: var(--space-6); }

.compare-view { display: flex; flex-direction: column; gap: var(--space-5); }
.compare-header h2 { font-size: var(--font-size-xl); font-weight: 600; letter-spacing: -0.02em; margin-bottom: var(--space-1); }

.compare-editors { display: flex; gap: var(--space-3); margin-bottom: var(--space-5); }
.compare-pane { flex: 1; display: flex; flex-direction: column; }
.compare-textarea { min-height: 200px; resize: vertical; font-family: var(--font-sans); font-size: var(--font-size-sm); line-height: var(--line-height-relaxed); }
.compare-divider { display: flex; align-items: center; padding-top: 18px; }

.compare-results { display: flex; flex-direction: column; gap: var(--space-4); }
.diff-content { font-size: var(--font-size-sm); line-height: var(--line-height-relaxed); white-space: pre-wrap; word-break: break-word; }
.analysis-content {
  font-size: var(--font-size-sm);
  line-height: var(--line-height-relaxed);
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-secondary);
  font-family: var(--font-sans);
  background: none;
  margin: 0;
  padding: 0;
}

.analysis-summary {
  background: var(--bg-hover);
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-4);
  font-size: var(--font-size-sm);
}
.analysis-summary p { margin-bottom: var(--space-2); }
.analysis-summary p:last-child { margin-bottom: 0; }
.analysis-summary .verdict { color: var(--accent-muted); }

.analysis-columns {
  display: flex;
  gap: var(--space-4);
}
.analysis-col {
  flex: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: var(--space-3);
}
.analysis-col h4 {
  font-size: var(--font-size-sm);
  font-weight: 600;
  margin-bottom: var(--space-3);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--border-subtle);
}
.analysis-col h5 {
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: var(--space-2);
}
.strengths, .weaknesses { margin-bottom: var(--space-3); }
.weaknesses { margin-bottom: 0; }
.analysis-col ul {
  list-style: none;
  padding: 0;
  margin: 0;
}
.analysis-col li {
  font-size: var(--font-size-sm);
  margin-bottom: var(--space-2);
  color: var(--text-secondary);
  padding-left: var(--space-3);
  position: relative;
}
.strengths li::before {
  content: "✓";
  color: var(--color-success);
  position: absolute;
  left: 0;
}
.weaknesses li::before {
  content: "✗";
  color: var(--color-error);
  position: absolute;
  left: 0;
}
</style>

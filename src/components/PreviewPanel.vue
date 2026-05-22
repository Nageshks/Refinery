<script setup lang="ts">
import type { PreviewResult } from '../types';

defineProps<{
  preview: PreviewResult | null;
}>();
</script>

<template>
  <div class="preview-panel" v-if="preview">
    <div class="preview-header">
      <h4>Preview</h4>
      <span class="badge badge-neutral">{{ preview.applied_count }} changes applied</span>
    </div>
    <div class="preview-content" v-html="preview.preview_content" />
  </div>
  <div class="preview-panel preview-empty" v-else>
    <p class="text-xs text-muted">Approve suggestions to see a preview</p>
  </div>
</template>

<style scoped>
.preview-panel {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  overflow: hidden;
}
.preview-header {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
}
.preview-header h4 { font-size: var(--font-size-xs); font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
.preview-content {
  padding: var(--space-3);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-relaxed);
  color: var(--text-primary);
  max-height: 300px;
  overflow-y: auto;
  white-space: pre-wrap;
}
.preview-empty {
  padding: var(--space-4);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>

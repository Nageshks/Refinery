<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { usePagesStore } from '../stores/pages';
import { useAppStore } from '../stores/app';
import { listVersions, restoreVersion, renameVersion, deleteVersion } from '../composables/useTauri';
import type { VersionSnapshot } from '../types';
import VersionPreviewModal from './VersionPreviewModal.vue';

const pagesStore = usePagesStore();
const appStore = useAppStore();
const versions = ref<VersionSnapshot[]>([]);
const loading = ref(false);

const renamingVersionId = ref<string | null>(null);
const renameVersionValue = ref('');
const selectedPreviewVersion = ref<VersionSnapshot | null>(null);

const fetchVersions = async () => {
  if (!pagesStore.activePageId) return;
  loading.value = true;
  try {
    versions.value = await listVersions(pagesStore.activePageId);
  } catch (e) {
    console.error('Failed to load versions:', e);
  } finally {
    loading.value = false;
  }
};

onMounted(fetchVersions);

// Reactive auto-refresh when switching pages or when the active page is updated (snapshots created)
watch(() => pagesStore.activePageId, fetchVersions);
watch(() => pagesStore.activePage?.updated_at, fetchVersions);

const showRestoreModal = ref(false);
const versionToRestore = ref<VersionSnapshot | null>(null);

const handleRestore = async (version: VersionSnapshot) => {
  if (!pagesStore.activePageId) return;
  versionToRestore.value = version;
  showRestoreModal.value = true;
};

const confirmRestore = async () => {
  if (!pagesStore.activePageId || !versionToRestore.value) return;
  const version = versionToRestore.value;
  showRestoreModal.value = false;
  try {
    await restoreVersion(pagesStore.activePageId, version.id);
    await pagesStore.selectPage(pagesStore.activePageId);
    appStore.notify('Version restored successfully', 'success');
    await fetchVersions();
  } catch (e) {
    appStore.notify('Failed to restore version', 'error');
  } finally {
    versionToRestore.value = null;
  }
};

const startRenameVersion = (version: VersionSnapshot) => {
  renamingVersionId.value = version.id;
  renameVersionValue.value = version.name || '';
};

const commitRenameVersion = async (version: VersionSnapshot) => {
  if (!renamingVersionId.value) return;
  const newName = renameVersionValue.value.trim();
  if (newName === '') {
    renamingVersionId.value = null;
    return;
  }
  try {
    await renameVersion(version.id, newName);
    appStore.notify('Version renamed successfully', 'success');
    await fetchVersions();
  } catch (e) {
    appStore.notify('Failed to rename version', 'error');
  } finally {
    renamingVersionId.value = null;
  }
};

const onRestoreFromPreview = async (version: VersionSnapshot) => {
  selectedPreviewVersion.value = null;
  await handleRestore(version);
};

// Version Snapshot Deletion
const showDeleteModal = ref(false);
const versionToDelete = ref<VersionSnapshot | null>(null);

const handleDeleteVersion = (version: VersionSnapshot) => {
  versionToDelete.value = version;
  showDeleteModal.value = true;
};

const confirmDeleteVersion = async () => {
  if (!versionToDelete.value) return;
  const version = versionToDelete.value;
  showDeleteModal.value = false;
  try {
    await deleteVersion(version.id);
    appStore.notify('Version deleted successfully', 'success');
    if (pagesStore.activePageId) {
      await pagesStore.selectPage(pagesStore.activePageId);
    }
    await fetchVersions();
  } catch (e) {
    appStore.notify('Failed to delete version', 'error');
  } finally {
    versionToDelete.value = null;
  }
};

const formatDate = (dateStr: string) => {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }) + ', ' + 
           d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: true });
  } catch { return dateStr; }
};
</script>

<template>
  <div class="versions-panel">
    <div class="versions-list" v-if="versions.length > 0">
      <div class="timeline-line"></div>
      <div v-for="v in versions" :key="v.id" class="version-item">
        <!-- Timeline node dot -->
        <div class="timeline-node">
          <div class="node-dot"></div>
        </div>
        
        <div class="version-card" @click="selectedPreviewVersion = v">
          <div class="version-info">
            <template v-if="renamingVersionId === v.id">
              <input
                class="rename-version-input"
                v-model="renameVersionValue"
                @blur="commitRenameVersion(v)"
                @keyup.enter="commitRenameVersion(v)"
                @keyup.escape="renamingVersionId = null"
                @click.stop
                autofocus
              />
            </template>
            <template v-else>
              <div class="version-title-row">
                <span class="version-name truncate" :title="v.name || 'Version Snapshot'" @dblclick.stop="startRenameVersion(v)">
                  {{ v.name || 'Version Snapshot' }}
                </span>
                <button class="btn-edit-icon" @click.stop="startRenameVersion(v)" title="Rename version">✏️</button>
                <button class="btn-delete-icon" @click.stop="handleDeleteVersion(v)" title="Delete version">🗑️</button>
              </div>
              <div class="version-meta-row">
                <span class="version-time text-xs">{{ formatDate(v.created_at) }}</span>
                <span class="meta-dot text-xs">•</span>
                <span class="version-suggestions text-xs">{{ v.applied_suggestion_ids.length }} suggestions</span>
              </div>
            </template>
          </div>
          <button class="btn-restore" @click.stop="handleRestore(v)" title="Restore this draft">
            Restore
          </button>
        </div>
      </div>
    </div>
    <div v-else-if="!loading" class="versions-empty">
      <div class="empty-icon">⏳</div>
      <p class="empty-text">No version history yet.</p>
      <p class="empty-subtext">Snapshots are created automatically when you apply AI suggestions.</p>
    </div>

    <!-- Preview Modal -->
    <VersionPreviewModal
      v-if="selectedPreviewVersion"
      :version="selectedPreviewVersion"
      :activePageContent="pagesStore.activePage?.content || ''"
      @close="selectedPreviewVersion = null"
      @restore="onRestoreFromPreview"
    />

    <!-- Custom Restore Confirm Modal -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showRestoreModal" class="modal-backdrop prompt-backdrop" @click="showRestoreModal = false">
          <div class="modal-content prompt-modal" @click.stop>
            <div class="prompt-accent-bar"></div>
            <div class="modal-header custom-prompt-header">
              <h3 class="modal-title-custom">Restore Snapshot</h3>
              <button class="close-modal-btn" @click="showRestoreModal = false">×</button>
            </div>
            <div class="modal-body custom-prompt-body">
              <p class="delete-warning-text">
                Are you sure you want to restore the snapshot <strong>"{{ versionToRestore?.name || 'Version Snapshot' }}"</strong>?
              </p>
              <p class="delete-sub-text">This will replace your current page content with this snapshot's content. This action can be undone by restoring another draft.</p>
            </div>
            <div class="modal-footer custom-prompt-footer">
              <button class="btn btn-outline" @click="showRestoreModal = false">Cancel</button>
              <button class="btn btn-primary" @click="confirmRestore">Restore Version</button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Custom Delete Modal -->
      <Transition name="fade">
        <div v-if="showDeleteModal" class="modal-backdrop prompt-backdrop" @click="showDeleteModal = false">
          <div class="modal-content prompt-modal" @click.stop>
            <div class="prompt-accent-bar bar-delete"></div>
            <div class="modal-header custom-prompt-header">
              <h3 class="modal-title-custom text-error">Delete Snapshot</h3>
              <button class="close-modal-btn" @click="showDeleteModal = false">×</button>
            </div>
            <div class="modal-body custom-prompt-body">
              <p class="delete-warning-text">
                Are you sure you want to permanently delete the snapshot <strong>"{{ versionToDelete?.name || 'Version Snapshot' }}"</strong>?
              </p>
              <p class="delete-sub-text">This will permanently remove this version snapshot from draft history. This action cannot be undone.</p>
            </div>
            <div class="modal-footer custom-prompt-footer">
              <button class="btn btn-outline" @click="showDeleteModal = false">Cancel</button>
              <button class="btn-danger" @click="confirmDeleteVersion">Delete Snapshot</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.versions-panel {
  padding: var(--space-3) var(--space-4);
  position: relative;
}

.versions-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  position: relative;
  padding-left: var(--space-4);
}

/* Vertical Timeline Track Line */
.timeline-line {
  position: absolute;
  left: 6px;
  top: 4px;
  bottom: 4px;
  width: 1px;
  background: var(--border-subtle);
}

.version-item {
  position: relative;
  display: flex;
  align-items: center;
}

/* Timeline bullet */
.timeline-node {
  position: absolute;
  left: -14px;
  width: 9px;
  height: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.node-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--text-muted);
  border: 1px solid var(--border-subtle);
  transition: all var(--transition-fast);
}

.version-item:hover .node-dot {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  box-shadow: 0 0 4px var(--accent-primary);
}

/* Card layout */
.version-card {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px var(--space-2);
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  transition: all var(--transition-fast);
}

.version-card:hover {
  background: var(--bg-hover);
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.version-time,
.version-suggestions {
  font-size: 11px;
  font-weight: 400;
  color: var(--text-muted);
}

.meta-dot {
  font-size: 11px;
  color: var(--text-disabled);
  opacity: 0.5;
}

/* Restore Action Button */
.btn-restore {
  padding: 3px var(--space-2);
  height: auto;
  min-height: auto;
  font-size: 10px;
  font-weight: 500;
  text-transform: none;
  border: 1px solid var(--border-default);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  cursor: pointer;
}

.btn-restore:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-strong);
}

/* Empty states */
.versions-empty {
  padding: var(--space-5) var(--space-2);
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
}

.empty-icon {
  font-size: 20px;
  opacity: 0.4;
  margin-bottom: var(--space-1);
}

.empty-text {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-secondary);
}

.empty-subtext {
  font-size: 10px;
  color: var(--text-muted);
  max-width: 170px;
  line-height: 1.4;
}

/* Custom styles for renaming and previewing versions */
.version-card {
  cursor: pointer;
  transition: all var(--transition-fast);
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  flex: 1;
  min-width: 0;
}

.version-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  min-width: 0;
  width: 100%;
}

.version-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.version-meta-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: 1px;
}

.rename-version-input {
  width: 100%;
  padding: 2px 4px;
  background: var(--bg-input);
  border: 1px solid var(--accent-primary);
  border-radius: var(--radius-xs);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  outline: none;
}

.btn-edit-icon {
  background: none;
  border: none;
  font-size: 10px;
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--transition-fast);
  padding: 2px;
}

.version-card:hover .btn-edit-icon {
  opacity: 0.6;
}

.btn-edit-icon:hover {
  opacity: 1 !important;
}

/* Custom Prompt Modals Styling (Purple accent / Glassmorphic) */
.prompt-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(8, 9, 12, 0.45);
  backdrop-filter: blur(10px);
  z-index: 3000; /* overlay above standard modals */
  display: flex;
  align-items: center;
  justify-content: center;
}

.prompt-modal {
  width: 95vw;
  max-width: 440px !important;
  background: var(--bg-dropdown) !important;
  border: 1px solid var(--border-strong) !important;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.2), 0 0 0 1px rgba(0, 0, 0, 0.05), 0 0 16px var(--accent-subtle) !important;
  border-radius: 16px;
  overflow: hidden;
  animation: modalEnter 0.24s cubic-bezier(0.34, 1.56, 0.64, 1);
  display: flex;
  flex-direction: column;
}

.prompt-accent-bar {
  height: 4px;
  background: var(--accent-primary);
  width: 100%;
}

.custom-prompt-header {
  padding: 20px 24px 12px 24px !important;
  border-bottom: 1px solid var(--border-subtle) !important;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary) !important;
}

.modal-title-custom {
  font-size: var(--font-size-base);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.01em;
}

.close-modal-btn {
  font-size: 22px;
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

.close-modal-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  transform: rotate(90deg);
}

.custom-prompt-body {
  padding: 24px 24px 18px 24px !important;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.delete-warning-text {
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  line-height: 1.5;
  margin: 0;
}

.delete-sub-text {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  line-height: 1.4;
  margin: 0;
}

.custom-prompt-footer {
  padding: 12px 24px 24px 24px !important;
  border-top: none !important;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background: transparent !important;
}

@keyframes modalEnter {
  from { opacity: 0; transform: scale(0.96) translateY(8px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Version Deletion & Modal Accents */
.btn-delete-icon {
  background: none;
  border: none;
  font-size: 10px;
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--transition-fast);
  padding: 2px;
  margin-left: 2px;
}

.version-card:hover .btn-delete-icon {
  opacity: 0.6;
}

.btn-delete-icon:hover {
  opacity: 1 !important;
  color: var(--color-error);
}

.prompt-accent-bar.bar-delete {
  background: var(--color-error);
}

.modal-title-custom.text-error {
  color: var(--color-error, #ef4444);
}

.btn-danger {
  background: var(--color-error, #ef4444);
  color: #ffffff !important;
  border: 1px solid transparent;
  padding: 8px 16px;
  font-size: var(--font-size-sm);
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  font-family: var(--font-sans);
}

.btn-danger:hover {
  background: #dc2626;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
}
</style>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useAppStore } from '../stores/app';
import { usePagesStore } from '../stores/pages';
import VersionHistory from './VersionHistory.vue';
import { createManualVersion } from '../composables/useTauri';

const appStore = useAppStore();
const pagesStore = usePagesStore();

const activeTab = ref<'pages' | 'history'>('pages');

// New Page Modal States
const newPageInputRef = ref<HTMLInputElement | null>(null);
const showNewPageModal = ref(false);
const newPageTitle = ref('');

const openNewPageModal = () => {
  newPageTitle.value = '';
  showNewPageModal.value = true;
};

const confirmCreatePage = async () => {
  const pageTitle = newPageTitle.value.trim() || 'Untitled';
  showNewPageModal.value = false;
  try {
    await pagesStore.createNewPage(pageTitle);
    appStore.notify(`Page "${pageTitle}" created`, 'success');
  } catch (e) {
    appStore.notify('Failed to create page', 'error');
  }
};

watch(showNewPageModal, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      newPageInputRef.value?.focus();
    });
  }
});

// New Snapshot Modal States
const newSnapshotInputRef = ref<HTMLInputElement | null>(null);
const showNewSnapshotModal = ref(false);
const newSnapshotName = ref('');
const snapshotDefaultPrefill = ref('');

const openNewSnapshotModal = () => {
  if (!pagesStore.activePageId) return;
  const defaultName = `Snapshot - ${new Date().toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}`;
  snapshotDefaultPrefill.value = defaultName;
  newSnapshotName.value = defaultName;
  showNewSnapshotModal.value = true;
};

const confirmCreateSnapshot = async () => {
  if (!pagesStore.activePageId) return;
  const finalName = newSnapshotName.value.trim() || snapshotDefaultPrefill.value;
  showNewSnapshotModal.value = false;
  try {
    await createManualVersion(pagesStore.activePageId, finalName);
    appStore.notify('Manual snapshot created', 'success');
    await pagesStore.selectPage(pagesStore.activePageId);
  } catch (e) {
    appStore.notify('Failed to create snapshot', 'error');
  }
};

watch(showNewSnapshotModal, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      newSnapshotInputRef.value?.focus();
    });
  }
});

// Custom Delete Page Modal States
const showDeletePageModal = ref(false);
const pageToDeleteId = ref('');
const pageToDeleteTitle = ref('');

const deletePage = (id: string, title: string) => {
  pageToDeleteId.value = id;
  pageToDeleteTitle.value = title;
  showDeletePageModal.value = true;
};

const confirmDeletePage = async () => {
  showDeletePageModal.value = false;
  try {
    await pagesStore.deletePage(pageToDeleteId.value);
    appStore.notify('Page deleted successfully', 'success');
  } catch (e) {
    appStore.notify('Failed to delete page', 'error');
  }
};

const editingPageId = ref<string | null>(null);
const renameInputVal = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

const startRename = (id: string, currentTitle: string) => {
  editingPageId.value = id;
  renameInputVal.value = currentTitle;
  nextTick(() => {
    renameInputRef.value?.focus();
    renameInputRef.value?.select();
  });
};

const finishRename = async (id: string) => {
  if (editingPageId.value !== id) return;
  const newTitle = renameInputVal.value.trim();
  editingPageId.value = null;
  if (!newTitle || newTitle === pagesStore.pages.find(p => p.id === id)?.title) return;
  try {
    await pagesStore.renamePage(id, newTitle);
    appStore.notify('Page renamed successfully', 'success');
  } catch (e) {
    appStore.notify('Failed to rename page', 'error');
  }
};

const isDragging = ref(false);

const initResize = (e: MouseEvent) => {
  e.preventDefault();
  isDragging.value = true;
  document.body.classList.add('is-resizing');

  const startWidth = appStore.sidebarWidth;
  const startX = e.clientX;

  const doResize = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX;
    appStore.sidebarWidth = Math.max(180, Math.min(450, startWidth + deltaX));
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
</script>

<template>
  <aside
    :class="['sidebar', { collapsed: appStore.sidebarCollapsed }]"
    :style="{ width: appStore.sidebarCollapsed ? '0px' : appStore.sidebarWidth + 'px' }"
  >
    <div v-if="!appStore.sidebarCollapsed" class="sidebar-wrapper">
      <!-- Premium Sliding Tabs Bar -->
      <div class="sidebar-tabs-container">
        <div class="sidebar-tabs">
          <div 
            class="tab-indicator" 
            :style="{ transform: `translateX(${activeTab === 'pages' ? '0%' : '100%'})` }"
          ></div>
          <button 
            class="tab-btn" 
            :class="{ active: activeTab === 'pages' }" 
            @click="activeTab = 'pages'"
          >
            <span>Pages</span>
          </button>
          <button 
            class="tab-btn" 
            :class="{ active: activeTab === 'history' }" 
            @click="activeTab = 'history'"
          >
            <span>History</span>
          </button>
        </div>
      </div>

      <!-- Tab Content Panels -->
      <div class="sidebar-panels-container">
        <!-- PAGES PANEL -->
        <div v-if="activeTab === 'pages'" class="sidebar-section pages-section">
          <div class="sidebar-header">
            <span class="sidebar-title">All Pages</span>
            <button 
              class="btn btn-ghost btn-icon sm" 
              @click="openNewPageModal" 
              title="Create new page"
            >+</button>
          </div>
          <div class="sidebar-content">
            <div class="pages-list" v-if="pagesStore.pages.length > 0">
              <div 
                v-for="page in pagesStore.pages" 
                :key="page.id" 
                :class="['page-item', { active: page.id === pagesStore.activePageId }]"
                @click="editingPageId !== page.id && appStore.requestPageSwitch(page.id)"
                @dblclick="startRename(page.id, page.title)"
              >
                <div v-if="editingPageId === page.id" class="page-item-left" @click.stop style="width: 100%; padding-right: 8px;">
                  <span class="page-item-icon">📄</span>
                  <input
                    ref="renameInputRef"
                    type="text"
                    class="sidebar-rename-input"
                    v-model="renameInputVal"
                    @keyup.enter="finishRename(page.id)"
                    @keyup.escape="editingPageId = null"
                    @blur="finishRename(page.id)"
                  />
                </div>
                <div v-else class="page-item-left">
                  <span class="page-item-icon">📄</span>
                  <span class="page-item-title truncate">{{ page.title || 'Untitled' }}</span>
                </div>
                <div class="page-item-right" @click.stop>
                  <button 
                    v-if="editingPageId !== page.id"
                    class="btn btn-ghost btn-icon sm edit-page-btn" 
                    @click="startRename(page.id, page.title)" 
                    title="Rename page"
                    style="margin-right: 4px; font-size: 10px;"
                  >
                    ✏️
                  </button>
                  <button 
                    v-if="editingPageId !== page.id"
                    class="btn btn-ghost btn-icon sm delete-page-btn" 
                    @click="deletePage(page.id, page.title)" 
                    title="Delete page"
                  >
                    ✕
                  </button>
                </div>
              </div>
            </div>
            <div v-else class="sidebar-empty">
              <p class="text-xs text-muted">No pages available.</p>
            </div>
          </div>
        </div>

        <!-- HISTORY PANEL -->
        <div v-else class="sidebar-section history-section">
          <div class="sidebar-header">
            <span class="sidebar-title">Version History</span>
            <button 
              v-if="pagesStore.activePageId" 
              class="btn btn-ghost btn-icon sm" 
              @click="openNewSnapshotModal" 
              title="New manual snapshot"
            >+</button>
          </div>
          <div class="sidebar-content">
            <VersionHistory v-if="pagesStore.activePageId" />
            <div v-else class="sidebar-empty">
              <p class="text-xs text-muted">No page selected</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Settings Icon Sidebar Footer -->
      <div class="sidebar-footer">
        <button class="sidebar-settings-btn" @click="appStore.showSettingsModal = true" title="Settings">
          <span class="settings-icon">⚙️</span>
          <span class="settings-text">Settings</span>
        </button>
      </div>
    </div>

    <!-- Resizer handle -->
    <div
      v-if="!appStore.sidebarCollapsed"
      :class="['sidebar-resizer', { 'is-dragging': isDragging }]"
      @mousedown="initResize"
    ></div>
  </aside>

  <!-- Custom Teleported Dialog Modals -->
  <Teleport to="body">
    <!-- Custom Page Create Modal -->
    <Transition name="fade">
      <div v-if="showNewPageModal" class="modal-backdrop prompt-backdrop" @click="showNewPageModal = false">
        <div class="modal-content prompt-modal" @click.stop>
          <div class="prompt-accent-bar"></div>
          <div class="modal-header custom-prompt-header">
            <h3 class="modal-title-custom">Create New Page</h3>
            <button class="close-modal-btn" @click="showNewPageModal = false">×</button>
          </div>
          <div class="modal-body custom-prompt-body">
            <div class="form-field">
              <label class="label prompt-label-style">Page Title</label>
              <input
                ref="newPageInputRef"
                type="text"
                class="input custom-prompt-input"
                v-model="newPageTitle"
                placeholder="Enter title here..."
                @keyup.enter="confirmCreatePage"
                @keyup.escape="showNewPageModal = false"
              />
            </div>
          </div>
          <div class="modal-footer custom-prompt-footer">
            <button class="btn btn-outline" @click="showNewPageModal = false">Cancel</button>
            <button class="btn btn-primary" @click="confirmCreatePage">Create Page</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Custom Snapshot Create Modal -->
    <Transition name="fade">
      <div v-if="showNewSnapshotModal" class="modal-backdrop prompt-backdrop" @click="showNewSnapshotModal = false">
        <div class="modal-content prompt-modal" @click.stop>
          <div class="prompt-accent-bar"></div>
          <div class="modal-header custom-prompt-header">
            <h3 class="modal-title-custom">Create Version Snapshot</h3>
            <button class="close-modal-btn" @click="showNewSnapshotModal = false">×</button>
          </div>
          <div class="modal-body custom-prompt-body">
            <div class="form-field">
              <label class="label prompt-label-style">Snapshot Name</label>
              <input
                ref="newSnapshotInputRef"
                type="text"
                class="input custom-prompt-input"
                v-model="newSnapshotName"
                :placeholder="snapshotDefaultPrefill"
                @keyup.enter="confirmCreateSnapshot"
                @keyup.escape="showNewSnapshotModal = false"
              />
            </div>
          </div>
          <div class="modal-footer custom-prompt-footer">
            <button class="btn btn-outline" @click="showNewSnapshotModal = false">Cancel</button>
            <button class="btn btn-primary" @click="confirmCreateSnapshot">Create Snapshot</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Custom Page Delete Modal -->
    <Transition name="fade">
      <div v-if="showDeletePageModal" class="modal-backdrop prompt-backdrop" @click="showDeletePageModal = false">
        <div class="modal-content prompt-modal" @click.stop>
          <div class="prompt-accent-bar bar-delete"></div>
          <div class="modal-header custom-prompt-header">
            <h3 class="modal-title-custom text-error">Delete Page</h3>
            <button class="close-modal-btn" @click="showDeletePageModal = false">×</button>
          </div>
          <div class="modal-body custom-prompt-body">
            <p class="delete-warning-text">
              Are you sure you want to permanently delete <strong>"{{ pageToDeleteTitle }}"</strong>?
            </p>
            <p class="delete-sub-text">This action cannot be undone and you will lose all the version snapshots associated with this page.</p>
          </div>
          <div class="modal-footer custom-prompt-footer">
            <button class="btn btn-outline" @click="showDeletePageModal = false">Cancel</button>
            <button class="btn-danger" @click="confirmDeletePage">Delete Page</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width var(--transition-slow);
  overflow: hidden;
}
.sidebar.collapsed { width: var(--sidebar-collapsed); border-right: none; }

.sidebar-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* Premium Sliding Tab Switcher styles */
.sidebar-tabs-container {
  padding: var(--space-3) var(--space-4) 0;
  flex-shrink: 0;
}

.sidebar-tabs {
  position: relative;
  display: flex;
  height: 32px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
  border: 1px solid var(--border-subtle);
}

.tab-indicator {
  position: absolute;
  left: 2px;
  top: 2px;
  bottom: 2px;
  width: calc(50% - 2px);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-sm);
  transition: transform var(--transition-normal);
  z-index: 1;
}

.tab-btn {
  flex: 1;
  z-index: 2;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color var(--transition-fast);
}

.tab-btn span {
  position: relative;
}

.tab-btn:hover,
.tab-btn.active {
  color: var(--text-primary);
}

/* Layout container for panels */
.sidebar-panels-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-section {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.sidebar-header {
  padding: var(--space-3) var(--space-4);
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.sidebar-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
}

.sidebar-empty { 
  padding: var(--space-6) var(--space-4); 
  text-align: center; 
}

/* Pages list visual items styles */
.pages-list {
  display: flex;
  flex-direction: column;
  padding: var(--space-2) 0;
}

.page-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px var(--space-4);
  border-left: 3px solid transparent;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
  user-select: none;
}

.page-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.page-item.active {
  background: var(--accent-subtle);
  border-left-color: var(--accent-primary);
  color: var(--text-primary);
}

.page-item-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.page-item-icon {
  font-size: 13px;
  opacity: 0.75;
}

.page-item-title {
  font-size: var(--font-size-sm);
  font-weight: 500;
  letter-spacing: -0.01em;
}

.page-item-right {
  display: flex;
  align-items: center;
  margin-left: var(--space-2);
}

.delete-page-btn,
.edit-page-btn {
  opacity: 0;
  transition: opacity var(--transition-fast);
  color: var(--text-muted);
}

.page-item:hover .delete-page-btn,
.page-item:hover .edit-page-btn {
  opacity: 0.65;
}

.page-item:hover .delete-page-btn:hover {
  opacity: 1;
  color: var(--color-error);
  background: rgba(248, 113, 113, 0.08);
}

.page-item:hover .edit-page-btn:hover {
  opacity: 1;
  color: var(--accent-primary);
  background: var(--accent-subtle);
}

.sidebar-rename-input {
  width: 100%;
  background: var(--bg-input);
  border: 1px solid var(--accent-primary);
  border-radius: var(--radius-sm);
  padding: 2px var(--space-2);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  outline: none;
  height: 24px;
}

/* Sidebar Resizer */
.sidebar-resizer {
  position: absolute;
  top: 0;
  right: -3px;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  z-index: 100;
  transition: background var(--transition-fast);
}

.sidebar-resizer:hover,
.sidebar-resizer.is-dragging {
  background: var(--accent-primary);
  opacity: 0.5;
}

/* Sidebar Bottom Footer Styling */
.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.sidebar-settings-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 8px var(--space-3);
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
}

.sidebar-settings-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-settings-btn .settings-icon {
  font-size: 14px;
  opacity: 0.85;
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

.prompt-accent-bar.bar-delete {
  background: var(--color-error);
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

.modal-title-custom.text-error {
  color: var(--color-error, #ef4444);
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

.prompt-label-style {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: 8px;
}

.custom-prompt-input {
  width: 100%;
  background: var(--bg-input) !important;
  border: 1px solid var(--border-default) !important;
  border-radius: var(--radius-md) !important;
  padding: 12px 14px !important;
  color: var(--text-primary) !important;
  font-size: var(--font-size-sm) !important;
  transition: all var(--transition-fast) !important;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.03);
}

.custom-prompt-input:focus {
  border-color: var(--accent-primary, #8b5cf6) !important;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.03), 0 0 0 3px var(--accent-subtle, rgba(139, 92, 246, 0.15)) !important;
  outline: none;
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
</style>

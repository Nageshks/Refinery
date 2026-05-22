<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useAppStore } from './stores/app';
import { usePagesStore } from './stores/pages';
import { useProvidersStore } from './stores/providers';
import AppBar from './components/AppBar.vue';
import Sidebar from './components/Sidebar.vue';
import Editor from './components/Editor.vue';
import ReviewSplitView from './components/ReviewSplitView.vue';
import ReviewPanel from './components/ReviewPanel.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import CompareView from './components/CompareView.vue';
import PageSwitcherModal from './components/PageSwitcherModal.vue';
import AppIcon from './components/AppIcon.vue';


const appStore = useAppStore();
const pagesStore = usePagesStore();
const providersStore = useProvidersStore();

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.ctrlKey || e.metaKey) {
    const key = e.key.toLowerCase();
    if (key === 'k') {
      e.preventDefault();
      e.stopPropagation();
      appStore.showPageSwitcher = !appStore.showPageSwitcher;
    } else if (key === '=' || key === '+') {
      e.preventDefault();
      appStore.zoomIn();
    } else if (key === '-') {
      e.preventDefault();
      appStore.zoomOut();
    } else if (key === '0') {
      e.preventDefault();
      appStore.resetZoom();
    }
  }
};

const handleWheel = (e: WheelEvent) => {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault();
    if (e.deltaY < 0) {
      appStore.zoomIn();
    } else if (e.deltaY > 0) {
      appStore.zoomOut();
    }
  }
};

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown, true);
  window.addEventListener('wheel', handleWheel, { passive: false });
  await pagesStore.fetchPages();
  await providersStore.fetchProviders();
  if (pagesStore.pages.length > 0 && !pagesStore.activePageId) {
    await pagesStore.selectPage(pagesStore.pages[0].id);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true);
  window.removeEventListener('wheel', handleWheel);
});
</script>

<template>
  <div class="app-shell">
    <AppBar />
    <div class="app-body">
      <Sidebar />
      <main class="workspace">
        <div class="editor-area">
          <ReviewSplitView v-if="appStore.activeView === 'review' && pagesStore.activePage" />
          <Editor v-else-if="pagesStore.activePage" />
          <div v-else class="empty-state">
            <AppIcon :size="64" class="empty-logo-accent" />
            <h2>No page selected</h2>
            <p>Create a new page or select one from the sidebar to start writing.</p>
            <button class="btn btn-primary btn-lg" @click="pagesStore.createNewPage()">
              Create Page
            </button>
          </div>
        </div>
        <ReviewPanel v-if="appStore.activeView === 'review'" />
      </main>
    </div>

    <!-- Modals -->
    <Teleport to="body">
      <CompareView v-if="appStore.showCompareModal" />
      <SettingsPanel v-if="appStore.showSettingsModal" />
      <PageSwitcherModal v-if="appStore.showPageSwitcher" />

      <!-- Discard Review Suggestions Warning Modal -->
      <Transition name="fade">
        <div v-if="appStore.showDiscardReviewModal" class="modal-backdrop prompt-backdrop" @click="appStore.cancelDiscardReview">
          <div class="modal-content prompt-modal" @click.stop>
            <div class="prompt-accent-bar bar-delete"></div>
            <div class="modal-header custom-prompt-header">
              <h3 class="modal-title-custom text-error">Discard Polishing Suggestions?</h3>
              <button class="close-modal-btn" @click="appStore.cancelDiscardReview">×</button>
            </div>
            <div class="modal-body custom-prompt-body">
              <p class="delete-warning-text">
                Switching views or changing pages will discard your active polishing suggestions and erase them.
              </p>
              <p class="delete-sub-text">Are you sure you want to proceed? This active session cannot be recovered.</p>
            </div>
            <div class="modal-footer custom-prompt-footer">
              <button class="btn btn-outline" @click="appStore.cancelDiscardReview">Cancel</button>
              <button class="btn-danger" @click="appStore.confirmDiscardReview">Discard & Proceed</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Notifications -->
    <Transition name="fade">
      <div
        v-if="appStore.notification"
        :class="['toast', `toast-${appStore.notification.type}`]"
      >
        {{ appStore.notification.message }}
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-primary);
  overflow: hidden;
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.workspace {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  color: var(--text-muted);
}
.empty-logo-accent {
  margin-bottom: var(--space-2);
  opacity: 0.85;
  filter: drop-shadow(0 8px 16px rgba(139, 92, 246, 0.12));
}
.empty-state h2 { font-size: var(--font-size-lg); font-weight: 600; color: var(--text-secondary); }
.empty-state p { font-size: var(--font-size-sm); max-width: 300px; text-align: center; }

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
</style>
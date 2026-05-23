<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { useAppStore } from '../stores/app';
import { usePagesStore } from '../stores/pages';
import { invoke } from '@tauri-apps/api/core';
import AppIcon from './AppIcon.vue';

const appStore = useAppStore();
const pagesStore = usePagesStore();

const toolsDropdownOpen = ref(false);

const toggleToolsDropdown = () => {
  toolsDropdownOpen.value = !toolsDropdownOpen.value;
};

const triggerCompareTool = () => {
  appStore.showCompareModal = true;
  toolsDropdownOpen.value = false;
};

const triggerPageSwitcher = () => {
  appStore.showPageSwitcher = true;
  toolsDropdownOpen.value = false;
};

const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.tools-dropdown-container')) {
    toolsDropdownOpen.value = false;
  }
};

onMounted(() => {
  window.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  window.removeEventListener('click', handleClickOutside);
});

const minimizeWindow = () => {
  invoke('minimize_window').catch(err => console.error('Failed to minimize:', err));
};

const maximizeWindow = () => {
  invoke('maximize_window').catch(err => console.error('Failed to maximize:', err));
};

const closeWindow = () => {
  invoke('close_window').catch(err => console.error('Failed to close:', err));
};

// Toolbar Page Title Renaming
const isEditingTitle = ref(false);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

const startRename = () => {
  if (!pagesStore.activePage) return;
  renameValue.value = pagesStore.activePage.title;
  isEditingTitle.value = true;
  nextTick(() => {
    renameInputRef.value?.focus();
    renameInputRef.value?.select();
  });
};

const finishRename = async () => {
  if (!isEditingTitle.value || !pagesStore.activePageId) return;
  const newTitle = renameValue.value.trim();
  isEditingTitle.value = false;
  if (!newTitle || newTitle === pagesStore.activePage?.title) return;
  try {
    await pagesStore.renamePage(pagesStore.activePageId, newTitle);
    appStore.notify('Page renamed successfully', 'success');
  } catch (e) {
    appStore.notify('Failed to rename page', 'error');
  }
};
</script>

<template>
  <header class="appbar">
    <div class="appbar-left">
      <button class="btn btn-ghost btn-icon sm toggle-sidebar-btn" @click="appStore.toggleSidebar">
        ☰
      </button>
      <div class="appbar-brand" data-tauri-drag-region style="cursor: default; gap: var(--space-2); margin-right: var(--space-4);">
        <AppIcon :size="18" data-tauri-drag-region />
        <span class="brand-text" data-tauri-drag-region>Refinery</span>
      </div>
      
      <!-- Tools Dropdown Menu -->
      <div class="tools-dropdown-container">
        <button class="btn btn-ghost btn-sm tools-btn" @click.stop="toggleToolsDropdown">
          Tools <span class="dropdown-chevron">▾</span>
        </button>
        <Transition name="dropdown">
          <div v-if="toolsDropdownOpen" class="tools-dropdown-menu">
            <button class="dropdown-item" @click="triggerCompareTool">
              <span class="dropdown-item-icon">⚖️</span>
              <div class="dropdown-item-details">
                <span class="dropdown-item-title">Compare Drafts</span>
                <span class="dropdown-item-subtitle">Compare active draft with reviews</span>
              </div>
            </button>
            <button class="dropdown-item" @click="triggerPageSwitcher">
              <span class="dropdown-item-icon">🔍</span>
              <div class="dropdown-item-details">
                <span class="dropdown-item-title">Command Palette</span>
                <span class="dropdown-item-subtitle">Search pages & run commands</span>
              </div>
            </button>
          </div>
        </Transition>
      </div>
    </div>
    
    <div class="appbar-center" data-tauri-drag-region style="cursor: default; display: flex; align-items: center; justify-content: center; height: 100%;">
      <template v-if="pagesStore.activePage">
        <div v-if="isEditingTitle" style="display: flex; align-items: center;" @click.stop>
          <input
            ref="renameInputRef"
            type="text"
            class="appbar-rename-input"
            v-model="renameValue"
            @keyup.enter="finishRename"
            @keyup.escape="isEditingTitle = false"
            @blur="finishRename"
          />
        </div>
        <span 
          v-else 
          class="page-title hover-editable" 
          @click="startRename"
          title="Click to rename draft"
        >
          {{ pagesStore.activePage.title }}
          <span class="edit-hint-icon">✏️</span>
        </span>
      </template>
    </div>
    
    <nav class="appbar-right">
      <div class="view-tabs">
        <button
          :class="['tab-btn', { active: appStore.activeView === 'edit' }]"
          @click="appStore.requestViewSwitch('edit')"
        >
          Think
        </button>
        <button
          :class="['tab-btn', { active: appStore.activeView === 'review' }]"
          @click="appStore.requestViewSwitch('review')"
        >
          Polish
        </button>
      </div>
      
      <div class="window-controls">
        <button class="window-btn minimize-btn" @click="minimizeWindow" title="Minimize">
          <svg width="10" height="1" viewBox="0 0 10 1"><path d="M0,0 L10,0" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
        <button class="window-btn maximize-btn" @click="maximizeWindow" title="Maximize">
          <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
        <button class="window-btn close-btn" @click="closeWindow" title="Close">
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1,1 L9,9 M9,1 L1,9" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>
    </nav>
  </header>
</template>

<style scoped>
.appbar {
  height: var(--appbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-left: var(--space-4);
  padding-right: 0;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  z-index: 20;
}

.appbar-left {
  display: flex;
  align-items: center;
  min-width: 250px;
}
.toggle-sidebar-btn {
  margin-right: var(--space-2);
  font-size: 16px;
}

.appbar-brand {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.brand-icon {
  color: var(--accent-primary);
  font-size: var(--font-size-lg);
}
.brand-text {
  font-size: var(--font-size-md);
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

.appbar-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
.page-title {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  font-weight: 500;
}

.appbar-right {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.view-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
}

.tab-btn {
  padding: 4px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.tab-btn:hover {
  color: var(--text-secondary);
}
.tab-btn.active {
  background: var(--bg-elevated);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
}

.window-controls {
  display: flex;
  align-items: center;
  height: 100%;
}
.window-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: var(--appbar-height);
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.window-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.close-btn:hover {
  background: #E81123; /* Windows close red */
  color: #fff;
}

/* Tools Dropdown Container */
.tools-dropdown-container {
  position: relative;
  margin-left: var(--space-2);
}

.tools-btn {
  font-weight: 500;
  font-size: var(--font-size-sm);
  display: flex;
  align-items: center;
  gap: 4px;
}

.dropdown-chevron {
  font-size: 10px;
  opacity: 0.6;
}

.tools-dropdown-menu {
  position: absolute;
  top: calc(100% + var(--space-2));
  left: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg), 0 0 12px rgba(139, 92, 246, 0.08);
  width: 240px;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 1000;
  backdrop-filter: blur(8px);
}

.dropdown-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 8px var(--space-3);
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
}

.dropdown-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.dropdown-item-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.dropdown-item-details {
  display: flex;
  flex-direction: column;
}

.dropdown-item-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  line-height: 1.2;
}

.dropdown-item-subtitle {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.3;
  margin-top: 1px;
}

/* Transitions */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* Toolbar Title Renaming Styles */
.appbar-rename-input {
  background: var(--bg-input);
  border: 1px solid var(--accent-primary);
  border-radius: var(--radius-sm);
  padding: 4px 10px;
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  width: 240px;
  height: 28px;
  outline: none;
  font-family: var(--font-sans);
  text-align: center;
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.page-title.hover-editable {
  cursor: pointer;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-sm);
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  transition: all var(--transition-fast);
}

.page-title.hover-editable:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.edit-hint-icon {
  font-size: 11px;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.page-title.hover-editable:hover .edit-hint-icon {
  opacity: 0.65;
}
</style>

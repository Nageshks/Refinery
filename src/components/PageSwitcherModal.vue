<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import { usePagesStore } from '../stores/pages';
import { useAppStore } from '../stores/app';

console.log('PageSwitcherModal: Component file evaluated.');

const pagesStore = usePagesStore();
const appStore = useAppStore();

const searchQuery = ref('');
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

const currentView = ref<'main' | 'canvas-bg'>('main');
const originalCanvasBg = ref<string>('');

const canvasBgItems = [
  { id: 'default', name: 'Charcoal', desc: 'Balanced grey-slate draft canvas', icon: '🌑' },
  { id: 'stark', name: 'Minimal Stark', desc: 'Absolute distraction-free high-ink contrast', icon: '⚪' },
  { id: 'alabaster', name: 'Alabaster', desc: 'Clean off-white with cool-grey touch', icon: '🌫️' },
  { id: 'sakura', name: 'Cozy Sakura', desc: 'Soothing off-white with warm rose blush', icon: '🌸' },
  { id: 'dracula', name: 'Gothic Dracula', desc: 'Vibrant violet-gothic overlay style', icon: '🧛' },
  { id: 'slate', name: 'Slate Blue', desc: 'Calming, deep slate blue-grey preset', icon: '🦕' },
  { id: 'nord', name: 'Nordic Frost', desc: 'Frost-grey and nordic slate elements', icon: '❄️' },
  { id: 'rose', name: 'Rose Pine', desc: 'Dusky twilight pink-rose and gold-clay', icon: '🌲' },
  { id: 'parchment', name: 'Warm Ivory', desc: 'Comforting sepia warm paper textures', icon: '📜' },
  { id: 'contrast', name: 'High Contrast', desc: 'Pure solid high-contrast borders', icon: '☯️' }
];

// Predefined palette commands
interface CommandItem {
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  category: 'Commands' | 'Pages' | 'Canvas Environments';
  action: () => void;
}

const commands: CommandItem[] = [
  {
    id: 'toggle-theme',
    title: 'Toggle Light/Dark Theme',
    subtitle: 'Switch between light and dark interface colors',
    icon: '🌓',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle Theme.');
      appStore.theme = appStore.theme === 'light' ? 'dark' : 'light';
      appStore.notify(`Theme set to ${appStore.theme === 'dark' ? 'Dark' : 'Light'}`, 'success');
    }
  },
  {
    id: 'set-theme-dark',
    title: 'Set Theme to Dark',
    subtitle: 'Apply beautiful violet dark mode aesthetics',
    icon: '🌑',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Set Theme to Dark.');
      appStore.theme = 'dark';
      appStore.notify('Theme set to dark', 'success');
    }
  },
  {
    id: 'set-theme-light',
    title: 'Set Theme to Light',
    subtitle: 'Apply clean high-contrast light aesthetics',
    icon: '☀️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Set Theme to Light.');
      appStore.theme = 'light';
      appStore.notify('Theme set to light', 'success');
    }
  },
  {
    id: 'set-theme-system',
    title: 'Set Theme to System',
    subtitle: 'Follow operating system theme preferences',
    icon: '🖥️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Set Theme to System.');
      appStore.theme = 'system';
      appStore.notify('Theme set to system preferred', 'success');
    }
  },
  {
    id: 'open-settings',
    title: 'Open Settings',
    subtitle: 'Configure AI provider, api keys, and model options',
    icon: '⚙️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Open Settings.');
      appStore.showSettingsModal = true;
    }
  },
  {
    id: 'open-compare',
    title: 'Open Compare Tool',
    subtitle: 'Compare active page drafts with version snapshots',
    icon: '⚖️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Open Compare.');
      appStore.showCompareModal = true;
    }
  },
  {
    id: 'zoom-in',
    title: 'Zoom In',
    subtitle: 'Increase editor text sizing (+10%)',
    icon: '➕',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Zoom In.');
      appStore.zoomIn();
    }
  },
  {
    id: 'zoom-out',
    title: 'Zoom Out',
    subtitle: 'Decrease editor text sizing (-10%)',
    icon: '➖',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Zoom Out.');
      appStore.zoomOut();
    }
  },
  {
    id: 'reset-zoom',
    title: 'Reset Zoom to 100%',
    subtitle: 'Restore editor default text sizing',
    icon: '🔄',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Reset Zoom.');
      appStore.resetZoom();
    }
  },
  {
    id: 'toggle-zen-focus',
    title: 'Toggle Zen Focus Mode',
    subtitle: 'Automatically hide sidebars and headers when typing',
    icon: '🧘',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle Zen Focus.');
      appStore.zenFocusEnabled = !appStore.zenFocusEnabled;
      appStore.notify(`Zen Focus Mode ${appStore.zenFocusEnabled ? 'Enabled' : 'Disabled'}`, 'success');
    }
  },
  {
    id: 'toggle-sidebar',
    title: 'Toggle Left Sidebar',
    subtitle: 'Show or hide the navigation and drafts panel',
    icon: '📋',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle Sidebar.');
      appStore.toggleSidebar();
    }
  },
  {
    id: 'toggle-right-sidebar',
    title: 'Toggle Right Sidebar',
    subtitle: 'Show or hide the AI tools and auditor right panel',
    icon: '📋',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle Right Sidebar.');
      appStore.toggleRightSidebar();
    }
  },
  {
    id: 'toggle-appbar',
    title: 'Toggle Title/App Bar',
    subtitle: 'Show or hide the top menu and window action bar',
    icon: '🖥️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle App Bar.');
      appStore.appbarHidden = !appStore.appbarHidden;
      appStore.notify(`App bar ${appStore.appbarHidden ? 'Hidden' : 'Visible'}`, 'success');
    }
  },
  {
    id: 'switch-to-polish',
    title: 'Switch to Polish Mode',
    subtitle: 'Access AI feedback, grammar polish, and edit history side-by-side',
    icon: '✨',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Switch to Polish Mode.');
      if (appStore.activeView !== 'review') {
        appStore.togglePolishPanel();
      }
    }
  },
  {
    id: 'switch-to-think',
    title: 'Switch to Think Mode',
    subtitle: 'Immerse in clean, distraction-free markdown editing',
    icon: '✍️',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Switch to Think Mode.');
      if (appStore.activeView === 'review') {
        appStore.togglePolishPanel();
      }
    }
  },
  {
    id: 'toggle-audit-panel',
    title: 'Toggle Draft Auditor Panel',
    subtitle: 'Show or hide the dynamic Chameleon UI writing auditor sidebar',
    icon: '🔍',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Executing Toggle Audit Panel.');
      appStore.toggleAuditorPanel();
    }
  },
  {
    id: 'switch-canvas-bg',
    title: 'Switch Canvas Environment (Theme)...',
    subtitle: 'Interactively preview and select canvas backgrounds',
    icon: '🎨',
    category: 'Commands',
    action: () => {
      console.log('CommandPalette: Morphing into Canvas Bg view.');
      originalCanvasBg.value = appStore.canvasBg;
      currentView.value = 'canvas-bg';
      searchQuery.value = '';
      selectedIndex.value = 0;
    }
  }
];

const formatDate = (dateStr: string | null | undefined) => {
  if (!dateStr) return 'Unknown';
  try {
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return 'Unknown';
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return d.toLocaleDateString();
  } catch (err) {
    console.error('formatDate error:', err);
    return 'Unknown';
  }
};

const filteredItems = computed(() => {
  try {
    const q = searchQuery.value.toLowerCase().trim();

    if (currentView.value === 'canvas-bg') {
      const items: CommandItem[] = canvasBgItems.map((bg): CommandItem => ({
        id: bg.id,
        title: bg.name,
        subtitle: bg.desc,
        icon: bg.icon,
        category: 'Canvas Environments',
        action: () => {
          appStore.canvasBg = bg.id as any;
          appStore.notify(`Canvas environment set to "${bg.name}"`, 'success');
        }
      }));
      if (!q) return items;
      return items.filter(item => 
        item.title.toLowerCase().includes(q) || 
        (item.subtitle && item.subtitle.toLowerCase().includes(q))
      );
    }

    const rawPages = pagesStore.pages || [];

    // Page items formatted as executable switcher items
    const pageItems: CommandItem[] = rawPages.map((p): CommandItem | null => {
      if (!p) return null;
      return {
        id: p.id || Math.random().toString(),
        title: p.title || 'Untitled',
        subtitle: `Last updated ${formatDate(p.updated_at)}`,
        icon: '📄',
        category: 'Pages',
        action: async () => {
          console.log(`CommandPalette: Selecting page: ${p.id}`);
          if (p.id) await appStore.requestPageSwitch(p.id);
        }
      };
    }).filter((item): item is CommandItem => item !== null);

    if (!q) {
      return [...pageItems, ...commands];
    }

    // Filter lists independently then combine
    const matchedPages = pageItems.filter(item => item.title.toLowerCase().includes(q));
    const matchedCommands = commands.filter(item => 
      item.title.toLowerCase().includes(q) || 
      (item.subtitle && item.subtitle.toLowerCase().includes(q))
    );

    return [...matchedPages, ...matchedCommands];
  } catch (err) {
    console.error('filteredItems computed error:', err);
    return [];
  }
});

// Keep selectedIndex within bounds when results list changes
watch(filteredItems, () => {
  selectedIndex.value = 0;
  if (currentView.value === 'canvas-bg' && filteredItems.value.length > 0) {
    const activeItem = filteredItems.value[0];
    if (activeItem) {
      appStore.canvasBg = activeItem.id as any;
    }
  }
}, { deep: true });

// Live arrow-key real-time previews for canvas backgrounds
watch(selectedIndex, (newIdx) => {
  if (currentView.value === 'canvas-bg' && filteredItems.value.length > 0) {
    const activeItem = filteredItems.value[newIdx];
    if (activeItem) {
      appStore.canvasBg = activeItem.id as any;
    }
  }
});

// Safely restore start environment on cancel / Esc / Backdrop clicks
const handleCancel = () => {
  if (currentView.value === 'canvas-bg') {
    appStore.canvasBg = originalCanvasBg.value as any;
    currentView.value = 'main';
    searchQuery.value = '';
    selectedIndex.value = 0;
    nextTick(() => {
      inputRef.value?.focus();
    });
  } else {
    appStore.showPageSwitcher = false;
  }
};

onMounted(() => {
  console.log('PageSwitcherModal: Component mounted.');
  nextTick(() => {
    if (inputRef.value) {
      console.log('PageSwitcherModal: Auto-focusing input field.');
      inputRef.value.focus();
    } else {
      console.warn('PageSwitcherModal: Input reference not ready for focus.');
    }
  });
});

const navigateDown = () => {
  if (filteredItems.value.length === 0) return;
  selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length;
  scrollActiveIntoView();
};

const navigateUp = () => {
  if (filteredItems.value.length === 0) return;
  selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length;
  scrollActiveIntoView();
};

const scrollActiveIntoView = () => {
  nextTick(() => {
    const activeItem = document.querySelector('.switcher-item.active');
    activeItem?.scrollIntoView({ block: 'nearest' });
  });
};

const executeItem = async (item: CommandItem) => {
  if (!item) return;
  console.log('PageSwitcherModal: Executing item:', item.title);

  if (item.id === 'switch-canvas-bg') {
    await item.action();
    nextTick(() => {
      inputRef.value?.focus();
    });
    return;
  }

  appStore.showPageSwitcher = false;
  try {
    await item.action();
  } catch (err) {
    console.error('PageSwitcherModal: Action execution error:', err);
  }
};

const handleEnter = async () => {
  if (filteredItems.value.length > 0) {
    const targetItem = filteredItems.value[selectedIndex.value];
    await executeItem(targetItem);
  } else if (searchQuery.value.trim() !== '') {
    // Dynamic fast creation
    try {
      const title = searchQuery.value.trim();
      console.log('PageSwitcherModal: Fast-creating page:', title);
      const newPage = await pagesStore.createNewPage(title);
      appStore.notify(`Created page "${newPage.title}"`, 'success');
      appStore.showPageSwitcher = false;
    } catch (e) {
      console.error('PageSwitcherModal: Fast page creation error:', e);
      appStore.notify('Failed to create page', 'error');
    }
  }
};
</script>

<template>
  <div :class="['modal-backdrop', 'switcher-backdrop', { 'preview-mode': currentView === 'canvas-bg' }]" @click="handleCancel">
    <div class="switcher-modal" @click.stop>
      <div class="switcher-search-container">
        <!-- Back Arrow Button for Canvas Mode -->
        <button 
          v-if="currentView === 'canvas-bg'" 
          class="btn btn-ghost btn-icon sm switcher-back-btn" 
          @click="handleCancel" 
          title="Back to commands"
          style="margin-right: var(--space-1); width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; font-size: 14px; padding: 0; min-height: 0; background: transparent; border: none; cursor: pointer; color: var(--text-muted);"
        >
          ←
        </button>
        <span v-else class="switcher-search-icon">🔍</span>
        <input 
          ref="inputRef"
          type="text" 
          class="switcher-input" 
          v-model="searchQuery" 
          :placeholder="currentView === 'canvas-bg' ? 'Filter environments... (Esc to cancel)' : 'Search pages or type commands...'"
          @keydown.down.prevent="navigateDown"
          @keydown.up.prevent="navigateUp"
          @keydown.enter.prevent="handleEnter"
          @keydown.esc.prevent="handleCancel"
        />
        <span class="switcher-esc-badge">ESC</span>
      </div>

      <div class="switcher-body">
        <div class="switcher-list" v-if="filteredItems && filteredItems.length > 0">
          <template v-for="(item, idx) in filteredItems" :key="item.id">
            <!-- Render category header on transition -->
            <div 
              v-if="idx === 0 || (filteredItems[idx - 1] && item.category !== filteredItems[idx - 1].category)" 
              class="switcher-section-title"
            >
              {{ item.category }}
            </div>
            
            <button
              :class="['switcher-item', { active: idx === selectedIndex }]"
              @click="executeItem(item)"
              @mouseenter="selectedIndex = idx"
            >
              <div class="switcher-item-left">
                <span class="switcher-item-icon">{{ item.icon }}</span>
                <div class="switcher-item-details">
                  <span class="page-title truncate">{{ item.title }}</span>
                  <span class="page-subtitle truncate" v-if="item.subtitle">{{ item.subtitle }}</span>
                </div>
              </div>
              <span class="page-category-badge">{{ item.category }}</span>
            </button>
          </template>
        </div>
        
        <div class="switcher-empty" v-else>
          <span class="empty-icon">💡</span>
          <template v-if="searchQuery.trim() !== ''">
            <p class="empty-text">No pages or commands match "{{ searchQuery }}"</p>
            <p class="empty-subtext">Press <kbd class="kbd">Enter</kbd> to create a new page with this title.</p>
          </template>
          <template v-else>
            <p class="empty-text">No pages or commands available.</p>
            <p class="empty-subtext">Type a search query to search.</p>
          </template>
        </div>
      </div>

      <div class="switcher-footer">
        <div class="footer-help">
          <span v-if="currentView === 'canvas-bg'"><kbd class="kbd">↑↓</kbd> Preview Theme</span>
          <span v-else><kbd class="kbd">↑↓</kbd> Navigate</span>
          
          <span v-if="currentView === 'canvas-bg'"><kbd class="kbd">Enter</kbd> Apply Theme</span>
          <span v-else><kbd class="kbd">Enter</kbd> Execute</span>
          
          <span><kbd class="kbd">Esc</kbd> {{ currentView === 'canvas-bg' ? 'Cancel' : 'Close' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.switcher-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(8, 9, 12, 0.6);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  z-index: 2000;
  animation: fadeIn var(--transition-fast);
  transition: background 0.25s ease, backdrop-filter 0.25s ease;
}

.switcher-backdrop.preview-mode {
  background: rgba(0, 0, 0, 0.08); /* Near-transparent background tint */
  backdrop-filter: none; /* Zero blur to ensure complete backdrop visibility */
}

.switcher-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  width: 90vw;
  max-width: 580px;
  max-height: 60vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl), 0 0 24px rgba(139, 92, 246, 0.15);
  animation: scaleIn var(--transition-fast);
}

.switcher-search-container {
  display: flex;
  align-items: center;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  gap: var(--space-3);
}

.switcher-search-icon {
  font-size: 14px;
  color: var(--text-muted);
}

.switcher-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-md);
  outline: none;
}
.switcher-input::placeholder {
  color: var(--text-muted);
}

.switcher-esc-badge {
  font-size: 9px;
  font-weight: 600;
  padding: 2px 5px;
  background: var(--bg-hover);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xs);
  color: var(--text-muted);
}

.switcher-body {
  flex: 1;
  overflow-y: auto;
  max-height: 400px;
}

.switcher-list {
  padding: var(--space-2);
}

.switcher-section-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: var(--space-3) var(--space-3) var(--space-1);
}

.switcher-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px var(--space-3);
  background: transparent;
  border: none;
  border-left: 3px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
  gap: var(--space-3);
}

.switcher-item:hover, .switcher-item.active {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.switcher-item.active {
  background: var(--accent-subtle);
  border-left-color: var(--accent-primary);
}

.switcher-item-left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex: 1;
  min-width: 0;
}

.switcher-item-icon {
  font-size: 16px;
  opacity: 0.85;
  flex-shrink: 0;
  width: 20px;
  text-align: center;
}

.switcher-item-details {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.page-title {
  font-weight: 600;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  line-height: 1.2;
}

.page-subtitle {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  font-weight: 400;
}

.page-category-badge {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 6px;
  background: rgba(139, 92, 246, 0.08);
  border: 1px solid rgba(139, 92, 246, 0.15);
  border-radius: var(--radius-sm);
  color: var(--accent-primary);
  opacity: 0.85;
  flex-shrink: 0;
}

.switcher-item.active .page-category-badge {
  background: rgba(139, 92, 246, 0.16);
  border-color: rgba(139, 92, 246, 0.3);
  color: var(--accent-primary);
  opacity: 1;
}

.switcher-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
  gap: var(--space-2);
  color: var(--text-muted);
}

.empty-icon {
  font-size: 24px;
  opacity: 0.5;
}

.empty-text {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-secondary);
}

.empty-subtext {
  font-size: var(--font-size-xs);
}

.kbd {
  font-family: var(--font-mono);
  background: var(--bg-hover);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xs);
  padding: 1px 4px;
  font-size: 10px;
  color: var(--text-secondary);
}

.switcher-footer {
  padding: var(--space-3) var(--space-5);
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-subtle);
}

.footer-help {
  display: flex;
  gap: var(--space-5);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleIn {
  from { transform: scale(0.97); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>

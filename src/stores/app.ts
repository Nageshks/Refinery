import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { ViewMode } from '../types';
import { useReviewStore } from './review';
import { usePagesStore } from './pages';
import { getGpuAcceleration, setGpuAcceleration } from '../composables/useTauri';

export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(localStorage.getItem('refinery_sidebar_collapsed') === 'true');

  watch(sidebarCollapsed, (val) => {
    localStorage.setItem('refinery_sidebar_collapsed', val ? 'true' : 'false');
  });

  const activeView = ref<ViewMode>('edit');
  const showCompareModal = ref(false);
  const showSettingsModal = ref(false);
  const showPageSwitcher = ref(false);
  const auditPanelVisible = ref(false);
  const activeSidebarTab = ref<'speller' | 'polish' | 'auditor' | null>(null);

  // Sync initial state if activeView starts as review
  if (activeView.value === 'review') {
    activeSidebarTab.value = 'polish';
  } else if (auditPanelVisible.value) {
    activeSidebarTab.value = 'auditor';
  }

  // Keep tab rail in sync with view modifications
  watch(activeView, (newView) => {
    if (newView !== 'review') {
      if (activeSidebarTab.value === 'speller' || activeSidebarTab.value === 'polish') {
        activeSidebarTab.value = auditPanelVisible.value ? 'auditor' : null;
      }
    } else if (newView === 'review' && activeSidebarTab.value !== 'speller' && activeSidebarTab.value !== 'polish') {
      activeSidebarTab.value = 'polish'; // default
    }
  });

  watch(auditPanelVisible, (visible) => {
    if (visible) {
      activeSidebarTab.value = 'auditor';
    } else if (activeSidebarTab.value === 'auditor') {
      activeSidebarTab.value = null;
    }
  });

  const notification = ref<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);

  const showDiscardReviewModal = ref(false);
  const pendingPageSwitchId = ref<string | null>(null);
  const pendingViewSwitch = ref<ViewMode | null>(null);

  // Dynamic widths with localStorage backup
  const sidebarWidth = ref(Number(localStorage.getItem('refinery_sidebar_width') || '260'));
  const reviewPanelWidth = ref(Number(localStorage.getItem('refinery_panel_width') || '380'));

  watch(sidebarWidth, (val) => {
    localStorage.setItem('refinery_sidebar_width', val.toString());
  });

  watch(reviewPanelWidth, (val) => {
    localStorage.setItem('refinery_panel_width', val.toString());
  });

  const theme = ref<'dark' | 'light' | 'system'>((localStorage.getItem('refinery_theme') as 'dark' | 'light' | 'system') || 'system');
  const zoomFactor = ref(Number(localStorage.getItem('refinery_zoom_factor') || '1.0'));
  const spellcheckEnabled = ref(localStorage.getItem('refinery_spellcheck') === 'true');

  // Extended Appearance Controls
  const accentColor = ref<'purple' | 'emerald' | 'ocean' | 'amber' | 'rose'>((localStorage.getItem('refinery_accent_color') as any) || 'purple');
  const highlightColor = ref<'purple' | 'gold' | 'mint' | 'sky'>((localStorage.getItem('refinery_highlight_color') as any) || 'purple');
  const editorWidth = ref<'cozy' | 'wide'>((localStorage.getItem('refinery_editor_width') as any) || 'cozy');
  const editorFont = ref<'sans' | 'inter' | 'outfit' | 'jakarta' | 'instrument' | 'merriweather' | 'lora' | 'mono' | 'atkinson' | 'lexend' | 'poppins' | 'montserrat'>((localStorage.getItem('refinery_editor_font') as any) || 'inter');
  const lineHeight = ref(Number(localStorage.getItem('refinery_line_height') || '1.6'));
  const zenFocusEnabled = ref(localStorage.getItem('refinery_zen_focus') === 'true');
  const gpuAccelerationEnabled = ref(true);

  // New Personalization & Accessibility Controls
  const canvasBg = ref<'default' | 'dracula' | 'slate' | 'parchment' | 'contrast' | 'nord' | 'rose' | 'stark' | 'alabaster' | 'sakura'>((localStorage.getItem('refinery_canvas_bg') as any) || 'default');
  const uiLayoutVariant = ref<'unified' | 'contrasted' | 'glassmorphic'>((localStorage.getItem('refinery_ui_layout_variant') as any) || 'contrasted');
  const editorFontSize = ref(Number(localStorage.getItem('refinery_editor_font_size') || '14'));
  const appbarHidden = ref(localStorage.getItem('refinery_appbar_hidden') === 'true');

  // Load initial GPU state from Rust
  getGpuAcceleration().then((enabled) => {
    gpuAccelerationEnabled.value = enabled;
  }).catch((err) => {
    console.error('Failed to get GPU settings:', err);
  });

  watch(gpuAccelerationEnabled, (val) => {
    setGpuAcceleration(val).catch((err) => {
      console.error('Failed to set GPU settings:', err);
    });
  });

  watch(spellcheckEnabled, (val) => {
    localStorage.setItem('refinery_spellcheck', val ? 'true' : 'false');
  });

  const applyThemeOverrides = () => {
    const root = document.documentElement;
    root.setAttribute('data-accent', accentColor.value);
    root.setAttribute('data-highlight', highlightColor.value);
    root.setAttribute('data-editor-width', editorWidth.value);
    root.setAttribute('data-editor-font', editorFont.value);
    root.setAttribute('data-canvas-bg', canvasBg.value);
    root.setAttribute('data-ui-variant', uiLayoutVariant.value);
    root.setAttribute('data-zen-focus', zenFocusEnabled.value.toString());
    root.setAttribute('data-appbar-hidden', appbarHidden.value.toString());
    root.style.setProperty('--editor-line-height', lineHeight.value.toString());
    root.style.setProperty('--editor-font-size', `${editorFontSize.value}px`);
  };

  watch(accentColor, (val) => {
    localStorage.setItem('refinery_accent_color', val);
    applyThemeOverrides();
  });

  watch(highlightColor, (val) => {
    localStorage.setItem('refinery_highlight_color', val);
    applyThemeOverrides();
  });

  watch(editorWidth, (val) => {
    localStorage.setItem('refinery_editor_width', val);
    applyThemeOverrides();
  });

  watch(editorFont, (val) => {
    localStorage.setItem('refinery_editor_font', val);
    applyThemeOverrides();
  });

  watch(lineHeight, (val) => {
    localStorage.setItem('refinery_line_height', val.toString());
    applyThemeOverrides();
  });

  watch(zenFocusEnabled, (val) => {
    localStorage.setItem('refinery_zen_focus', val ? 'true' : 'false');
    applyThemeOverrides();
  });

  watch(canvasBg, (val) => {
    localStorage.setItem('refinery_canvas_bg', val);
    applyThemeOverrides();
  });

  watch(uiLayoutVariant, (val) => {
    localStorage.setItem('refinery_ui_layout_variant', val);
    applyThemeOverrides();
  });

  watch(editorFontSize, (val) => {
    localStorage.setItem('refinery_editor_font_size', val.toString());
    applyThemeOverrides();
  });

  watch(appbarHidden, (val) => {
    localStorage.setItem('refinery_appbar_hidden', val ? 'true' : 'false');
    applyThemeOverrides();
  });

  const applyZoom = () => {
    document.documentElement.style.removeProperty('zoom');
    document.documentElement.style.setProperty('--editor-zoom', zoomFactor.value.toString());
  };

  const applyTheme = () => {
    const root = document.documentElement;
    if (theme.value === 'system') {
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.setAttribute('data-theme', isDark ? 'dark' : 'light');
    } else {
      root.setAttribute('data-theme', theme.value);
    }
  };

  const dbThemeChange = () => {
    localStorage.setItem('refinery_theme', theme.value);
    applyTheme();
  };

  watch(theme, () => {
    dbThemeChange();
  });

  watch(zoomFactor, (val) => {
    localStorage.setItem('refinery_zoom_factor', val.toString());
    applyZoom();
  });

  // Setup system listener
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaQuery.addEventListener('change', () => {
    if (theme.value === 'system') {
      applyTheme();
    }
  });

  // Call once on init
  applyTheme();
  applyZoom();
  applyThemeOverrides();

  const toggleSidebar = () => { sidebarCollapsed.value = !sidebarCollapsed.value; };
  const setView = (view: ViewMode) => { activeView.value = view; };

  const requestViewSwitch = (view: ViewMode) => {
    const reviewStore = useReviewStore();
    if (activeView.value === 'review' && view === 'edit' && reviewStore.groups.length > 0) {
      pendingViewSwitch.value = 'edit';
      pendingPageSwitchId.value = null;
      showDiscardReviewModal.value = true;
    } else {
      setView(view);
    }
  };

  const requestPageSwitch = (id: string) => {
    const reviewStore = useReviewStore();
    if (activeView.value === 'review' && reviewStore.groups.length > 0) {
      pendingPageSwitchId.value = id;
      pendingViewSwitch.value = null;
      showDiscardReviewModal.value = true;
    } else {
      doPageSwitch(id);
    }
  };

  const toggleSpellerPanel = () => {
    if (activeSidebarTab.value === 'speller') {
      activeSidebarTab.value = null;
      requestViewSwitch('edit');
    } else {
      activeSidebarTab.value = 'speller';
      auditPanelVisible.value = false;
      if (activeView.value !== 'review') {
        requestViewSwitch('review');
      }
      notify('Speller Activated', 'success');
    }
  };

  const togglePolishPanel = () => {
    if (activeSidebarTab.value === 'polish') {
      activeSidebarTab.value = null;
      requestViewSwitch('edit');
    } else {
      activeSidebarTab.value = 'polish';
      auditPanelVisible.value = false;
      if (activeView.value !== 'review') {
        requestViewSwitch('review');
      }
      notify('Polishing Activated', 'success');
    }
  };

  const toggleAuditorPanel = () => {
    if (activeSidebarTab.value === 'auditor') {
      activeSidebarTab.value = null;
      auditPanelVisible.value = false;
      notify('Auditor Panel Hidden', 'success');
    } else {
      activeSidebarTab.value = 'auditor';
      if (activeView.value === 'review') {
        requestViewSwitch('edit');
        const unwatch = watch(activeView, (newView) => {
          if (newView === 'edit') {
            auditPanelVisible.value = true;
            notify('Auditor Panel Visible', 'success');
            unwatch();
          }
        });
        watch(showDiscardReviewModal, (isOpen) => {
          if (!isOpen && activeView.value === 'review') {
            unwatch();
          }
        });
      } else {
        auditPanelVisible.value = true;
        notify('Auditor Panel Visible', 'success');
      }
    }
  };

  const rightRailCollapsed = ref(localStorage.getItem('refinery_right_rail_collapsed') === 'true');

  watch(rightRailCollapsed, (val) => {
    localStorage.setItem('refinery_right_rail_collapsed', val ? 'true' : 'false');
    if (val) {
      activeSidebarTab.value = null;
      auditPanelVisible.value = false;
      if (activeView.value === 'review') {
        setView('edit');
      }
    }
  });

  const toggleRightSidebar = () => {
    rightRailCollapsed.value = !rightRailCollapsed.value;
    notify(`Right Sidebar Rail ${rightRailCollapsed.value ? 'Hidden' : 'Visible'}`, 'success');
  };

  const doPageSwitch = async (id: string) => {
    const pagesStore = usePagesStore();
    const reviewStore = useReviewStore();
    // Default to Think mode on page switch
    setView('edit');
    // Clear review suggestions
    if (pagesStore.activePageId) {
      await reviewStore.clearReview(pagesStore.activePageId);
    } else {
      await reviewStore.clearReview();
    }
    // Select the new page
    await pagesStore.selectPage(id);
  };

  const confirmDiscardReview = async () => {
    const reviewStore = useReviewStore();
    const pagesStore = usePagesStore();
    showDiscardReviewModal.value = false;
    
    // Clear active review session
    if (pagesStore.activePageId) {
      await reviewStore.clearReview(pagesStore.activePageId);
    } else {
      await reviewStore.clearReview();
    }
    setView('edit'); // Always go back to Think mode

    if (pendingPageSwitchId.value) {
      const targetId = pendingPageSwitchId.value;
      pendingPageSwitchId.value = null;
      await pagesStore.selectPage(targetId);
    }
    
    pendingViewSwitch.value = null;
    notify('Polishing session cleared', 'success');
  };

  const cancelDiscardReview = () => {
    showDiscardReviewModal.value = false;
    pendingPageSwitchId.value = null;
    pendingViewSwitch.value = null;
  };

  const notify = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    notification.value = { message, type };
    setTimeout(() => { notification.value = null; }, 4000);
  };

  const zoomIn = () => {
    if (zoomFactor.value < 2.0) {
      zoomFactor.value = Math.min(2.0, Math.round((zoomFactor.value + 0.1) * 10) / 10);
      notify(`Zoom: ${Math.round(zoomFactor.value * 100)}%`, 'info');
    }
  };

  const zoomOut = () => {
    if (zoomFactor.value > 0.5) {
      zoomFactor.value = Math.max(0.5, Math.round((zoomFactor.value - 0.1) * 10) / 10);
      notify(`Zoom: ${Math.round(zoomFactor.value * 100)}%`, 'info');
    }
  };

  const resetZoom = () => {
    zoomFactor.value = 1.0;
    notify('Zoom: 100%', 'info');
  };

  return {
    sidebarCollapsed,
    activeView,
    showCompareModal,
    showSettingsModal,
    showPageSwitcher,
    auditPanelVisible,
    activeSidebarTab,
    notification,
    sidebarWidth,
    reviewPanelWidth,
    theme,
    zoomFactor,
    spellcheckEnabled,
    accentColor,
    highlightColor,
    editorWidth,
    editorFont,
    lineHeight,
    zenFocusEnabled,
    gpuAccelerationEnabled,
    canvasBg,
    uiLayoutVariant,
    editorFontSize,
    appbarHidden,
    showDiscardReviewModal,
    pendingPageSwitchId,
    pendingViewSwitch,
    toggleSidebar,
    setView,
    requestViewSwitch,
    requestPageSwitch,
    toggleAuditorPanel,
    togglePolishPanel,
    toggleSpellerPanel,
    confirmDiscardReview,
    cancelDiscardReview,
    doPageSwitch,
    notify,
    applyTheme,
    applyThemeOverrides,
    zoomIn,
    zoomOut,
    resetZoom,
    applyZoom,
    toggleRightSidebar,
    rightRailCollapsed
  };
});

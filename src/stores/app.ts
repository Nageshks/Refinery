import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { ViewMode } from '../types';
import { useReviewStore } from './review';
import { usePagesStore } from './pages';

export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(false);
  const activeView = ref<ViewMode>('edit');
  const showCompareModal = ref(false);
  const showSettingsModal = ref(false);
  const showPageSwitcher = ref(false);
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

  watch(spellcheckEnabled, (val) => {
    localStorage.setItem('refinery_spellcheck', val ? 'true' : 'false');
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

  watch(theme, (newVal) => {
    localStorage.setItem('refinery_theme', newVal);
    applyTheme();
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
    notification,
    sidebarWidth,
    reviewPanelWidth,
    theme,
    zoomFactor,
    spellcheckEnabled,
    showDiscardReviewModal,
    pendingPageSwitchId,
    pendingViewSwitch,
    toggleSidebar,
    setView,
    requestViewSwitch,
    requestPageSwitch,
    confirmDiscardReview,
    cancelDiscardReview,
    doPageSwitch,
    notify,
    applyTheme,
    zoomIn,
    zoomOut,
    resetZoom,
    applyZoom
  };
});

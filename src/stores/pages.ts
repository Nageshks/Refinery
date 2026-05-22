import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Page } from '../types';
import * as api from '../composables/useTauri';

export const usePagesStore = defineStore('pages', () => {
  const pages = ref<Page[]>([]);
  const activePageId = ref<string | null>(null);
  const loading = ref(false);

  const activePage = computed(() => pages.value.find(p => p.id === activePageId.value) || null);

  const fetchPages = async () => {
    loading.value = true;
    try {
      pages.value = await api.listPages();
    } catch (e) {
      console.error('Failed to fetch pages:', e);
    } finally {
      loading.value = false;
    }
  };

  const createNewPage = async (title: string = 'Untitled') => {
    try {
      const page = await api.createPage(title);
      pages.value.unshift(page);
      activePageId.value = page.id;
      return page;
    } catch (e) {
      console.error('Failed to create page:', e);
      throw e;
    }
  };

  const selectPage = async (id: string) => {
    activePageId.value = id;
    try {
      const page = await api.getPage(id);
      const idx = pages.value.findIndex(p => p.id === id);
      if (idx >= 0) pages.value[idx] = page;
    } catch (e) {
      console.error('Failed to load page:', e);
    }
  };

  const updateActivePage = (title: string, content: string) => {
    const page = pages.value.find(p => p.id === activePageId.value);
    if (page) {
      page.title = title;
      page.content = content;
    }
  };

  const renamePage = async (id: string, title: string) => {
    try {
      const updated = await api.renamePage(id, title);
      const idx = pages.value.findIndex(p => p.id === id);
      if (idx >= 0) pages.value[idx] = updated;
    } catch (e) {
      console.error('Failed to rename page:', e);
      throw e;
    }
  };

  const deletePage = async (id: string) => {
    try {
      await api.deletePage(id);
      pages.value = pages.value.filter(p => p.id !== id);
      if (activePageId.value === id) {
        activePageId.value = pages.value[0]?.id || null;
      }
    } catch (e) {
      console.error('Failed to delete page:', e);
      throw e;
    }
  };

  return {
    pages, activePageId, activePage, loading,
    fetchPages, createNewPage, selectPage, updateActivePage, renamePage, deletePage
  };
});

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProviderConfig } from '../types';
import * as api from '../composables/useTauri';

export const useProvidersStore = defineStore('providers', () => {
  const providers = ref<ProviderConfig[]>([]);
  const loading = ref(false);

  const activeProvider = computed(() => providers.value.find(p => p.enabled) || null);

  const fetchProviders = async () => {
    loading.value = true;
    try {
      providers.value = await api.listProviders();
    } catch (e) {
      console.error('Failed to fetch providers:', e);
    } finally {
      loading.value = false;
    }
  };

  const saveProvider = async (config: {
    id?: string; name: string; providerType: string;
    endpoint?: string; selectedModel: string; enabled: boolean;
  }) => {
    try {
      const saved = await api.saveProviderConfig(config);
      const idx = providers.value.findIndex(p => p.id === saved.id);
      if (idx >= 0) providers.value[idx] = saved;
      else providers.value.push(saved);
      return saved;
    } catch (e) {
      console.error('Failed to save provider:', e);
      throw e;
    }
  };

  const removeProvider = async (id: string) => {
    await api.deleteProviderConfig(id);
    providers.value = providers.value.filter(p => p.id !== id);
  };

  return { providers, activeProvider, loading, fetchProviders, saveProvider, removeProvider };
});

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProviderConfig, ModelConfig } from '../types';
import * as api from '../composables/useTauri';

export const useProvidersStore = defineStore('providers', () => {
  const providers = ref<ProviderConfig[]>([]);
  const models = ref<ModelConfig[]>([]);
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

  const fetchModels = async () => {
    try {
      models.value = await api.listModels();
    } catch (e) {
      console.error('Failed to fetch models:', e);
    }
  };

  const saveModel = async (model: {
    id: string; providerType: string; name: string;
    useCase: string; icon: string; isCustom: boolean;
    enabled: boolean;
  }) => {
    try {
      const saved = await api.saveModelConfig(model);
      const idx = models.value.findIndex(m => m.id === saved.id);
      if (idx >= 0) models.value[idx] = saved;
      else models.value.push(saved);
      return saved;
    } catch (e) {
      console.error('Failed to save model:', e);
      throw e;
    }
  };

  const removeModel = async (id: string) => {
    try {
      await api.deleteModelConfig(id);
      models.value = models.value.filter(m => m.id !== id);
    } catch (e) {
      console.error('Failed to remove model:', e);
      throw e;
    }
  };

  const resetModels = async () => {
    try {
      models.value = await api.resetDefaultModels();
    } catch (e) {
      console.error('Failed to reset models:', e);
      throw e;
    }
  };

  return {
    providers, models, activeProvider, loading,
    fetchProviders, saveProvider, removeProvider,
    fetchModels, saveModel, removeModel, resetModels
  };
});

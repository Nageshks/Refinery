<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { Store } from '@tauri-apps/plugin-store';
import { testProvider, setActiveProvider } from '../composables/useTauri';
import { useAuditStore, type PageTypeDefinition, type AuditCheckDefinition, type AuditWidgetType } from '../stores/audit';

const providersStore = useProvidersStore();
const appStore = useAppStore();
const auditStore = useAuditStore();

watch(() => appStore.gpuAccelerationEnabled, () => {
  appStore.notify('GPU setting updated. Please restart Refinery to apply.', 'info');
});

const currentTab = ref<'appearance' | 'providers' | 'testing' | 'page_types'>('appearance');
const activeSubTab = ref<'openrouter' | 'groq' | 'nvidia'>('openrouter');
const showApiKey = ref(false);

const testing = ref(false);
const testResult = ref<string | null>(null);

const openrouterConfig = ref({
  id: null as string | null,
  name: 'OpenRouter',
  endpoint: 'https://openrouter.ai/api/v1/chat/completions',
  selectedModel: 'openai/gpt-4o-mini',
  apiKey: '',
  enabled: false,
});

const groqConfig = ref({
  id: null as string | null,
  name: 'Groq',
  endpoint: 'https://api.groq.com/openai/v1/chat/completions',
  selectedModel: 'llama-3.3-70b-versatile',
  apiKey: '',
  enabled: false,
});

const nvidiaConfig = ref({
  id: null as string | null,
  name: 'NVIDIA NIM',
  endpoint: 'https://integrate.api.nvidia.com/v1/chat/completions',
  selectedModel: 'nvidia/nemotron-3-nano-omni-30b-a3b-reasoning',
  apiKey: '',
  enabled: false,
});

onMounted(async () => {
  await providersStore.fetchProviders();
  const list = providersStore.providers;
  
  // Load OpenRouter
  const or = list.find(p => p.provider_type === 'openrouter');
  if (or) {
    openrouterConfig.value.id = or.id;
    openrouterConfig.value.name = or.name;
    openrouterConfig.value.endpoint = or.endpoint || 'https://openrouter.ai/api/v1/chat/completions';
    openrouterConfig.value.selectedModel = or.selected_model;
    openrouterConfig.value.enabled = or.enabled;
    try {
      const store = await Store.load('credentials.json');
      const key = await store.get<string>(`apikey_${or.id}`);
      if (key) openrouterConfig.value.apiKey = key;
    } catch {}
  }
  
  // Load Groq
  const gr = list.find(p => p.provider_type === 'groq');
  if (gr) {
    groqConfig.value.id = gr.id;
    groqConfig.value.name = gr.name;
    groqConfig.value.endpoint = gr.endpoint || 'https://api.groq.com/openai/v1/chat/completions';
    groqConfig.value.selectedModel = gr.selected_model;
    groqConfig.value.enabled = gr.enabled;
    try {
      const store = await Store.load('credentials.json');
      const key = await store.get<string>(`apikey_${gr.id}`);
      if (key) groqConfig.value.apiKey = key;
    } catch {}
  }

  // Load NVIDIA NIM
  const nv = list.find(p => p.provider_type === 'nvidia');
  if (nv) {
    nvidiaConfig.value.id = nv.id;
    nvidiaConfig.value.name = nv.name;
    nvidiaConfig.value.endpoint = nv.endpoint || 'https://integrate.api.nvidia.com/v1/chat/completions';
    nvidiaConfig.value.selectedModel = nv.selected_model;
    nvidiaConfig.value.enabled = nv.enabled;
    try {
      const store = await Store.load('credentials.json');
      const key = await store.get<string>(`apikey_${nv.id}`);
      if (key) nvidiaConfig.value.apiKey = key;
    } catch {}
  }

  // Set default active provider tab
  if (nvidiaConfig.value.enabled) {
    activeSubTab.value = 'nvidia';
  } else if (groqConfig.value.enabled) {
    activeSubTab.value = 'groq';
  } else {
    activeSubTab.value = 'openrouter';
  }
});

const providerList = computed(() => [
  { 
    id: 'openrouter' as const, 
    name: 'OpenRouter', 
    icon: '🔗', 
    enabled: openrouterConfig.value.enabled 
  },
  { 
    id: 'groq' as const, 
    name: 'Groq', 
    icon: '⚡', 
    enabled: groqConfig.value.enabled 
  },
  { 
    id: 'nvidia' as const, 
    name: 'NVIDIA NIM', 
    icon: '💚', 
    enabled: nvidiaConfig.value.enabled 
  }
]);

const currentProviderDetails = computed(() => {
  const providerType = activeSubTab.value;
  const list = providersStore.models.filter(m => m.provider_type === providerType);
  const modelIds = list.map(m => m.id);

  if (activeSubTab.value === 'nvidia') {
    return {
      name: 'NVIDIA NIM',
      icon: '💚',
      desc: 'Access high-performance NVIDIA NIM models optimized for local and cloud deployment',
      keyPlaceholder: 'nvapi-...',
      defaultEndpoint: 'https://integrate.api.nvidia.com/v1/chat/completions',
      models: modelIds
    };
  }
  if (activeSubTab.value === 'groq') {
    return {
      name: 'Groq',
      icon: '⚡',
      desc: 'Access ultra-fast Llama and GPT-OSS models via Groq API',
      keyPlaceholder: 'gsk_...',
      defaultEndpoint: 'https://api.groq.com/openai/v1/chat/completions',
      models: modelIds
    };
  }
  return {
    name: 'OpenRouter',
    icon: '🔗',
    desc: 'Access multiple AI models through a single OpenRouter API key',
    keyPlaceholder: 'sk-or-...',
    defaultEndpoint: 'https://openrouter.ai/api/v1/chat/completions',
    models: modelIds
  };
});

const currentConfig = computed(() => {
  if (activeSubTab.value === 'nvidia') return nvidiaConfig.value;
  return activeSubTab.value === 'openrouter' ? openrouterConfig.value : groqConfig.value;
});

const handleActiveToggle = (e: Event) => {
  const checked = (e.target as HTMLInputElement).checked;
  if (activeSubTab.value === 'openrouter') {
    openrouterConfig.value.enabled = checked;
    if (checked) {
      groqConfig.value.enabled = false;
      nvidiaConfig.value.enabled = false;
    }
  } else if (activeSubTab.value === 'groq') {
    groqConfig.value.enabled = checked;
    if (checked) {
      openrouterConfig.value.enabled = false;
      nvidiaConfig.value.enabled = false;
    }
  } else {
    nvidiaConfig.value.enabled = checked;
    if (checked) {
      openrouterConfig.value.enabled = false;
      groqConfig.value.enabled = false;
    }
  }
};

const handleSave = async (type: 'openrouter' | 'groq' | 'nvidia') => {
  const config = type === 'openrouter' ? openrouterConfig.value : type === 'groq' ? groqConfig.value : nvidiaConfig.value;
  try {
    const saved = await providersStore.saveProvider({
      id: config.id || undefined,
      name: config.name,
      providerType: type,
      endpoint: config.endpoint || undefined,
      selectedModel: config.selectedModel,
      enabled: config.enabled,
    });
    config.id = saved.id;
    
    // Save API key securely
    if (config.apiKey) {
      const store = await Store.load('credentials.json');
      await store.set(`apikey_${saved.id}`, config.apiKey);
      await store.save();
    }
    
    // Set active in backend if enabled
    if (config.enabled) {
      await setActiveProvider(saved.id);
    }
    
    await providersStore.fetchProviders();
    appStore.notify(`${config.name} configuration saved`, 'success');
  } catch (e) {
    appStore.notify(`Failed to save ${config.name}`, 'error');
  }
};

const handleTest = async (type: 'openrouter' | 'groq' | 'nvidia') => {
  const config = type === 'openrouter' ? openrouterConfig.value : type === 'groq' ? groqConfig.value : nvidiaConfig.value;
  testing.value = true;
  testResult.value = null;
  try {
    const result = await testProvider(config.apiKey, config.selectedModel, config.endpoint || undefined);
    testResult.value = result;
    appStore.notify('Connection successful', 'success');
  } catch (e: any) {
    testResult.value = typeof e === 'string' ? e : 'Connection failed';
    appStore.notify('Connection test failed', 'error');
  } finally {
    testing.value = false;
  }
};

// Model Management logic
const showConfirmModal = ref(false);
const confirmTitle = ref('');
const confirmMessage = ref('');
const onConfirmCallback = ref<(() => void) | null>(null);

const triggerConfirmation = (title: string, message: string, onConfirm: () => void) => {
  confirmTitle.value = title;
  confirmMessage.value = message;
  onConfirmCallback.value = onConfirm;
  showConfirmModal.value = true;
};

const handleConfirmAction = () => {
  if (onConfirmCallback.value) {
    onConfirmCallback.value();
  }
  showConfirmModal.value = false;
};

const showModelModal = ref(false);
const editingModel = ref<{
  id: string;
  name: string;
  useCase: string;
  icon: string;
  isNew: boolean;
}>({
  id: '',
  name: '',
  useCase: '',
  icon: '🤖',
  isNew: true,
});

const openAddModel = () => {
  editingModel.value = {
    id: '',
    name: '',
    useCase: '',
    icon: '🤖',
    isNew: true,
  };
  showModelModal.value = true;
};

const openEditModel = (model: any) => {
  editingModel.value = {
    id: model.id,
    name: model.name,
    useCase: model.use_case,
    icon: model.icon,
    isNew: false,
  };
  showModelModal.value = true;
};

const handleSaveModel = async () => {
  if (!editingModel.value.id || !editingModel.value.name) {
    appStore.notify('Model ID and Name are required', 'error');
    return;
  }
  try {
    const existing = providersStore.models.find(m => m.id === editingModel.value.id);
    await providersStore.saveModel({
      id: editingModel.value.id,
      providerType: activeSubTab.value,
      name: editingModel.value.name,
      useCase: editingModel.value.useCase,
      icon: editingModel.value.icon,
      isCustom: editingModel.value.isNew ? true : existing?.is_custom ?? true,
      enabled: editingModel.value.isNew ? true : existing?.enabled ?? true,
    });
    showModelModal.value = false;
    appStore.notify('Model saved successfully', 'success');
  } catch (e) {
    appStore.notify('Failed to save model', 'error');
  }
};

const handleDeleteModel = (modelId: string) => {
  triggerConfirmation(
    'Delete AI Model',
    'Are you sure you want to delete this model? This action is permanent and cannot be undone.',
    async () => {
      try {
        await providersStore.removeModel(modelId);
        
        // Check if the deleted model was the currently selected model, if so fallback to first available model
        const config = currentConfig.value;
        if (config.selectedModel === modelId) {
          const remainingModels = providersStore.models.filter(m => m.provider_type === activeSubTab.value && m.id !== modelId);
          if (remainingModels.length > 0) {
            config.selectedModel = remainingModels[0].id;
            await handleSave(activeSubTab.value);
          }
        }
        
        appStore.notify('Model deleted successfully', 'success');
      } catch (e) {
        appStore.notify('Failed to delete model', 'error');
      }
    }
  );
};

const handleResetModels = () => {
  triggerConfirmation(
    'Reset to Default Models',
    'Are you sure you want to reset all default models? Your custom-added models will not be deleted.',
    async () => {
      try {
        await providersStore.resetModels();
        
        // Ensure selected model is still valid
        const config = currentConfig.value;
        const providerModels = providersStore.models.filter(m => m.provider_type === activeSubTab.value);
        if (providerModels.length > 0 && !providerModels.some(m => m.id === config.selectedModel)) {
          config.selectedModel = providerModels[0].id;
          await handleSave(activeSubTab.value);
        }
        
        appStore.notify('Models reset to defaults successfully', 'success');
      } catch (e) {
        appStore.notify('Failed to reset models', 'error');
      }
    }
  );
};

const toggleModelEnabled = async (model: any) => {
  try {
    const isCurrentlyEnabled = model.enabled;
    const newEnabledState = !isCurrentlyEnabled;

    await providersStore.saveModel({
      id: model.id,
      providerType: model.provider_type,
      name: model.name,
      useCase: model.use_case || '',
      icon: model.icon || '🤖',
      isCustom: model.is_custom,
      enabled: newEnabledState,
    });

    // If the model was selected and we just disabled it, fallback to first available enabled model
    if (isCurrentlyEnabled) {
      const config = model.provider_type === 'openrouter' ? openrouterConfig.value 
                   : model.provider_type === 'groq' ? groqConfig.value 
                   : nvidiaConfig.value;
      if (config.selectedModel === model.id) {
        const remainingModels = providersStore.models.filter(
          m => m.provider_type === model.provider_type && m.id !== model.id && m.enabled
        );
        if (remainingModels.length > 0) {
          config.selectedModel = remainingModels[0].id;
          await handleSave(model.provider_type);
        }
      }
    }

    appStore.notify(`${model.name} is now ${newEnabledState ? 'enabled' : 'disabled'}`, 'success');
  } catch (e) {
    console.error('Failed to toggle model:', e);
    appStore.notify('Failed to update model status', 'error');
  }
};

// Model Diagnostics & Latency Parallel Testing Tab logic
const selectedTestModels = ref<string[]>([]);
const testTimeout = ref<number>(10);
const testRunning = ref(false);
const testStatuses = ref<Record<string, { status: 'idle' | 'testing' | 'success' | 'error'; message: string; duration?: number }>>({});

const handleSelectAllTestModels = (checked: boolean) => {
  if (checked) {
    selectedTestModels.value = providersStore.models.map(m => m.id);
  } else {
    selectedTestModels.value = [];
  }
};

const runParallelTests = async () => {
  if (selectedTestModels.value.length === 0) {
    appStore.notify('Please select at least one model to test', 'error');
    return;
  }

  testRunning.value = true;
  appStore.notify(`Starting parallel test for ${selectedTestModels.value.length} models...`, 'info');

  // Ensure all states are reset before parallel tests run
  selectedTestModels.value.forEach(modelId => {
    testStatuses.value[modelId] = { status: 'testing', message: 'Testing connection...' };
  });

  const testPromises = selectedTestModels.value.map(async (modelId) => {
    const model = providersStore.models.find(m => m.id === modelId);
    if (!model) return;

    // Resolve API key and endpoint for this provider type
    let apiKey = '';
    let endpoint = '';
    
    if (model.provider_type === 'openrouter') {
      apiKey = openrouterConfig.value.apiKey;
      endpoint = openrouterConfig.value.endpoint || 'https://openrouter.ai/api/v1/chat/completions';
    } else if (model.provider_type === 'groq') {
      apiKey = groqConfig.value.apiKey;
      endpoint = groqConfig.value.endpoint || 'https://api.groq.com/openai/v1/chat/completions';
    } else if (model.provider_type === 'nvidia') {
      apiKey = nvidiaConfig.value.apiKey;
      endpoint = nvidiaConfig.value.endpoint || 'https://integrate.api.nvidia.com/v1/chat/completions';
    }

    if (!apiKey) {
      testStatuses.value[modelId] = { 
        status: 'error', 
        message: 'API Key not saved in Settings yet' 
      };
      return;
    }

    const startTime = Date.now();

    try {
      await testProvider(apiKey, modelId, endpoint || undefined, testTimeout.value);
      const duration = Date.now() - startTime;
      testStatuses.value[modelId] = { 
        status: 'success', 
        message: 'Connected', 
        duration 
      };
    } catch (e: any) {
      const errorMsg = typeof e === 'string' ? e : e.message || 'Connection failed';
      // Clean up the error message to be highly readable
      const cleanMsg = errorMsg
        .replace(/Connection failed.*?:/i, '')
        .replace(/HTTP request error:.*?(Status: \d+|Connect error)/i, '$1')
        .trim();
      testStatuses.value[modelId] = { 
        status: 'error', 
        message: cleanMsg || 'Connection timed out or failed' 
      };
    }
  });

  await Promise.all(testPromises);
  testRunning.value = false;
  appStore.notify('All connection tests completed!', 'success');
};

const setAccentColor = (color: any) => {
  appStore.accentColor = color;
};

const setHighlightColor = (color: any) => {
  appStore.highlightColor = color;
};

// ─── Page Types Editor Local State ───
const selectedPageTypeId = ref<string | null>(null);
const editingPageType = ref<PageTypeDefinition | null>(null);

const newCheckName = ref('');
const newCheckWidgetType = ref<AuditWidgetType>('binary');
const newCheckPrompt = ref('');

const selectPageTypeForEdit = (id: string) => {
  if (id === 'new') {
    editingPageType.value = {
      id: 'pt_' + Math.random().toString(36).substring(2, 9),
      name: '',
      icon: '📝',
      inheritDefault: true,
      checks: []
    };
    selectedPageTypeId.value = 'new';
  } else {
    const pt = auditStore.getPageType(id);
    if (pt) {
      editingPageType.value = JSON.parse(JSON.stringify(pt));
      selectedPageTypeId.value = id;
    }
  }
};

const saveEditingPageType = async () => {
  if (!editingPageType.value) return;
  if (!editingPageType.value.name.trim()) {
    appStore.notify('Auditor name is required', 'error');
    return;
  }

  try {
    if (selectedPageTypeId.value === 'new') {
      await auditStore.addPageType(editingPageType.value);
      appStore.notify('Auditor created successfully', 'success');
    } else {
      await auditStore.updatePageType(editingPageType.value);
      appStore.notify('Auditor updated successfully', 'success');
    }
    cancelEditingPageType();
  } catch (err) {
    appStore.notify('Failed to save Auditor', 'error');
  }
};

const cancelEditingPageType = () => {
  selectedPageTypeId.value = null;
  editingPageType.value = null;
  newCheckName.value = '';
  newCheckWidgetType.value = 'binary';
  newCheckPrompt.value = '';
  cancelEditingCheck();
};

const deletePageType = (id: string) => {
  if (id === 'default') {
    appStore.notify('Cannot delete the default Auditor', 'error');
    return;
  }

  triggerConfirmation(
    'Delete Auditor',
    'Are you sure you want to delete this Auditor? Drafts assigned to it will fallback to General Auditor.',
    async () => {
      try {
        await auditStore.deletePageType(id);
        appStore.notify('Auditor deleted successfully', 'success');
      } catch (err) {
        appStore.notify('Failed to delete Auditor', 'error');
      }
    }
  );
};

const addCheckToEditing = () => {
  if (!editingPageType.value) return;
  if (!newCheckName.value.trim() || !newCheckPrompt.value.trim()) {
    appStore.notify('Check Name and AI Prompt instruction are required', 'error');
    return;
  }

  editingPageType.value.checks.push({
    id: 'chk_' + Math.random().toString(36).substring(2, 9),
    name: newCheckName.value.trim(),
    widgetType: newCheckWidgetType.value,
    prompt: newCheckPrompt.value.trim()
  });

  newCheckName.value = '';
  newCheckWidgetType.value = 'binary';
  newCheckPrompt.value = '';
  appStore.notify('Check added', 'success');
};

const removeCheckFromEditing = (checkId: string) => {
  if (!editingPageType.value) return;
  editingPageType.value.checks = editingPageType.value.checks.filter(c => c.id !== checkId);
};

// ─── Inline Checklist Parameter Editing State ───
const editingCheckId = ref<string | null>(null);
const editingCheckName = ref('');
const editingCheckWidgetType = ref<AuditWidgetType>('binary');
const editingCheckPrompt = ref('');

const startEditingCheck = (chk: AuditCheckDefinition) => {
  editingCheckId.value = chk.id;
  editingCheckName.value = chk.name;
  editingCheckWidgetType.value = chk.widgetType;
  editingCheckPrompt.value = chk.prompt;
};

const saveEditedCheck = () => {
  if (!editingPageType.value || !editingCheckId.value) return;
  if (!editingCheckName.value.trim() || !editingCheckPrompt.value.trim()) {
    appStore.notify('Check Name and AI Prompt instruction are required', 'error');
    return;
  }

  const check = editingPageType.value.checks.find(c => c.id === editingCheckId.value);
  if (check) {
    check.name = editingCheckName.value.trim();
    check.widgetType = editingCheckWidgetType.value;
    check.prompt = editingCheckPrompt.value.trim();
    appStore.notify('Parameter updated', 'success');
  }
  cancelEditingCheck();
};

const cancelEditingCheck = () => {
  editingCheckId.value = null;
  editingCheckName.value = '';
  editingCheckWidgetType.value = 'binary';
  editingCheckPrompt.value = '';
};
</script>

<template>
  <div class="modal-backdrop" @click="appStore.showSettingsModal = false">
    <div class="modal-content settings-modal" @click.stop>
      <div class="modal-header">
        <h2 class="settings-title">Settings</h2>
        <button class="btn btn-ghost btn-icon" @click="appStore.showSettingsModal = false">×</button>
      </div>

      <!-- Settings Premium Tabs -->
      <div class="settings-tabs">
        <button 
          :class="['settings-tab', { active: currentTab === 'appearance' }]"
          @click="currentTab = 'appearance'"
        >
          <span class="tab-icon">🎨</span> Appearance
        </button>
        <button 
          :class="['settings-tab', { active: currentTab === 'providers' }]"
          @click="currentTab = 'providers'"
        >
          <span class="tab-icon">🤖</span> AI Providers
        </button>
        <button 
          :class="['settings-tab', { active: currentTab === 'testing' }]"
          @click="currentTab = 'testing'"
        >
          <span class="tab-icon">🧪</span> Model Testing
        </button>
        <button 
          :class="['settings-tab', { active: currentTab === 'page_types' }]"
          @click="currentTab = 'page_types'"
        >
          <span class="tab-icon">🕵️‍♂️</span> Auditors
        </button>
      </div>

      <div class="modal-body settings-view">
        <!-- Appearance Settings Tab -->
        <div v-if="currentTab === 'appearance'" class="settings-tab-content">
          <div class="settings-container">
            <h3 style="margin-bottom: 8px; font-weight: 600; font-size: var(--font-size-lg);">Visual Environment</h3>
            <p class="text-sm text-secondary" style="margin-bottom: var(--space-5);">
              Customize the look and feel of your Refinery writing workspace.
            </p>
            
            <div class="appearance-grid">
              <!-- Left side: Theme, Accent, Highlights -->
              <div class="appearance-column">
                <div class="appearance-card card">
                  <h4 class="card-section-title">🎨 Theme & Accents</h4>
                  
                  <!-- Interface Theme -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Interface Theme</h5>
                      <p class="setting-desc">Choose application color mode</p>
                    </div>
                    <div class="theme-selector">
                      <button 
                        :class="['theme-btn', { active: appStore.theme === 'light' }]"
                        @click="appStore.theme = 'light'"
                        title="Light Mode"
                      >
                        <span class="theme-icon">☀️</span> Light
                      </button>
                      <button 
                        :class="['theme-btn', { active: appStore.theme === 'dark' }]"
                        @click="appStore.theme = 'dark'"
                        title="Dark Mode"
                      >
                        <span class="theme-icon">🌑</span> Dark
                      </button>
                      <button 
                        :class="['theme-btn', { active: appStore.theme === 'system' }]"
                        @click="appStore.theme = 'system'"
                        title="System Theme"
                      >
                        <span class="theme-icon">🖥️</span> System
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Accent Color Swatches -->
                  <div class="setting-item">
                    <div style="margin-bottom: var(--space-3);">
                      <h5 class="setting-label">Accent Theme</h5>
                      <p class="setting-desc">Change primary brand color and editor environments</p>
                    </div>
                    <div class="accent-swatches">
                      <button 
                        v-for="color in ['purple', 'emerald', 'ocean', 'amber', 'rose']" 
                        :key="color"
                        :class="['accent-swatch-btn', `accent-${color}`, { active: appStore.accentColor === color }]"
                        @click="setAccentColor(color)"
                        :title="color.charAt(0).toUpperCase() + color.slice(1)"
                      >
                        <span class="swatch-color"></span>
                        <span class="swatch-name">
                          {{ color === 'purple' ? 'Amethyst' : color === 'amber' ? 'Amber (Sepia)' : color.charAt(0).toUpperCase() + color.slice(1) }}
                        </span>
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Canvas Environment presets -->
                  <div class="setting-item">
                    <div style="margin-bottom: var(--space-3);">
                      <h5 class="setting-label">Canvas Environment</h5>
                      <p class="setting-desc">Choose comfortable reading backgrounds for different environments</p>
                    </div>
                    <div class="canvas-swatches">
                      <button 
                        v-for="bg in [
                          { id: 'default', name: 'Charcoal', color: '#111318', lightColor: '#F8F9FC' },
                          { id: 'dracula', name: 'Gothic Dracula', color: '#191622', lightColor: '#F6F3FC' },
                          { id: 'slate', name: 'Slate Blue', color: '#0F172A', lightColor: '#F1F5F9' },
                          { id: 'parchment', name: 'Warm Ivory', color: '#181411', lightColor: '#F5EEDB' },
                          { id: 'contrast', name: 'High Contrast', color: '#000000', lightColor: '#FFFFFF' }
                        ]"
                        :key="bg.id"
                        :class="['canvas-swatch-btn', `canvas-${bg.id}`, { active: appStore.canvasBg === bg.id }]"
                        @click="appStore.canvasBg = (bg.id as any)"
                        :title="bg.name"
                      >
                        <span class="canvas-color-indicator" :style="{ backgroundColor: appStore.theme === 'light' ? bg.lightColor : bg.color }"></span>
                        <span class="swatch-name">{{ bg.name }}</span>
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Selection Highlight Selector -->
                  <div class="setting-item">
                    <div style="margin-bottom: var(--space-3);">
                      <h5 class="setting-label">Text Selection Highlight</h5>
                      <p class="setting-desc">Color used for highlighting selected text in the editor</p>
                    </div>
                    <div class="highlight-selectors">
                      <button 
                        v-for="hl in [
                          { id: 'purple', name: 'Classic Lavender', bg: '#8B5CF6' },
                          { id: 'gold', name: 'Warm Gold', bg: '#F59E0B' },
                          { id: 'mint', name: 'Mint Green', bg: '#10B981' },
                          { id: 'sky', name: 'Ice Sky', bg: '#0EA5E9' }
                        ]"
                        :key="hl.id"
                        :class="['highlight-btn', `hl-${hl.id}`, { active: appStore.highlightColor === hl.id }]"
                        @click="setHighlightColor(hl.id)"
                      >
                        <span class="hl-indicator" :style="{ backgroundColor: hl.bg }"></span>
                        <span class="hl-name">{{ hl.name }}</span>
                        <span class="hl-preview" :style="{ background: `rgba(${hl.id === 'purple' ? '139, 92, 246' : hl.id === 'gold' ? '245, 158, 11' : hl.id === 'mint' ? '16, 185, 129' : '14, 165, 233'}, 0.2)` }">abc</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Right side: Typography, Layout Width, Zen Mode, and Zoom/Spellcheck -->
              <div class="appearance-column">
                <div class="appearance-card card">
                  <h4 class="card-section-title">✍️ Editor Layout & Zen Focus</h4>

                  <!-- Editor Typography Font Family -->
                  <div class="setting-item">
                    <div style="margin-bottom: var(--space-2);">
                      <h5 class="setting-label">Typography Font</h5>
                      <p class="setting-desc">Select the font family for the writing area</p>
                    </div>
                    <div class="font-selector-expanded">
                      <button 
                        v-for="f in [
                          { id: 'sans', name: 'Sans-Serif', family: 'var(--font-sans)' },
                          { id: 'serif', name: 'Serif', family: 'Georgia, serif' },
                          { id: 'mono', name: 'Monospace', family: 'var(--font-mono)' },
                          { id: 'slab', name: 'Slab Serif', family: 'Courier Prime, Courier, monospace' },
                          { id: 'geometric', name: 'Geometric', family: 'Outfit, Montserrat, sans-serif' }
                        ]"
                        :key="f.id"
                        :class="['font-btn-selector-expanded', { active: appStore.editorFont === f.id }]"
                        @click="appStore.editorFont = (f.id as any)"
                        :style="{ fontFamily: f.family }"
                      >
                        {{ f.name }}
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Editor Font Size Slider -->
                  <div class="setting-item">
                    <div class="flex-between" style="margin-bottom: var(--space-3);">
                      <div>
                        <h5 class="setting-label">Text Sizing</h5>
                        <p class="setting-desc">Scale the editor text size for absolute readability</p>
                      </div>
                      <span class="lh-value">{{ appStore.editorFontSize }}px</span>
                    </div>
                    <div class="zoom-slider-container">
                      <span class="zoom-edge-label">12px</span>
                      <input 
                        type="range" 
                        min="12" 
                        max="22" 
                        step="1" 
                        v-model.number="appStore.editorFontSize"
                        class="zoom-slider"
                      />
                      <span class="zoom-edge-label">22px</span>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Editor Line Height Slider -->
                  <div class="setting-item">
                    <div class="flex-between" style="margin-bottom: var(--space-3);">
                      <div>
                        <h5 class="setting-label">Line Spacing</h5>
                        <p class="setting-desc">Adjust vertical spacing between editor lines</p>
                      </div>
                      <span class="lh-value">{{ appStore.lineHeight.toFixed(1) }}</span>
                    </div>
                    <div class="zoom-slider-container">
                      <span class="zoom-edge-label">1.4</span>
                      <input 
                        type="range" 
                        min="1.4" 
                        max="1.8" 
                        step="0.1" 
                        v-model.number="appStore.lineHeight"
                        class="zoom-slider"
                      />
                      <span class="zoom-edge-label">1.8</span>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Editor Width Cozy Layout Centering -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Editor Columns</h5>
                      <p class="setting-desc">Cozy centered margin or wide fullscreen layout</p>
                    </div>
                    <div class="width-selector">
                      <button 
                        :class="['width-btn', { active: appStore.editorWidth === 'cozy' }]"
                        @click="appStore.editorWidth = 'cozy'"
                      >
                        🧘 Cozy Centered
                      </button>
                      <button 
                        :class="['width-btn', { active: appStore.editorWidth === 'wide' }]"
                        @click="appStore.editorWidth = 'wide'"
                      >
                        🖥️ Fullscreen Wide
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Zen Focus Mode -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Zen Focus Mode</h5>
                      <p class="setting-desc">Hide sidebars when typing. Restores on mouse movement.</p>
                    </div>
                    <label class="switch-container">
                      <input type="checkbox" v-model="appStore.zenFocusEnabled" class="switch-input" />
                      <span class="switch-slider"></span>
                    </label>
                  </div>
                </div>

                <!-- Zoom and Spellcheck settings card -->
                <div class="appearance-card card" style="margin-top: var(--space-4);">
                  <h4 class="card-section-title">⚙️ Workspace Settings</h4>

                  <!-- UI Layout Variant -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Interface Layout Style</h5>
                      <p class="setting-desc">Configure visual boundaries for headers and sidebars</p>
                    </div>
                    <div class="width-selector">
                      <button 
                        v-for="variant in [
                          { id: 'unified', name: 'Unified' },
                          { id: 'contrasted', name: 'Contrasted' },
                          { id: 'glassmorphic', name: 'Acrylic Glass' }
                        ]"
                        :key="variant.id"
                        :class="['width-btn', { active: appStore.uiLayoutVariant === variant.id }]"
                        @click="appStore.uiLayoutVariant = (variant.id as any)"
                        :title="variant.name"
                      >
                        {{ variant.name }}
                      </button>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Interface Zoom -->
                  <div class="setting-item">
                    <div class="flex-between" style="margin-bottom: var(--space-3);">
                      <div>
                        <h5 class="setting-label">Editor & Interface Zoom</h5>
                        <p class="setting-desc">Scale workspace text and elements</p>
                      </div>
                      <div class="flex items-center gap-3">
                        <span class="zoom-value">{{ Math.round(appStore.zoomFactor * 100) }}%</span>
                        <button class="btn btn-xs btn-outline" @click="appStore.resetZoom()">Reset</button>
                      </div>
                    </div>
                    <div class="zoom-slider-container">
                      <span class="zoom-edge-label">50%</span>
                      <input 
                        type="range" 
                        min="0.5" 
                        max="2.0" 
                        step="0.1" 
                        v-model.number="appStore.zoomFactor"
                        class="zoom-slider"
                      />
                      <span class="zoom-edge-label">200%</span>
                    </div>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- Spellchecker Toggle -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Editor Spellcheck</h5>
                      <p class="setting-desc">Enable browser-based spellcheck highlights</p>
                    </div>
                    <label class="switch-container">
                      <input type="checkbox" v-model="appStore.spellcheckEnabled" class="switch-input" />
                      <span class="switch-slider"></span>
                    </label>
                  </div>

                  <div class="setting-divider"></div>

                  <!-- GPU Hardware Acceleration Toggle -->
                  <div class="setting-item flex-between">
                    <div>
                      <h5 class="setting-label">Hardware Acceleration</h5>
                      <p class="setting-desc">Use GPU acceleration. Disable to resolve cursor flickering or blank screens.</p>
                    </div>
                    <label class="switch-container">
                      <input type="checkbox" v-model="appStore.gpuAccelerationEnabled" class="switch-input" />
                      <span class="switch-slider"></span>
                    </label>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- AI Provider Settings Tab -->
        <div v-else-if="currentTab === 'providers'" class="settings-tab-content">
          <div class="providers-split-layout">
            <!-- Left sidebar: Provider selection list -->
            <div class="providers-sidebar">
              <button 
                v-for="provider in providerList" 
                :key="provider.id"
                :class="['provider-sidebar-btn', { active: activeSubTab === provider.id }]"
                @click="activeSubTab = provider.id"
              >
                <div class="provider-btn-meta">
                  <span class="provider-btn-icon">{{ provider.icon }}</span>
                  <div class="provider-btn-text">
                    <span class="provider-btn-name">{{ provider.name }}</span>
                    <span class="provider-btn-status" :class="{ enabled: provider.enabled }">
                      {{ provider.enabled ? 'Active' : 'Inactive' }}
                    </span>
                  </div>
                </div>
              </button>
            </div>

            <!-- Right content area: Config form -->
            <div class="provider-config-pane">
              <div class="provider-pane-header">
                <span class="provider-pane-icon">{{ currentProviderDetails.icon }}</span>
                <div>
                  <h3 class="provider-pane-title">{{ currentProviderDetails.name }}</h3>
                  <p class="provider-pane-desc">{{ currentProviderDetails.desc }}</p>
                </div>
              </div>

              <div class="provider-form-grid">
                <!-- Column 1: Connection Details -->
                <div class="config-column">
                  <div class="form-field">
                    <label class="label">API Key</label>
                    <div class="password-input-wrapper">
                      <input
                        :type="showApiKey ? 'text' : 'password'"
                        class="input password-input"
                        v-model="currentConfig.apiKey"
                        :placeholder="currentProviderDetails.keyPlaceholder"
                        autocomplete="off"
                      />
                      <button 
                        type="button" 
                        class="password-toggle-btn"
                        @click="showApiKey = !showApiKey"
                      >
                        {{ showApiKey ? '👁️' : '🔒' }}
                      </button>
                    </div>
                    <p class="text-xs text-muted" style="margin-top: 4px;">
                      Stored securely in your OS credential store
                    </p>
                  </div>

                  <div class="form-field">
                    <label class="label">Endpoint</label>
                    <input class="input" v-model="currentConfig.endpoint" :placeholder="currentProviderDetails.defaultEndpoint" />
                  </div>
                </div>

                <!-- Column 2: Selection & Status -->
                <div class="config-column">
                  <div class="form-field">
                    <label class="label">Default Assisting Voice</label>
                    <select class="input select" v-model="currentConfig.selectedModel">
                      <option v-for="m in currentProviderDetails.models" :key="m" :value="m">{{ m }}</option>
                    </select>
                  </div>

                  <!-- Toggle to set active -->
                  <div class="form-field flex-between active-toggle-row">
                    <div>
                      <span class="label" style="margin: 0; font-weight: 500; font-size: var(--font-size-md);">Set as Active Provider</span>
                      <p class="text-xs text-muted" style="margin-top: 2px;">Use this provider for AI review and rewriting features</p>
                    </div>
                    <label class="switch-container">
                      <input 
                        type="checkbox" 
                        :checked="currentConfig.enabled"
                        @change="handleActiveToggle" 
                        class="switch-input" 
                      />
                      <span class="switch-slider"></span>
                    </label>
                  </div>
                </div>
              </div>

              <div class="provider-actions">
                <button class="btn btn-outline" @click="handleTest(activeSubTab)" :disabled="testing || !currentConfig.apiKey">
                  {{ testing ? 'Testing...' : 'Test Connection' }}
                </button>
                <button class="btn btn-primary" @click="handleSave(activeSubTab)" :disabled="!currentConfig.apiKey">
                  Save
                </button>
              </div>

              <div v-if="testResult" :class="['test-result', testResult.includes('successful') ? 'success' : 'error']">
                {{ testResult }}
              </div>

              <!-- Models Infrastructure Section -->
              <div class="models-infrastructure-section">
                <div class="setting-divider"></div>
                <div class="flex-between models-section-header">
                  <div>
                    <h4 class="setting-label">Models Infrastructure</h4>
                    <p class="setting-desc">Add, edit, or remove models for {{ currentProviderDetails.name }}</p>
                  </div>
                  <div class="flex gap-2">
                    <button class="btn btn-xs btn-outline" @click="handleResetModels">Reset Defaults</button>
                    <button class="btn btn-xs btn-primary" @click="openAddModel">+ Add Model</button>
                  </div>
                </div>

                <div class="models-grid">
                  <div 
                    v-for="model in providersStore.models.filter(m => m.provider_type === activeSubTab)" 
                    :key="model.id"
                    class="model-card"
                    :class="{ 
                      'selected-active-model': currentConfig.selectedModel === model.id,
                      'model-disabled': !model.enabled
                    }"
                  >
                    <div class="model-info-block">
                      <span class="model-card-icon">{{ model.icon || '🤖' }}</span>
                      <div class="model-text-block">
                        <div class="model-title-row">
                          <span class="model-card-name">{{ model.name }}</span>
                          <span class="model-badge badge-active" v-if="currentConfig.selectedModel === model.id">Selected</span>
                        </div>
                        <div class="model-card-id">{{ model.id }}</div>
                        <div class="model-card-desc" v-if="model.use_case">{{ model.use_case }}</div>
                      </div>
                    </div>
                    <div class="model-actions-block">
                      <button 
                        class="btn btn-xs btn-outline" 
                        @click="toggleModelEnabled(model)" 
                        :title="model.enabled ? 'Disable Model' : 'Enable Model'"
                      >
                        {{ model.enabled ? '🙈 Disable' : '👁️ Enable' }}
                      </button>
                      <button class="btn btn-xs btn-outline" @click="openEditModel(model)" title="Edit Model">✏️ Edit</button>
                      <button 
                        class="btn btn-xs btn-outline btn-text-error" 
                        @click="handleDeleteModel(model.id)" 
                        title="Delete Model"
                      >
                        🗑️ Delete
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Model Testing Diagnostics Tab -->
        <div v-else-if="currentTab === 'testing'" class="settings-tab-content diagnostics-tab-view">
          <div class="diagnostics-layout">
            <div class="diagnostics-sidebar-options">
              <h3 class="diagnostics-title">Parallel Diagnostics</h3>
              <p class="diagnostics-desc">Select configured AI models to run parallel connection & latency tests.</p>
              
              <div class="diagnostics-controls">
                <div class="form-field" style="margin-bottom: var(--space-4);">
                  <label class="label">Test Timeout</label>
                  <select class="input select" v-model.number="testTimeout">
                    <option :value="3">3 seconds</option>
                    <option :value="5">5 seconds</option>
                    <option :value="10">10 seconds</option>
                    <option :value="15">15 seconds</option>
                    <option :value="30">30 seconds</option>
                  </select>
                </div>

                <div class="select-all-row flex-between" style="background: var(--bg-secondary); padding: var(--space-3) var(--space-4); border-radius: var(--radius-md); border: 1px solid var(--border-subtle); margin-bottom: var(--space-4);">
                  <span class="text-sm font-medium">Select All Models</span>
                  <input 
                    type="checkbox" 
                    class="checkbox-test" 
                    :checked="selectedTestModels.length === providersStore.models.length && providersStore.models.length > 0"
                    @change="handleSelectAllTestModels(($event.target as HTMLInputElement).checked)"
                    style="width: 16px; height: 16px; cursor: pointer;"
                  />
                </div>

                <button 
                  class="btn btn-primary btn-run-diagnostics" 
                  :disabled="testRunning || selectedTestModels.length === 0"
                  @click="runParallelTests"
                  style="width: 100%; display: flex; align-items: center; justify-content: center; gap: 8px; padding: 12px; font-weight: 600;"
                >
                  {{ testRunning ? 'Running Tests...' : '🧪 Run Parallel Tests' }}
                </button>
              </div>
            </div>

            <div class="diagnostics-results-pane">
              <div 
                v-for="provider in ['openrouter', 'groq', 'nvidia']" 
                :key="provider"
                class="diagnostics-provider-group"
              >
                <!-- Render provider headers if they have configured models -->
                <div 
                  v-if="providersStore.models.some(m => m.provider_type === provider)"
                  class="diagnostics-provider-header"
                >
                  <span class="diagnostics-provider-icon">
                    {{ provider === 'openrouter' ? '🔗' : provider === 'groq' ? '⚡' : '💚' }}
                  </span>
                  <span class="diagnostics-provider-name">
                    {{ provider === 'openrouter' ? 'OpenRouter' : provider === 'groq' ? 'Groq' : 'NVIDIA NIM' }}
                  </span>
                  <span class="badge badge-neutral" style="margin-left: 8px; font-size: 10px; padding: 2px 6px;">
                    {{ providersStore.models.filter(m => m.provider_type === provider).length }} models
                  </span>
                </div>

                <div class="diagnostics-models-list">
                  <div 
                    v-for="model in providersStore.models.filter(m => m.provider_type === provider)" 
                    :key="model.id"
                    class="diagnostics-model-row"
                    :class="[
                      testStatuses[model.id]?.status || 'idle',
                      { 
                        'checked': selectedTestModels.includes(model.id),
                        'row-disabled': !model.enabled
                      }
                    ]"
                  >
                    <div class="diagnostics-model-meta flex items-center gap-3" style="flex: 1; min-width: 0;">
                      <input 
                        type="checkbox" 
                        class="checkbox-test" 
                        v-model="selectedTestModels" 
                        :value="model.id"
                        style="width: 16px; height: 16px; cursor: pointer; flex-shrink: 0;"
                      />
                      <span class="model-row-icon" style="font-size: 20px; flex-shrink: 0;">{{ model.icon || '🤖' }}</span>
                      <div class="model-row-text" style="flex: 1; min-width: 0;">
                        <div class="model-row-name" style="font-weight: 600; font-size: var(--font-size-sm); color: var(--text-primary);">{{ model.name }}</div>
                        <div class="model-row-id" style="font-family: var(--font-mono); font-size: 10px; color: var(--text-muted); text-overflow: ellipsis; overflow: hidden; white-space: nowrap;">{{ model.id }}</div>
                      </div>
                    </div>

                    <!-- Testing status indicator badge -->
                    <div class="diagnostics-status-col" style="flex-shrink: 0;">
                      <span v-if="!testStatuses[model.id] || testStatuses[model.id].status === 'idle'" class="test-badge-status status-idle">
                        Ready
                      </span>
                      <span v-else-if="testStatuses[model.id].status === 'testing'" class="test-badge-status status-testing">
                        <span class="spinner-small"></span> Testing...
                      </span>
                      <span v-else-if="testStatuses[model.id].status === 'success'" class="test-badge-status status-success">
                        Success ({{ testStatuses[model.id].duration }}ms)
                      </span>
                      <span 
                        v-else-if="testStatuses[model.id].status === 'error'" 
                        class="test-badge-status status-error"
                        :title="testStatuses[model.id].message"
                      >
                        Failed
                      </span>
                    </div>

                    <!-- Diagnostics action buttons -->
                    <div class="diagnostics-actions-col flex items-center gap-2" style="flex-shrink: 0; margin-left: 12px;">
                      <button 
                        class="btn btn-xs btn-outline btn-diagnostic-action" 
                        @click="toggleModelEnabled(model)" 
                        :title="model.enabled ? 'Disable Model' : 'Enable Model'"
                        style="padding: var(--space-1) var(--space-2); min-height: 28px;"
                      >
                        {{ model.enabled ? '🙈' : '👁️' }}
                      </button>
                      <button 
                        class="btn btn-xs btn-outline btn-text-error btn-diagnostic-action" 
                        @click="handleDeleteModel(model.id)" 
                        title="Delete Model"
                        style="padding: var(--space-1) var(--space-2); min-height: 28px;"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Auditors Settings Tab -->
        <div v-else-if="currentTab === 'page_types'" class="settings-tab-content page-types-tab-view">
          <!-- List Mode -->
          <div v-if="!editingPageType" class="page-types-list-layout">
            <div class="editor-header-area list-header-area">
              <div class="editor-header-title-block">
                <h3 class="editor-title">Auditors</h3>
                <p class="editor-desc">Define custom Auditors with specific AI-driven audit instructions and parameters.</p>
              </div>
              <button class="btn btn-primary btn-create-blueprint" @click="selectPageTypeForEdit('new')">
                ✨ Create Auditor
              </button>
            </div>

            <div class="blueprints-grid">
              <div 
                v-for="pt in auditStore.pageTypes" 
                :key="pt.id" 
                class="blueprint-card"
                :class="{ 'is-default': pt.id === 'default' }"
              >
                <div class="blueprint-card-header flex-between">
                  <div class="blueprint-title flex items-center gap-3">
                    <span class="blueprint-icon">{{ pt.icon || '📄' }}</span>
                    <div>
                      <div class="flex items-center gap-2" style="flex-wrap: wrap; margin-bottom: 2px;">
                        <h4 class="blueprint-name">{{ pt.name }}</h4>
                        <span v-if="pt.id === 'default'" class="system-default-tag">SYSTEM DEFAULT</span>
                      </div>
                      <span class="blueprint-id-badge">ID: {{ pt.id }}</span>
                    </div>
                  </div>
                  <div class="blueprint-badge">
                    {{ pt.checks.length }} parameters
                  </div>
                </div>

                <div class="blueprint-card-body">
                  <div class="blueprint-preview-checks">
                    <div v-for="chk in pt.checks.slice(0, 3)" :key="chk.id" class="blueprint-preview-check">
                      <span class="widget-type-dot" :class="chk.widgetType"></span>
                      <span class="check-name">{{ chk.name }}</span>
                    </div>
                    <div v-if="pt.checks.length > 3" class="blueprint-more-checks">
                      + {{ pt.checks.length - 3 }} more parameters
                    </div>
                    <div v-if="pt.inheritDefault && pt.id !== 'default'" class="blueprint-inheritance-note">
                      🔗 Inherits General Auditor parameters
                    </div>
                  </div>
                </div>

                <div class="blueprint-card-actions flex items-center gap-2">
                  <button 
                    class="btn btn-sm btn-outline btn-blueprint-action" 
                    @click="selectPageTypeForEdit(pt.id)"
                  >
                    ✏️ Edit Auditor
                  </button>
                  <button 
                    v-if="pt.id !== 'default'"
                    class="btn btn-sm btn-outline btn-text-error btn-blueprint-action" 
                    @click="deletePageType(pt.id)"
                  >
                    🗑️ Delete
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Editor Mode -->
          <div v-else class="page-types-editor-layout">
            <div class="editor-header-area">
              <button class="btn-back" @click="cancelEditingPageType">
                <span class="back-arrow">←</span> Back
              </button>
              <div class="editor-header-title-block">
                <h3 class="editor-title">
                  {{ selectedPageTypeId === 'new' ? 'New Auditor' : `Edit Auditor: ${editingPageType.name}` }}
                </h3>
                <p class="editor-desc">Configure the name, icon, and specific audit rules for this Auditor.</p>
              </div>
            </div>

            <div class="editor-grid">
              <!-- Sidebar Info Card -->
              <div class="editor-sidebar-column">
                <div class="card appearance-card">
                  <h4 class="card-section-title">Auditor Identity</h4>
                  
                  <div class="form-field" style="margin-bottom: var(--space-4);">
                    <label class="label">Auditor Name</label>
                    <input 
                      type="text" 
                      class="input" 
                      v-model="editingPageType.name" 
                      placeholder="e.g. SEO Auditor, Concision Expert" 
                    />
                  </div>

                  <div class="form-field" style="margin-bottom: var(--space-4);">
                    <label class="label">Emoji Icon</label>
                    <input 
                      type="text" 
                      class="input" 
                      v-model="editingPageType.icon" 
                      placeholder="e.g. 🕵️‍♂️, 📝, ⚡" 
                      style="width: 80px; text-align: center; font-size: 1.5rem;"
                    />
                  </div>

                  <div v-if="editingPageType.id !== 'default'" class="setting-item flex-between" style="padding-top: var(--space-2); flex-direction: row; display: flex; align-items: center; justify-content: space-between;">
                    <div>
                      <h5 class="setting-label">Inherit Defaults</h5>
                      <p class="setting-desc">Run General Auditor checks in addition to custom ones.</p>
                    </div>
                    <label class="switch-container">
                      <input type="checkbox" v-model="editingPageType.inheritDefault" class="switch-input" />
                      <span class="switch-slider"></span>
                    </label>
                  </div>
                </div>

                <div class="editor-sidebar-actions" style="margin-top: var(--space-4); display: flex; flex-direction: column; gap: var(--space-2);">
                  <button class="btn btn-primary" @click="saveEditingPageType" style="width: 100%;">
                    💾 Save Auditor
                  </button>
                  <button class="btn btn-outline" @click="cancelEditingPageType" style="width: 100%;">
                    Discard Changes
                  </button>
                </div>
              </div>

              <!-- Main Checklist Column -->
              <div class="editor-main-column">
                <div class="card appearance-card checklist-card-layout">
                  <h4 class="card-section-title flex-between">
                    <span>📋 Audit Parameters Checklist</span>
                    <span class="badge badge-accent">{{ editingPageType.checks.length }} defined</span>
                  </h4>

                  <!-- Checklist parameters -->
                  <div class="blueprint-checklist-scroller flex-grow">
                    <div v-if="editingPageType.checks.length === 0" class="checklist-empty-state">
                      <div class="empty-icon">📌</div>
                      <h5 class="empty-title">No parameters defined yet</h5>
                      <p class="empty-desc">Add at least one audit parameter below to configure how this Auditor analyzes drafts.</p>
                    </div>
                    <div v-else class="checklist-items">
                      <div v-for="(chk, idx) in editingPageType.checks" :key="chk.id" :class="['checklist-item-card', chk.widgetType]">
                        <!-- Inline Edit Mode -->
                        <div v-if="editingCheckId === chk.id" class="inline-edit-check-form">
                          <div class="inline-edit-row">
                            <div class="form-field">
                              <label class="label text-xs">Parameter Name</label>
                              <input 
                                type="text" 
                                class="input inline-edit-input" 
                                v-model="editingCheckName" 
                              />
                            </div>
                            <div class="form-field">
                              <label class="label text-xs">Type</label>
                              <select class="input select inline-edit-select" v-model="editingCheckWidgetType">
                                <option value="binary">Yes / No</option>
                                <option value="scale_1_5">1 to 5</option>
                                <option value="text_match">AI Extract</option>
                              </select>
                            </div>
                          </div>
                          <div class="form-field">
                            <label class="label text-xs">AI Prompt Instruction</label>
                            <textarea 
                              class="input textarea inline-edit-textarea" 
                              v-model="editingCheckPrompt" 
                              rows="5"
                            ></textarea>
                          </div>
                          <div class="flex justify-end gap-2 inline-edit-actions">
                            <button class="btn btn-xs btn-outline" @click="cancelEditingCheck">
                              Cancel
                            </button>
                            <button class="btn btn-xs btn-primary btn-save-inline" @click="saveEditedCheck">
                              ✓ Save
                            </button>
                          </div>
                        </div>

                        <!-- Read Mode -->
                        <div v-else>
                          <div class="checklist-item-header flex-between">
                            <div class="flex items-center gap-2 checklist-item-info">
                              <span class="checklist-item-number">#{{ idx + 1 }}</span>
                              <h5 class="checklist-item-name">{{ chk.name }}</h5>
                              <span class="badge widget-type-badge" :class="chk.widgetType">
                                {{ chk.widgetType === 'binary' ? 'Yes / No' : chk.widgetType === 'scale_1_5' ? '1 to 5 Scale' : 'AI Text Match' }}
                              </span>
                            </div>
                            <div class="checklist-item-actions flex items-center gap-2">
                              <button class="btn btn-ghost btn-icon btn-sm" @click="startEditingCheck(chk)" title="Edit Parameter">
                                ✏️
                              </button>
                              <button class="btn btn-ghost btn-icon btn-sm btn-text-error" @click="removeCheckFromEditing(chk.id)" title="Delete Parameter">
                                ×
                              </button>
                            </div>
                          </div>
                          <p class="checklist-item-prompt">{{ chk.prompt }}</p>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- Inheritance Banner -->
                  <div v-if="editingPageType.inheritDefault && editingPageType.id !== 'default'" class="checklist-inheritance-banner">
                    <span class="inheritance-icon">🔗</span>
                    <div>
                      <h6 class="inheritance-title">Inheriting Parameters from General Auditor:</h6>
                      <p class="text-xs text-secondary inheritance-desc">
                        Tone Consistency (Yes/No), Clarity & Concision (1-5 Scale), Core Takeaway (AI Text Match). These will run automatically.
                      </p>
                    </div>
                  </div>

                  <!-- Add Parameter Form -->
                  <div class="add-parameter-form">
                    <h5 class="add-parameter-title">➕ Add Audit Parameter</h5>
                    
                    <div class="form-grid-parameter">
                      <div class="form-field">
                        <label class="label text-xs">Parameter Name</label>
                        <input 
                          type="text" 
                          class="input input-sm" 
                          v-model="newCheckName" 
                          placeholder="e.g. SEO Focus Keyword, Active Voice" 
                        />
                      </div>
                      <div class="form-field">
                        <label class="label text-xs">Widget Response Type</label>
                        <select class="input select select-sm" v-model="newCheckWidgetType">
                          <option value="binary">Yes / No Indicator</option>
                          <option value="scale_1_5">1 to 5 Rating Scale</option>
                          <option value="text_match">AI Extract Text Match</option>
                        </select>
                      </div>
                    </div>

                    <div class="form-field">
                      <label class="label text-xs">AI Prompt Instruction (What should the AI check?)</label>
                      <textarea 
                        class="input textarea textarea-sm" 
                        v-model="newCheckPrompt" 
                        placeholder="e.g. Extract the primary focus keyword and evaluate if it appears in the first paragraph."
                        rows="2"
                      ></textarea>
                    </div>

                    <button class="btn btn-outline btn-add-param" @click="addCheckToEditing">
                      ➕ Add Parameter to Auditor Profile
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Sub-modal for Add/Edit Model -->
    <Transition name="fade">
      <div v-if="showModelModal" class="sub-modal-backdrop" @click="showModelModal = false">
        <div class="sub-modal-content" @click.stop>
          <div class="sub-modal-accent-bar"></div>
          <div class="modal-header sub-modal-header">
            <h3 class="modal-title-custom">{{ editingModel.isNew ? 'Add AI Model' : 'Edit AI Model' }}</h3>
            <button class="close-modal-btn" @click="showModelModal = false">×</button>
          </div>
          <div class="modal-body sub-modal-body">
            <div class="form-grid">
              <div class="form-field">
                <label class="label">Model ID</label>
                <input 
                  class="input" 
                  v-model="editingModel.id" 
                  :disabled="!editingModel.isNew" 
                  placeholder="e.g. openai/gpt-4o" 
                />
                <p class="text-xs text-muted" style="margin-top: 4px;" v-if="editingModel.isNew">
                  The exact identifier required by the provider's API.
                </p>
              </div>
              <div class="form-field">
                <label class="label">Friendly Name</label>
                <input class="input" v-model="editingModel.name" placeholder="e.g. GPT-4o" />
              </div>
              <div class="form-field">
                <label class="label">Use Case Description</label>
                <input class="input" v-model="editingModel.useCase" placeholder="e.g. Deep reasoning and creative novels" />
              </div>
              <div class="form-field">
                <label class="label">Icon / Emoji</label>
                <input class="input" v-model="editingModel.icon" placeholder="e.g. 🧠" style="max-width: 80px;" />
              </div>
            </div>
          </div>
          <div class="modal-footer sub-modal-footer">
            <button class="btn btn-outline" @click="showModelModal = false">Cancel</button>
            <button class="btn btn-primary" @click="handleSaveModel" :disabled="!editingModel.id || !editingModel.name">
              {{ editingModel.isNew ? 'Create Model' : 'Save Changes' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Custom confirmation modal -->
    <Transition name="fade">
      <div v-if="showConfirmModal" class="sub-modal-backdrop" @click="showConfirmModal = false">
        <div class="sub-modal-content confirmation-modal-content" @click.stop>
          <div class="sub-modal-accent-bar danger-accent"></div>
          <div class="modal-header sub-modal-header">
            <h3 class="modal-title-custom">{{ confirmTitle }}</h3>
            <button class="close-modal-btn" @click="showConfirmModal = false">×</button>
          </div>
          <div class="modal-body sub-modal-body text-center" style="padding: var(--space-6) var(--space-5);">
            <p class="text-md" style="color: var(--text-primary); margin: 0; line-height: 1.5; text-align: center;">
              {{ confirmMessage }}
            </p>
          </div>
          <div class="modal-footer sub-modal-footer">
            <button class="btn btn-outline" @click="showConfirmModal = false">Cancel</button>
            <button class="btn btn-primary btn-danger-custom" @click="handleConfirmAction">
              Confirm
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.65); display: flex; align-items: center; justify-content: center; z-index: 1000; backdrop-filter: blur(4px); }
.modal-content { background: var(--bg-primary); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); width: 96vw; max-width: 1400px; height: 90vh; max-height: 900px; display: flex; flex-direction: column; overflow: hidden; box-shadow: var(--shadow-xl); }
.modal-header { padding: var(--space-4) var(--space-5); border-bottom: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: space-between; flex-shrink: 0; background: var(--bg-secondary); }
.modal-body { flex: 1; overflow-y: auto; padding: 0; }

.settings-title { font-size: var(--font-size-lg); font-weight: 600; }
.settings-container { max-width: 100%; padding: var(--space-6); }

/* Settings Tab Container & Tab Buttons */
.settings-tabs {
  display: flex;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  padding: 0 var(--space-5);
  gap: var(--space-4);
  flex-shrink: 0;
}
.settings-tab {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) 4px;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.settings-tab:hover {
  color: var(--text-primary);
}
.settings-tab.active {
  color: var(--accent-muted);
  border-bottom-color: var(--accent-primary);
}
.tab-icon {
  font-size: 14px;
}

.settings-tab-content {
  height: 100%;
}

.card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-5);
}

/* Appearance Tab Styles */
.setting-item {
  padding: var(--space-2) 0;
}
.setting-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: var(--space-4) 0;
}
.setting-label {
  font-size: var(--font-size-md);
  font-weight: 500;
  color: var(--text-primary);
}
.setting-desc {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin-top: 2px;
}

.flex-between {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
}

.gap-3 { gap: var(--space-3); }

.zoom-value {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
}

.zoom-slider-container {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
.zoom-edge-label {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  width: 32px;
}
.zoom-edge-label:last-child {
  text-align: right;
}
.zoom-slider {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  border-radius: 2px;
  background: var(--bg-tertiary);
  outline: none;
}
.zoom-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent-primary);
  cursor: pointer;
  transition: transform var(--transition-fast);
}
.zoom-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

/* Custom Premium Toggle Switch */
.switch-container {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
}
.switch-input {
  opacity: 0;
  width: 0;
  height: 0;
}
.switch-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  transition: all var(--transition-normal);
}
.switch-slider:before {
  position: absolute;
  content: "";
  height: 12px;
  width: 12px;
  left: 3px;
  bottom: 3px;
  background-color: var(--text-secondary);
  border-radius: 50%;
  transition: all var(--transition-normal);
}
.switch-input:checked + .switch-slider {
  background-color: var(--accent-subtle);
  border-color: var(--accent-primary);
}
.switch-input:checked + .switch-slider:before {
  transform: translateX(16px);
  background-color: var(--accent-muted);
}

/* ─── Appearance Customizer Settings Grid & Selectors ─── */
.canvas-swatches {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
.canvas-swatch-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 8px 12px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.canvas-swatch-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-muted);
  background: var(--bg-hover);
  transform: translateY(-1px);
}
.canvas-swatch-btn.active {
  color: var(--text-primary);
  background: var(--accent-subtle);
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 1px var(--accent-primary);
}
.canvas-color-indicator {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid var(--border-subtle);
}

.font-selector-expanded {
  display: flex;
  flex-wrap: wrap;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 4px;
  gap: 4px;
  margin-top: var(--space-2);
}
.font-btn-selector-expanded {
  flex: 1;
  min-width: 105px;
  padding: 8px 10px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  text-align: center;
  transition: all var(--transition-fast);
  white-space: nowrap;
}
.font-btn-selector-expanded:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.02);
}
html[data-theme="light"] .font-btn-selector-expanded:hover {
  background: rgba(0, 0, 0, 0.02);
}
.font-btn-selector-expanded.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
  border-color: var(--border-subtle);
}

.appearance-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-5);
  align-items: start;
}
@media (max-width: 900px) {
  .appearance-grid {
    grid-template-columns: 1fr;
  }
}
.card-section-title {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 0;
  margin-bottom: var(--space-4);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.setting-label {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
  margin: 0;
}
.lh-value {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--accent-primary);
  background: var(--accent-subtle);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}
.accent-swatches {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
.accent-swatch-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 8px 12px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.accent-swatch-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-muted);
  background: var(--bg-hover);
  transform: translateY(-1px);
}
.accent-swatch-btn.active {
  color: var(--text-primary);
  background: var(--accent-subtle);
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 1px var(--accent-primary);
}
.swatch-color {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid rgba(0, 0, 0, 0.1);
}
.accent-purple .swatch-color { background-color: #8B5CF6; }
.accent-emerald .swatch-color { background-color: #10B981; }
.accent-ocean .swatch-color { background-color: #0EA5E9; }
.accent-amber .swatch-color { background-color: #F59E0B; }
.accent-rose .swatch-color { background-color: #EC4899; }

.highlight-selectors {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
.highlight-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 8px 10px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.highlight-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-muted);
  background: var(--bg-hover);
}
.highlight-btn.active {
  color: var(--text-primary);
  background: var(--accent-subtle);
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 1px var(--accent-primary);
}
.hl-indicator {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  flex-shrink: 0;
}
.hl-name {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hl-preview {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.font-selector {
  display: flex;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 3px;
  gap: 2px;
}
.font-btn-selector {
  padding: 6px 12px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.font-btn-selector:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.02);
}
html[data-theme="light"] .font-btn-selector:hover {
  background: rgba(0, 0, 0, 0.02);
}
.font-btn-selector.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
  border-color: var(--border-subtle);
}

.width-selector {
  display: flex;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 3px;
  gap: 2px;
}
.width-btn {
  padding: 6px 10px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.width-btn:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.02);
}
html[data-theme="light"] .width-btn:hover {
  background: rgba(0, 0, 0, 0.02);
}
.width-btn.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
  border-color: var(--border-subtle);
}

/* AI Providers Split Pane Styles */
.providers-split-layout {
  display: flex;
  height: 100%;
  min-height: 480px;
}
.providers-sidebar {
  width: 220px;
  border-right: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  padding: var(--space-4) var(--space-3);
  gap: var(--space-1);
  flex-shrink: 0;
}
.provider-sidebar-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--space-3) var(--space-4);
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
}
.provider-sidebar-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.provider-sidebar-btn.active {
  background: var(--accent-subtle);
  color: var(--text-primary);
}
.provider-btn-meta {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.provider-btn-icon {
  font-size: 18px;
}
.provider-btn-text {
  display: flex;
  flex-direction: column;
}
.provider-btn-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
}
.provider-btn-status {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin-top: 1px;
}
.provider-btn-status.enabled {
  color: var(--color-success);
  font-weight: 500;
}

.provider-config-pane {
  flex: 1;
  padding: var(--space-6);
  overflow-y: auto;
  background: var(--bg-primary);
}
.provider-pane-header {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  margin-bottom: var(--space-5);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}
.provider-pane-icon {
  font-size: 28px;
}
.provider-pane-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
}
.provider-pane-desc {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin-top: 2px;
}

.form-grid { display: flex; flex-direction: column; gap: var(--space-4); }
.form-field { display: flex; flex-direction: column; }

/* Password visibility input eye style */
.password-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}
.password-input {
  width: 100%;
  padding-right: 36px !important;
}
.password-toggle-btn {
  position: absolute;
  right: 12px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
}
.password-toggle-btn:hover {
  color: var(--text-secondary);
}

.active-toggle-row {
  background: var(--bg-secondary);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  margin-top: var(--space-2);
}

.provider-actions {
  display: flex;
  gap: var(--space-3);
  margin-top: var(--space-6);
  justify-content: flex-end;
}

.test-result {
  margin-top: var(--space-4);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-family: var(--font-mono);
  border: 1px solid transparent;
}
.test-result.success { background: var(--color-success-bg); color: var(--color-success); border-color: var(--color-success-border); }
.test-result.error { background: var(--color-error-bg); color: var(--color-error); border-color: var(--color-error-border); }

/* Theme Selector Styles */
.theme-selector {
  display: flex;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 3px;
  gap: 2px;
}
.theme-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 6px 12px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.theme-btn:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.02);
}
html[data-theme="light"] .theme-btn:hover {
  background: rgba(0, 0, 0, 0.02);
}
.theme-btn.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
  border-color: var(--border-subtle);
}
.theme-icon {
  font-size: 14px;
}

/* Models Infrastructure & Forms Redesign */
.provider-form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-5);
  margin-bottom: var(--space-4);
}
.config-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.models-infrastructure-section {
  margin-top: var(--space-6);
  display: flex;
  flex-direction: column;
}
.models-section-header {
  margin-bottom: var(--space-4);
}
.models-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
  margin-top: var(--space-3);
}
.model-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  justify-content: space-between;
  transition: all var(--transition-normal);
}
.model-card:hover {
  border-color: var(--accent-border);
  background: var(--bg-hover);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}
.model-card.selected-active-model {
  border-color: var(--accent-primary);
  background: var(--accent-subtle);
  box-shadow: 0 0 0 1px var(--accent-primary);
}
.model-card.model-disabled {
  opacity: 0.55;
  filter: grayscale(0.2);
}
.model-card.model-disabled:hover {
  opacity: 0.85;
  filter: none;
}
.model-info-block {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  width: 100%;
}
.model-card-icon {
  font-size: 24px;
  line-height: 1;
}
.model-text-block {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
  min-width: 0;
}
.model-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}
.model-card-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.model-badge {
  font-size: 9px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 8px;
  line-height: 1;
}
.badge-custom {
  background: var(--accent-muted);
  color: #ffffff;
}
.badge-active {
  background: var(--color-success-bg);
  color: var(--color-success);
  border: 1px solid var(--color-success-border);
}
.model-card-id {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  word-break: break-all;
}
.model-card-desc {
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
  margin-top: 2px;
}
.model-actions-block {
  display: flex;
  gap: var(--space-2);
  justify-content: flex-end;
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-2);
  margin-top: auto;
  width: 100%;
}
.btn-text-error:hover {
  color: var(--color-error) !important;
  background: var(--color-error-bg) !important;
  border-color: var(--color-error-border) !important;
}

/* Sub-modal Overlay Styles */
.sub-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(10px);
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
}
.sub-modal-content {
  width: 95vw;
  max-width: 460px;
  background: var(--bg-dropdown);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  box-shadow: var(--shadow-2xl);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.sub-modal-accent-bar {
  height: 4px;
  background: var(--accent-primary);
  width: 100%;
}
.sub-modal-accent-bar.danger-accent {
  background: var(--color-error, #ef4444);
}
.btn-danger-custom {
  background: var(--color-error, #ef4444) !important;
  border-color: var(--color-error, #ef4444) !important;
  color: #fff !important;
}
.btn-danger-custom:hover {
  background: #dc2626 !important;
  border-color: #dc2626 !important;
}
.sub-modal-header {
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
}
.sub-modal-body {
  padding: var(--space-5);
  max-height: 60vh;
  overflow-y: auto;
}
.sub-modal-footer {
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
}

/* Model Testing Diagnostics Tab Styles */
.diagnostics-tab-view {
  height: 100%;
}
.diagnostics-layout {
  display: flex;
  height: 100%;
  min-height: 480px;
}
.diagnostics-sidebar-options {
  width: 260px;
  border-right: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  padding: var(--space-6) var(--space-5);
  gap: var(--space-4);
  flex-shrink: 0;
}
.diagnostics-title {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}
.diagnostics-desc {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
  margin: 0;
}
.diagnostics-controls {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  margin-top: var(--space-2);
}
.diagnostics-results-pane {
  flex: 1;
  padding: var(--space-6);
  overflow-y: auto;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}
.diagnostics-provider-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.diagnostics-provider-header {
  display: flex;
  align-items: center;
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-subtle);
  padding-bottom: var(--space-2);
}
.diagnostics-provider-icon {
  margin-right: var(--space-2);
  font-size: 16px;
}
.diagnostics-provider-name {
  letter-spacing: -0.01em;
}
.diagnostics-models-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.diagnostics-model-row {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-3) var(--space-4);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  transition: all var(--transition-fast);
}
.diagnostics-model-row:hover {
  border-color: var(--accent-border);
  background: var(--bg-hover);
}
.diagnostics-model-row.checked {
  border-color: rgba(139, 92, 246, 0.3);
  background: rgba(139, 92, 246, 0.02);
}
.diagnostics-model-row.testing {
  border-color: var(--accent-primary);
  background: var(--accent-subtle);
  box-shadow: 0 0 0 1px var(--accent-primary);
}
.diagnostics-model-row.success {
  border-color: var(--color-success-border);
  background: rgba(16, 185, 129, 0.03);
}
.diagnostics-model-row.error {
  border-color: var(--color-error-border);
  background: rgba(239, 68, 68, 0.03);
}
.diagnostics-model-row.row-disabled {
  opacity: 0.55;
}
.diagnostics-model-row.row-disabled:hover {
  opacity: 0.85;
}
.btn-diagnostic-action {
  opacity: 0.5;
  border-color: transparent;
  background: transparent;
  transition: all var(--transition-fast);
}
.diagnostics-model-row:hover .btn-diagnostic-action {
  opacity: 0.8;
  border-color: var(--border-subtle);
  background: var(--bg-secondary);
}
.diagnostics-model-row:hover .btn-diagnostic-action:hover {
  opacity: 1;
  background: var(--bg-tertiary);
  border-color: var(--accent-border);
}
.checkbox-test {
  accent-color: var(--accent-primary);
}

/* Diagnostics Status Badge Styles */
.test-badge-status {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  white-space: nowrap;
}
.status-idle {
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border: 1px solid var(--border-subtle);
}
.status-testing {
  background: var(--accent-subtle);
  color: var(--accent-muted);
  border: 1px solid var(--accent-primary);
  animation: pulse-badge 1.5s infinite ease-in-out;
}
.status-success {
  background: var(--color-success-bg);
  color: var(--color-success);
  border: 1px solid var(--color-success-border);
}
.status-error {
  background: var(--color-error-bg);
  color: var(--color-error);
  border: 1px solid var(--color-error-border);
  cursor: help;
}

/* Spinner and micro-animations */
.spinner-small {
  width: 10px;
  height: 10px;
  border: 2px solid var(--accent-muted);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin-indicator 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin-indicator {
  to { transform: rotate(360deg); }
}
@keyframes pulse-badge {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

/* ─── Page Types settings tab styles ─── */
.page-types-tab-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.page-types-list-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.blueprints-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}
.blueprint-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 200px;
  position: relative;
  transition: all var(--transition-medium);
}
.blueprint-card:hover {
  transform: translateY(-2px);
  border-color: var(--accent-border);
  box-shadow: var(--shadow-md);
}
.blueprint-card.is-default {
  border-color: rgba(139, 92, 246, 0.2);
  background: linear-gradient(135deg, var(--bg-secondary), rgba(139, 92, 246, 0.02));
}
.blueprint-card.is-default::before {
  display: none;
}
.system-default-tag {
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: var(--accent-primary, #8b5cf6);
  background: var(--accent-subtle, rgba(139, 92, 246, 0.1));
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  line-height: 1;
}
.blueprint-icon {
  font-size: 1.8rem;
}
.blueprint-name {
  font-weight: 600;
  margin: 0;
  font-size: 1rem;
  color: var(--text-primary);
}
.blueprint-id-badge {
  font-size: 0.7rem;
  color: var(--text-muted);
}
.blueprint-badge {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
  border: 1px solid var(--border-subtle);
  background: var(--bg-primary);
  display: inline-flex;
  align-items: center;
}
.blueprint-card-body {
  margin: var(--space-4) 0;
  flex-grow: 1;
}
.blueprint-preview-checks {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.blueprint-preview-check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  color: var(--text-secondary);
}
.widget-type-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.widget-type-dot.binary {
  background: var(--color-success, #10b981);
}
.widget-type-dot.scale_1_5 {
  background: var(--color-warning, #f59e0b);
}
.widget-type-dot.text_match {
  background: var(--accent-primary, #8b5cf6);
}
.blueprint-more-checks {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-style: italic;
  padding-left: 14px;
}
.blueprint-inheritance-note {
  font-size: 0.75rem;
  color: var(--accent-primary);
  background: var(--accent-subtle);
  padding: 2px 8px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  margin-top: 4px;
}
.blueprint-card-actions {
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-3);
  margin-top: var(--space-2);
}
.btn-blueprint-action {
  flex: 1;
  text-align: center;
  justify-content: center;
}

/* Page Blueprint Editor Layout */
.page-types-editor-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  height: 100%;
}
.editor-grid {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--space-4);
  align-items: start;
}
.editor-sidebar-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.editor-main-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.checklist-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
  background: rgba(255, 255, 255, 0.01);
  border: 1px dashed var(--border-subtle);
  border-radius: var(--radius-lg);
  color: var(--text-muted);
}
.badge.widget-type-badge.binary {
  background: rgba(16, 185, 129, 0.08);
  color: var(--color-success, #10b981);
  border: 1px solid rgba(16, 185, 129, 0.2);
}
.badge.widget-type-badge.scale_1_5 {
  background: rgba(245, 158, 11, 0.08);
  color: var(--color-warning, #f59e0b);
  border: 1px solid rgba(245, 158, 11, 0.2);
}
.badge.widget-type-badge.text_match {
  background: rgba(139, 92, 246, 0.08);
  color: var(--accent-primary, #8b5cf6);
  border: 1px solid rgba(139, 92, 246, 0.2);
}
.blueprint-checklist-scroller::-webkit-scrollbar {
  width: 4px;
}
.blueprint-checklist-scroller::-webkit-scrollbar-track {
  background: transparent;
}
.blueprint-checklist-scroller::-webkit-scrollbar-thumb {
  background: var(--border-subtle);
  border-radius: 4px;
}

/* ─── Premium Page Types Tab Styles ─── */
.page-types-tab-view {
  padding: var(--space-6);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  height: auto;
}
.page-types-list-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.blueprints-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}
.blueprint-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 200px;
  position: relative;
  transition: all var(--transition-medium);
}
.blueprint-card:hover {
  transform: translateY(-2px);
  border-color: var(--accent-border);
  box-shadow: var(--shadow-md);
}
.blueprint-card.is-default {
  border-color: rgba(139, 92, 246, 0.2);
  background: linear-gradient(135deg, var(--bg-secondary), rgba(139, 92, 246, 0.02));
}
.blueprint-card.is-default::before {
  display: none;
}
.system-default-tag {
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: var(--accent-primary, #8b5cf6);
  background: var(--accent-subtle, rgba(139, 92, 246, 0.1));
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  line-height: 1;
}
.blueprint-icon {
  font-size: 1.8rem;
}
.blueprint-name {
  font-weight: 600;
  margin: 0;
  font-size: 1rem;
  color: var(--text-primary);
}
.blueprint-id-badge {
  font-size: 0.7rem;
  color: var(--text-muted);
}
.blueprint-badge {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
  border: 1px solid var(--border-subtle);
  background: var(--bg-primary);
  display: inline-flex;
  align-items: center;
}
.blueprint-card-body {
  margin: var(--space-4) 0;
  flex-grow: 1;
}
.blueprint-preview-checks {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.blueprint-preview-check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  color: var(--text-secondary);
}
.widget-type-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.widget-type-dot.binary {
  background: var(--color-success, #10b981);
}
.widget-type-dot.scale_1_5 {
  background: var(--color-warning, #f59e0b);
}
.widget-type-dot.text_match {
  background: var(--accent-primary, #8b5cf6);
}
.blueprint-more-checks {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-style: italic;
  padding-left: 14px;
}
.blueprint-inheritance-note {
  font-size: 0.75rem;
  color: var(--accent-primary);
  background: var(--accent-subtle);
  padding: 2px 8px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  margin-top: 4px;
}
.blueprint-card-actions {
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-3);
  margin-top: var(--space-2);
}
.btn-blueprint-action {
  flex: 1;
  text-align: center;
  justify-content: center;
}

/* Page Blueprint Editor Layout */
.page-types-editor-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  height: auto;
}
.editor-grid {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--space-5);
  align-items: start;
}
.editor-sidebar-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.editor-main-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* Elegant Sub-Header Styling */
.editor-header-area {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-2) 0 var(--space-4) 0;
  border-bottom: 1px solid var(--border-subtle);
  margin-bottom: var(--space-4);
  background: transparent;
}
.list-header-area {
  justify-content: space-between;
}
.btn-back {
  border: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  padding: 6px 14px;
  font-weight: 600;
  font-size: var(--font-size-xs);
  transition: all var(--transition-fast);
  color: var(--text-secondary);
  display: inline-flex;
  align-items: center;
  height: 32px;
  gap: 6px;
  cursor: pointer;
}
.btn-back:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-strong);
}
.btn-back .back-arrow {
  transition: transform var(--transition-fast, 0.15s) ease;
  font-size: 1.1em;
}
.btn-back:hover .back-arrow {
  transform: translateX(-2.5px);
}
.editor-header-title-block .editor-title {
  font-size: var(--font-size-md);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 var(--space-1) 0;
  letter-spacing: -0.015em;
}
.editor-header-title-block .editor-desc {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin: 0;
  line-height: 1.4;
}

/* Premium Checklist Layout & Card Styles */
.checklist-card-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.blueprint-checklist-scroller {
  /* Natural expansion to leverage SettingsModal's main scrollbar */
  margin-bottom: var(--space-4);
  padding-right: var(--space-1);
}
.checklist-empty-state {
  text-align: center;
  padding: var(--space-6) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}
.checklist-empty-state .empty-icon {
  font-size: 2rem;
  margin-bottom: var(--space-2);
}
.checklist-empty-state .empty-title {
  font-weight: 600;
  margin-bottom: var(--space-1);
}
.checklist-empty-state .empty-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
  max-width: 250px;
  margin: 0 auto;
}
.checklist-items {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.checklist-item-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-left: 4px solid var(--border-default);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}
.checklist-item-card.binary {
  border-left-color: var(--color-success, #10b981);
}
.checklist-item-card.scale_1_5 {
  border-left-color: var(--color-warning, #f59e0b);
}
.checklist-item-card.text_match {
  border-left-color: var(--accent-primary, #8b5cf6);
}
.checklist-item-card:hover {
  border-color: var(--border-strong);
  transform: translateX(2px);
  background: var(--bg-hover);
}
.checklist-item-number {
  color: var(--text-muted);
  font-size: 0.75rem;
  font-weight: 700;
  font-family: var(--font-mono);
}
.checklist-item-name {
  font-weight: 600;
  margin: 0;
  font-size: 0.85rem;
  color: var(--text-primary);
}
.widget-type-badge {
  font-size: 0.7rem;
  padding: 2px 6px;
}
.checklist-item-prompt {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin: var(--space-2) 0 0 0;
  line-height: 1.4;
}
.checklist-item-actions button {
  opacity: 0.6;
  transition: opacity var(--transition-fast);
}
.checklist-item-card:hover .checklist-item-actions button {
  opacity: 1;
}

/* Inheritance Banner Styles */
.checklist-inheritance-banner {
  background: rgba(139, 92, 246, 0.04);
  border: 1px dashed rgba(139, 92, 246, 0.25);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  display: flex;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
  align-items: flex-start;
}
.checklist-inheritance-banner .inheritance-icon {
  font-size: 1.1rem;
}
.checklist-inheritance-banner .inheritance-title {
  font-weight: 600;
  margin: 0 0 2px 0;
  font-size: 0.8rem;
  color: var(--text-primary);
}
.checklist-inheritance-banner .inheritance-desc {
  margin: 0;
  font-size: 0.75rem;
  line-height: 1.4;
}

/* Add Parameter Form Styles */
.add-parameter-form {
  border-top: 1px solid var(--border-subtle);
  padding-top: var(--space-4);
  background: var(--bg-tertiary);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
}
.add-parameter-title {
  font-weight: 600;
  margin: 0 0 var(--space-3) 0;
  font-size: 0.85rem;
}
.form-grid-parameter {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}
.input-sm, .select-sm {
  padding: 6px 12px;
  min-height: 32px;
  font-size: 0.8rem;
}
.textarea-sm {
  resize: none;
  padding: 8px 12px;
  font-size: 0.8rem;
}
.btn-add-param {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 34px;
  padding: 6px var(--space-4);
  font-size: 0.8rem;
  font-weight: 600;
}

/* Inline Edit Form Styles */
.inline-edit-check-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.inline-edit-row {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: var(--space-3);
}
.inline-edit-input, .inline-edit-select {
  padding: 6px 12px;
  min-height: 32px;
  font-size: 0.8rem;
}
.inline-edit-textarea {
  resize: vertical;
  min-height: 120px;
  padding: 10px 12px;
  font-size: 0.85rem;
  line-height: 1.5;
}
.inline-edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}
.btn-save-inline {
  padding: 6px 14px;
  font-size: 0.8rem;
  min-height: 32px;
  font-weight: 600;
}
</style>

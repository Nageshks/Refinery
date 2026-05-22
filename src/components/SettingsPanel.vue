<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useProvidersStore } from '../stores/providers';
import { useAppStore } from '../stores/app';
import { Store } from '@tauri-apps/plugin-store';
import { testProvider, setActiveProvider } from '../composables/useTauri';

const providersStore = useProvidersStore();
const appStore = useAppStore();

const currentTab = ref<'appearance' | 'providers'>('appearance');
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
  if (activeSubTab.value === 'nvidia') {
    return {
      name: 'NVIDIA NIM',
      icon: '💚',
      desc: 'Access high-performance NVIDIA NIM models optimized for local and cloud deployment',
      keyPlaceholder: 'nvapi-...',
      defaultEndpoint: 'https://integrate.api.nvidia.com/v1/chat/completions',
      models: [
        'nvidia/nemotron-3-nano-omni-30b-a3b-reasoning',
        'deepseek-ai/deepseek-v4-flash',
        'deepseek-ai/deepseek-v4-pro',
        'mistralai/mistral-medium-3.5-128b',
        'z-ai/glm-5.1',
        'moonshotai/kimi-k2.6'
      ]
    };
  }
  if (activeSubTab.value === 'groq') {
    return {
      name: 'Groq',
      icon: '⚡',
      desc: 'Access ultra-fast Llama and GPT-OSS models via Groq API',
      keyPlaceholder: 'gsk_...',
      defaultEndpoint: 'https://api.groq.com/openai/v1/chat/completions',
      models: [
        'openai/gpt-oss-120b',
        'llama-3.1-8b-instant',
        'llama-3.3-70b-versatile',
        'meta-llama/llama-4-scout-17b-16e-instruct'
      ]
    };
  }
  return {
    name: 'OpenRouter',
    icon: '🔗',
    desc: 'Access multiple AI models through a single OpenRouter API key',
    keyPlaceholder: 'sk-or-...',
    defaultEndpoint: 'https://openrouter.ai/api/v1/chat/completions',
    models: [
      'openai/gpt-4o-mini',
      'meta-llama/llama-3.3-70b-instruct',
      'z-ai/glm-4.5-air',
      'openrouter/owl-alpha',
      'google/gemma-4-31b-it',
      'openai/gpt-oss-120b',
      'deepseek/deepseek-v4-flash'
    ]
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
      </div>

      <div class="modal-body settings-view">
        <!-- Appearance Settings Tab -->
        <div v-if="currentTab === 'appearance'" class="settings-tab-content">
          <div class="settings-container">
            <h3 style="margin-bottom: 8px; font-weight: 600; font-size: var(--font-size-lg);">Visual Environment</h3>
            <p class="text-sm text-secondary" style="margin-bottom: var(--space-5);">
              Customize the look and feel of your Refinery writing workspace.
            </p>
            
            <div class="appearance-card card">
              <!-- Interface Theme -->
              <div class="setting-item flex-between">
                <div>
                  <h4 class="setting-label">Interface Theme</h4>
                  <p class="setting-desc">Choose your preferred application color theme</p>
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

              <!-- Interface Zoom -->
              <div class="setting-item">
                <div class="flex-between" style="margin-bottom: var(--space-3);">
                  <div>
                    <h4 class="setting-label">Editor & Interface Zoom</h4>
                    <p class="setting-desc">Scale the workspace text and visual elements</p>
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
                  <h4 class="setting-label">Editor Spellcheck</h4>
                  <p class="setting-desc">Enable standard browser-based spellchecking highlights in the editor</p>
                </div>
                <label class="switch-container">
                  <input type="checkbox" v-model="appStore.spellcheckEnabled" class="switch-input" />
                  <span class="switch-slider"></span>
                </label>
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

              <div class="form-grid">
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
                  <label class="label">Model</label>
                  <select class="input select" v-model="currentConfig.selectedModel">
                    <option v-for="m in currentProviderDetails.models" :key="m" :value="m">{{ m }}</option>
                  </select>
                </div>

                <div class="form-field">
                  <label class="label">Endpoint</label>
                  <input class="input" v-model="currentConfig.endpoint" :placeholder="currentProviderDetails.defaultEndpoint" />
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
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.65); display: flex; align-items: center; justify-content: center; z-index: 1000; backdrop-filter: blur(4px); }
.modal-content { background: var(--bg-primary); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); width: 95vw; max-width: 720px; height: 80vh; max-height: 580px; display: flex; flex-direction: column; overflow: hidden; box-shadow: var(--shadow-xl); }
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

/* AI Providers Split Pane Styles */
.providers-split-layout {
  display: flex;
  height: 100%;
  min-height: 480px;
}
.providers-sidebar {
  width: 180px;
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
</style>

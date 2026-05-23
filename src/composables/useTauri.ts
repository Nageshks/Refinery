import { invoke } from '@tauri-apps/api/core';
import type {
  Page, ReviewResult, SuggestionGroupWithItems, PreviewResult,
  ApplyResult, ProviderConfig, VersionSnapshot, CompareResult, ModelConfig
} from '../types';

// ─── Pages ──────────────────────────────────────────────────────────────
export const listPages = () => invoke<Page[]>('list_pages');
export const getPage = (pageId: string) => invoke<Page>('get_page', { pageId });
export const createPage = (title: string) => invoke<Page>('create_page', { title });
export const updatePage = (pageId: string, title: string, content: string) =>
  invoke<Page>('update_page', { pageId, title, content });
export const renamePage = (pageId: string, title: string) =>
  invoke<Page>('rename_page', { pageId, title });
export const deletePage = (pageId: string) => invoke<void>('delete_page', { pageId });

// ─── Review ─────────────────────────────────────────────────────────────
export const startReview = (pageId: string, apiKey: string, model: string, endpoint?: string) =>
  invoke<ReviewResult>('start_review', { pageId, apiKey, model, endpoint });
export const getReviewSuggestions = (sessionId: string) =>
  invoke<SuggestionGroupWithItems[]>('get_review_suggestions', { sessionId });
export const updateSuggestionApproval = (opts: { itemId?: string; groupId?: string; approvalState: string }) =>
  invoke<void>('update_suggestion_approval', opts);
export const computePreview = (pageId: string, sessionId: string) =>
  invoke<PreviewResult>('compute_preview', { pageId, sessionId });
export const applyApprovedSuggestions = (pageId: string, sessionId: string) =>
  invoke<ApplyResult>('apply_approved_suggestions', { pageId, sessionId });
export const rewriteSelection = (opts: { apiKey: string; model: string; endpoint?: string; selectedText: string }) =>
  invoke<string[]>('rewrite_selection', opts);
export const clearReviewSession = (pageId: string) => invoke<void>('clear_review_session', { pageId });

// ─── Providers ──────────────────────────────────────────────────────────
export const listProviders = () => invoke<ProviderConfig[]>('list_providers');
export const getActiveProvider = () => invoke<ProviderConfig | null>('get_active_provider');
export const setActiveProvider = (providerId: string) =>
  invoke<void>('set_active_provider', { providerId });
export const saveProviderConfig = (config: {
  id?: string; name: string; providerType: string;
  endpoint?: string; selectedModel: string; enabled: boolean;
}) => invoke<ProviderConfig>('save_provider_config', config);
export const deleteProviderConfig = (providerId: string) =>
  invoke<void>('delete_provider_config', { providerId });
export const testProvider = (apiKey: string, model: string, endpoint?: string, timeoutSecs?: number) =>
  invoke<string>('test_provider', { apiKey, model, endpoint, timeoutSecs });

// ─── Compare ────────────────────────────────────────────────────────────
export const compareTexts = (textA: string, textB: string, apiKey?: string, model?: string, endpoint?: string) =>
  invoke<CompareResult>('compare_texts', { textA, textB, apiKey, model, endpoint });

// ─── Versions ───────────────────────────────────────────────────────────
export const listVersions = (pageId: string) => invoke<VersionSnapshot[]>('list_versions', { pageId });
export const getVersion = (versionId: string) => invoke<VersionSnapshot>('get_version', { versionId });
export const restoreVersion = (pageId: string, versionId: string) =>
  invoke<string>('restore_version', { pageId, versionId });
export const renameVersion = (versionId: string, name: string) =>
  invoke<void>('rename_version', { versionId, name });
export const createManualVersion = (pageId: string, name: string) =>
  invoke<VersionSnapshot>('create_manual_version', { pageId, name });
export const deleteVersion = (versionId: string) =>
  invoke<void>('delete_version', { versionId });

// ─── Models ─────────────────────────────────────────────────────────────
export const listModels = () => invoke<ModelConfig[]>('list_models');
export const saveModelConfig = (model: {
  id: string;
  providerType: string;
  name: string;
  useCase: string;
  icon: string;
  isCustom: boolean;
  enabled?: boolean;
}) => invoke<ModelConfig>('save_model_config', model);
export const deleteModelConfig = (modelId: string) =>
  invoke<void>('delete_model_config', { modelId });
export const resetDefaultModels = () => invoke<ModelConfig[]>('reset_default_models');

// ─── Window / GPU Options ───────────────────────────────────────────────
export const setGpuAcceleration = (enabled: boolean) => invoke<void>('set_gpu_acceleration', { enabled });
export const getGpuAcceleration = () => invoke<boolean>('get_gpu_acceleration');


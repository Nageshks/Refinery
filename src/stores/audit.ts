import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { Store } from '@tauri-apps/plugin-store';

// ─── Types ──────────────────────────────────────────────────────────────

export type AuditWidgetType = 'binary' | 'scale_1_5' | 'text_match';

export interface AuditCheckDefinition {
  id: string;
  name: string;
  widgetType: AuditWidgetType;
  prompt: string;
}

export interface PageTypeDefinition {
  id: string;
  name: string;
  icon: string;
  inheritDefault: boolean;
  checks: AuditCheckDefinition[];
}

// ─── AI Response Types ──────────────────────────────────────────────────

export interface AuditCheckResult {
  checkId: string;
  name: string;
  widgetType: AuditWidgetType;
  value: string | number; // 'pass'/'fail' for binary, 1-5 for scale, extracted text for text_match
  feedback: string;
  suggestion?: string;
  snippet?: string; // text snippet from the draft for click-to-highlight
  evaluation?: string; // for text_match: the AI's comparison evaluation
}

export interface AuditResult {
  pageId: string;
  pageTypeId: string;
  contentHash: string;
  checks: AuditCheckResult[];
  rawFallback?: string; // if AI returned malformed JSON, store raw text here
  timestamp: number;
}

// ─── Store ──────────────────────────────────────────────────────────────

export const useAuditStore = defineStore('audit', () => {
  const pageTypes = ref<PageTypeDefinition[]>([]);
  const auditCache = ref<Map<string, AuditResult>>(new Map());
  const auditing = ref(false);
  const auditError = ref<string | null>(null);
  const storeReady = ref(false);

  // ─── Persistence ────────────────────────────────────────────────────

  const loadPageTypes = async () => {
    try {
      const store = await Store.load('page_types.json');
      const saved = await store.get<PageTypeDefinition[]>('page_types');
      if (saved && Array.isArray(saved)) {
        pageTypes.value = saved;
      }
      
      // Seed a default page type if it doesn't exist
      if (!pageTypes.value.some(pt => pt.id === 'default')) {
        const defaultSeed: PageTypeDefinition = {
          id: 'default',
          name: 'General Auditor',
          icon: '📄',
          inheritDefault: false,
          checks: [
            {
              id: 'tone',
              name: 'Tone Consistency',
              widgetType: 'binary',
              prompt: 'Verify if the tone of the draft is highly professional, consistent, and matches its intended audience.'
            },
            {
              id: 'clarity',
              name: 'Clarity & Concision',
              widgetType: 'scale_1_5',
              prompt: 'Rate the writing clarity and conciseness from 1 (very convoluted/wordy) to 5 (perfectly crisp, clear, and punchy).'
            },
            {
              id: 'cta',
              name: 'Core Takeaway',
              widgetType: 'text_match',
              prompt: 'Extract the primary call to action, main thesis, or key next step in the draft.'
            }
          ]
        };
        pageTypes.value.unshift(defaultSeed);
        await savePageTypes();
      }
      storeReady.value = true;
    } catch (err) {
      console.error('Failed to load page types:', err);
      // Fallback seed if loading fails entirely
      if (!pageTypes.value.some(pt => pt.id === 'default')) {
        pageTypes.value.unshift({
          id: 'default',
          name: 'General Auditor',
          icon: '📄',
          inheritDefault: false,
          checks: [
            {
              id: 'tone',
              name: 'Tone Consistency',
              widgetType: 'binary',
              prompt: 'Verify if the tone of the draft is highly professional, consistent, and matches its intended audience.'
            },
            {
              id: 'clarity',
              name: 'Clarity & Concision',
              widgetType: 'scale_1_5',
              prompt: 'Rate the writing clarity and conciseness from 1 (very convoluted/wordy) to 5 (perfectly crisp, clear, and punchy).'
            },
            {
              id: 'cta',
              name: 'Core Takeaway',
              widgetType: 'text_match',
              prompt: 'Extract the primary call to action, main thesis, or key next step in the draft.'
            }
          ]
        });
      }
      storeReady.value = true;
    }
  };

  const savePageTypes = async () => {
    try {
      const store = await Store.load('page_types.json');
      await store.set('page_types', JSON.parse(JSON.stringify(pageTypes.value)));
      await store.save();
    } catch (err) {
      console.error('Failed to save page types:', err);
    }
  };

  // ─── Page Type CRUD ─────────────────────────────────────────────────

  const defaultPageType = computed(() =>
    pageTypes.value.find(pt => pt.id === 'default') || null
  );

  const customPageTypes = computed(() =>
    pageTypes.value.filter(pt => pt.id !== 'default')
  );

  const getPageType = (id: string): PageTypeDefinition | null => {
    return pageTypes.value.find(pt => pt.id === id) || null;
  };

  const addPageType = async (pageType: PageTypeDefinition) => {
    pageTypes.value.push(pageType);
    await savePageTypes();
  };

  const updatePageType = async (updated: PageTypeDefinition) => {
    const idx = pageTypes.value.findIndex(pt => pt.id === updated.id);
    if (idx >= 0) {
      pageTypes.value[idx] = updated;
    }
    await savePageTypes();
  };

  const deletePageType = async (id: string) => {
    pageTypes.value = pageTypes.value.filter(pt => pt.id !== id);
    await savePageTypes();
  };

  // ─── Resolved Checks (Inheritance) ──────────────────────────────────

  const getResolvedChecks = (pageTypeId: string): AuditCheckDefinition[] => {
    const pt = getPageType(pageTypeId);
    if (!pt) return [];

    const baseChecks: AuditCheckDefinition[] = [];
    if (pt.inheritDefault && pt.id !== 'default') {
      const def = defaultPageType.value;
      if (def) {
        baseChecks.push(...def.checks);
      }
    }

    // Extend with the page type's own checks (no deduplication needed — user controls both lists)
    return [...baseChecks, ...pt.checks];
  };

  // ─── Content Hashing ────────────────────────────────────────────────

  const computeContentHash = (content: string): string => {
    // Simple hash for stale detection — length + char sampling
    let hash = content.length;
    for (let i = 0; i < content.length; i += Math.max(1, Math.floor(content.length / 100))) {
      hash = ((hash << 5) - hash + content.charCodeAt(i)) | 0;
    }
    return hash.toString(36);
  };

  // ─── Audit Cache ────────────────────────────────────────────────────

  const getCachedAudit = (pageId: string): AuditResult | null => {
    return auditCache.value.get(pageId) || null;
  };

  const isAuditStale = (pageId: string, currentContent: string): boolean => {
    const cached = getCachedAudit(pageId);
    if (!cached) return false; // no audit to be stale
    return cached.contentHash !== computeContentHash(currentContent);
  };

  const setCachedAudit = (result: AuditResult) => {
    auditCache.value.set(result.pageId, result);
  };

  const clearCachedAudit = (pageId: string) => {
    auditCache.value.delete(pageId);
  };

  // ─── Prompt Synthesis ───────────────────────────────────────────────

  const buildAuditPrompt = (checks: AuditCheckDefinition[], content: string): string => {
    const checkInstructions = checks.map((check, idx) => {
      let typeInstruction = '';
      if (check.widgetType === 'binary') {
        typeInstruction = 'Respond with value "pass" or "fail".';
      } else if (check.widgetType === 'scale_1_5') {
        typeInstruction = 'Respond with a numeric value from 1 to 5 (integer).';
      } else if (check.widgetType === 'text_match') {
        typeInstruction = 'Respond with the extracted text as the value. Include an "evaluation" field assessing the extraction.';
      }

      return `
Parameter ${idx + 1}: "${check.name}"
ID: "${check.id}"
Widget Type: "${check.widgetType}"
${typeInstruction}
Audit instruction: ${check.prompt}`;
    }).join('\n---\n');

    return `You are a professional writing auditor. Analyze the following draft according to the parameters below. For EACH parameter, provide your assessment.

CRITICAL: You MUST respond with ONLY valid JSON matching this exact schema — no markdown fencing, no commentary before or after:
{
  "checks": [
    {
      "checkId": "<the parameter ID>",
      "name": "<the parameter name>",
      "widgetType": "<binary|scale_1_5|text_match>",
      "value": "<pass/fail for binary, 1-5 integer for scale, extracted text string for text_match>",
      "feedback": "<1-3 sentence explanation of your assessment>",
      "suggestion": "<optional: actionable improvement advice>",
      "snippet": "<optional: exact quote from the draft this check relates to, for highlighting>",
      "evaluation": "<optional, only for text_match: evaluation of the extracted content>"
    }
  ]
}

─── PARAMETERS TO CHECK ───
${checkInstructions}

─── DRAFT TO ANALYZE ───
${content}`;
  };

  // ─── AI Execution ───────────────────────────────────────────────────

  const runAudit = async (
    pageId: string,
    pageTypeId: string,
    content: string,
    apiKey: string,
    model: string,
    endpoint?: string
  ): Promise<AuditResult> => {
    const checks = getResolvedChecks(pageTypeId);
    if (checks.length === 0) {
      throw new Error('No checks configured for this page type.');
    }

    auditing.value = true;
    auditError.value = null;

    try {
      const prompt = buildAuditPrompt(checks, content);
      const contentHash = computeContentHash(content);

      // Call the AI via the existing provider endpoint pattern
      const response = await fetch(endpoint || 'https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          model,
          messages: [
            { role: 'system', content: 'You are a professional writing auditor. Always respond with valid JSON only.' },
            { role: 'user', content: prompt }
          ],
          temperature: 0.3,
          max_tokens: 4096,
        }),
      });

      if (!response.ok) {
        throw new Error(`API request failed: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      const rawContent = data.choices?.[0]?.message?.content || '';

      // Attempt JSON parse with fallback
      let parsedChecks: AuditCheckResult[] = [];
      let rawFallback: string | undefined;

      try {
        // Strip potential markdown fencing
        let jsonStr = rawContent.trim();
        if (jsonStr.startsWith('```')) {
          jsonStr = jsonStr.replace(/^```(?:json)?\s*/i, '').replace(/```\s*$/, '').trim();
        }
        const parsed = JSON.parse(jsonStr);
        parsedChecks = (parsed.checks || []).map((c: any) => ({
          checkId: c.checkId || '',
          name: c.name || '',
          widgetType: c.widgetType || 'binary',
          value: c.value,
          feedback: c.feedback || '',
          suggestion: c.suggestion || undefined,
          snippet: c.snippet || undefined,
          evaluation: c.evaluation || undefined,
        }));
      } catch {
        // Fallback: store raw response as text
        rawFallback = rawContent;
        console.warn('Audit: Failed to parse AI JSON response, using fallback.');
      }

      const result: AuditResult = {
        pageId,
        pageTypeId,
        contentHash,
        checks: parsedChecks,
        rawFallback,
        timestamp: Date.now(),
      };

      setCachedAudit(result);
      return result;
    } catch (err: any) {
      auditError.value = err.message || 'Audit failed';
      throw err;
    } finally {
      auditing.value = false;
    }
  };

  // Initialize on load
  loadPageTypes();

  return {
    pageTypes,
    auditCache,
    auditing,
    auditError,
    storeReady,
    defaultPageType,
    customPageTypes,
    getPageType,
    addPageType,
    updatePageType,
    deletePageType,
    getResolvedChecks,
    computeContentHash,
    getCachedAudit,
    isAuditStale,
    setCachedAudit,
    clearCachedAudit,
    buildAuditPrompt,
    runAudit,
    loadPageTypes,
    savePageTypes,
  };
});

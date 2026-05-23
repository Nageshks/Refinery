import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { SuggestionGroupWithItems, ReviewResult, PreviewResult } from '../types';
import { clearReviewSession } from '../composables/useTauri';

export const useReviewStore = defineStore('review', () => {
  const loading = ref(false);
  const cancelled = ref(false);
  const error = ref<string | null>(null);
  const reviewResult = ref<ReviewResult | null>(null);
  const groups = ref<SuggestionGroupWithItems[]>([]);
  const preview = ref<PreviewResult | null>(null);
  const sessionId = ref<string | null>(null);

  const setReviewResult = (result: ReviewResult) => {
    reviewResult.value = result;
    groups.value = result.groups;
    sessionId.value = result.session.id;
    error.value = null;
  };

  const clearReview = async (pageId?: string) => {
    reviewResult.value = null;
    groups.value = [];
    preview.value = null;
    sessionId.value = null;
    error.value = null;
    if (pageId) {
      try {
        await clearReviewSession(pageId);
      } catch (err) {
        console.error('Failed to clear review session in SQLite:', err);
      }
    }
  };

  return { loading, cancelled, error, reviewResult, groups, preview, sessionId, setReviewResult, clearReview };
});

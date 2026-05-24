import { storeToRefs } from 'pinia';
import { useReviewStore } from '../stores/review';
import { useAppStore } from '../stores/app';
import { startReview, updateSuggestionApproval, computePreview, applyApprovedSuggestions, getReviewSuggestions } from './useTauri';

export function useReview() {
  const store = useReviewStore();
  const { loading, error, reviewResult, groups, preview, editorialContext } = storeToRefs(store);

  const doReview = async (pageId: string, apiKey: string, model: string, endpoint?: string, context?: string) => {
    loading.value = true;
    error.value = null;
    store.cancelled = false;
    try {
      const result = await startReview(pageId, apiKey, model, endpoint, context);
      if (store.cancelled) {
        return; // Discard result since it was cancelled
      }
      store.setReviewResult(result);
      await refreshPreview(pageId);
      return result;
    } catch (e: any) {
      if (store.cancelled) {
        return; // Ignore error since it was cancelled
      }
      error.value = typeof e === 'string' ? e : e.message || 'Review failed';
      throw e;
    } finally {
      if (!store.cancelled) {
        loading.value = false;
      }
    }
  };

  const cancelReview = () => {
    store.cancelled = true;
    loading.value = false;
  };

  const toggleItemApproval = async (itemId: string, state: string) => {
    await updateSuggestionApproval({ itemId, approvalState: state });
    
    // Find our target item first
    let targetItem: any = null;
    for (const g of groups.value) {
      const found = g.items.find(i => i.id === itemId);
      if (found) {
        targetItem = found;
        break;
      }
    }

    // Update local state and check conflicts
    if (targetItem) {
      targetItem.approval_state = state as any;

      if (state === 'approved') {
        const appStore = useAppStore();
        for (const otherG of groups.value) {
          for (const otherItem of otherG.items) {
            if (otherItem.id !== itemId && otherItem.approval_state === 'approved') {
              const overlaps = !(targetItem.span_end <= otherItem.span_start || otherItem.span_end <= targetItem.span_start);
              if (overlaps) {
                // Auto-deselect!
                otherItem.approval_state = 'pending';
                await updateSuggestionApproval({ itemId: otherItem.id, approvalState: 'pending' });
                
                // Notify the user!
                const origText = otherItem.original_text.length > 25 ? otherItem.original_text.slice(0, 25) + '...' : otherItem.original_text;
                appStore.notify(`Deselected conflicting approved suggestion: "${origText}"`, 'info');
                
                // Recompute the other item's group state!
                const allApproved = otherG.items.every(i => i.approval_state === 'approved');
                const allRejected = otherG.items.every(i => i.approval_state === 'rejected');
                const someApproved = otherG.items.some(i => i.approval_state === 'approved');
                otherG.group.approval_state = allApproved ? 'approved' : allRejected ? 'rejected' : someApproved ? 'partial' : 'pending';
              }
            }
          }
        }
      }

      // Recompute target item's group state
      for (const g of groups.value) {
        if (g.items.some(i => i.id === itemId)) {
          const allApproved = g.items.every(i => i.approval_state === 'approved');
          const allRejected = g.items.every(i => i.approval_state === 'rejected');
          const someApproved = g.items.some(i => i.approval_state === 'approved');
          g.group.approval_state = allApproved ? 'approved' : allRejected ? 'rejected' : someApproved ? 'partial' : 'pending';
          break;
        }
      }
    }
  };

  const toggleGroupApproval = async (groupId: string, state: string) => {
    await updateSuggestionApproval({ groupId, approvalState: state });
    const g = groups.value.find(g => g.group.id === groupId);
    if (g) {
      g.group.approval_state = state as any;
      
      // Update each item in the group
      for (const item of g.items) {
        item.approval_state = state as any;
        
        if (state === 'approved') {
          const appStore = useAppStore();
          // Check conflict with other approved items in OTHER groups
          for (const otherG of groups.value) {
            if (otherG.group.id !== groupId) {
              for (const otherItem of otherG.items) {
                if (otherItem.approval_state === 'approved') {
                  const overlaps = !(item.span_end <= otherItem.span_start || otherItem.span_end <= item.span_start);
                  if (overlaps) {
                    otherItem.approval_state = 'pending';
                    await updateSuggestionApproval({ itemId: otherItem.id, approvalState: 'pending' });
                    
                    const origText = otherItem.original_text.length > 25 ? otherItem.original_text.slice(0, 25) + '...' : otherItem.original_text;
                    appStore.notify(`Deselected conflicting approved suggestion: "${origText}"`, 'info');
                    
                    // Recompute the other item's group state!
                    const allApproved = otherG.items.every(i => i.approval_state === 'approved');
                    const allRejected = otherG.items.every(i => i.approval_state === 'rejected');
                    const someApproved = otherG.items.some(i => i.approval_state === 'approved');
                    otherG.group.approval_state = allApproved ? 'approved' : allRejected ? 'rejected' : someApproved ? 'partial' : 'pending';
                  }
                }
              }
            }
          }
        }
      }
    }
  };

  const refreshPreview = async (pageId: string) => {
    if (!reviewResult.value) return;
    try {
      preview.value = await computePreview(pageId, reviewResult.value.session.id);
    } catch (e: any) {
      console.error('Preview failed:', e);
    }
  };

  const applyChanges = async (pageId: string) => {
    if (!reviewResult.value) throw new Error('No active review session');
    loading.value = true;
    try {
      const result = await applyApprovedSuggestions(pageId, reviewResult.value.session.id);
      // Clear review state after applying
      await store.clearReview(pageId);
      return result;
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e.message || 'Apply failed';
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const loadReviewSession = async (pageId: string, sessionId: string) => {
    loading.value = true;
    error.value = null;
    try {
      const suggestions = await getReviewSuggestions(sessionId);
      const result = {
        session: {
          id: sessionId,
          page_id: pageId,
          provider_id: '',
          model_id: '',
          prompt_version: '',
          raw_response: '',
          status: 'completed',
          created_at: ''
        },
        groups: suggestions
      };
      store.setReviewResult(result);
      await refreshPreview(pageId);
    } catch (e: any) {
      console.error('Failed to load review session:', e);
      error.value = typeof e === 'string' ? e : e.message || 'Load session failed';
    } finally {
      loading.value = false;
    }
  };

  const clearReview = (pageId?: string) => {
    store.clearReview(pageId);
  };

  const approvedCount = () => {
    return groups.value.reduce((sum, g) =>
      sum + g.items.filter(i => i.approval_state === 'approved').length, 0
    );
  };

  const totalCount = () => {
    return groups.value.reduce((sum, g) => sum + g.items.length, 0);
  };

  return {
    loading, error, reviewResult, groups, preview, editorialContext,
    doReview, cancelReview, toggleItemApproval, toggleGroupApproval,
    refreshPreview, applyChanges, clearReview, loadReviewSession,
    approvedCount, totalCount,
  };
}

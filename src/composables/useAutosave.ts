import { ref, watch, type Ref } from 'vue';
import { updatePage } from './useTauri';

export function useAutosave(
  pageId: Ref<string>,
  title: Ref<string>,
  content: Ref<string>,
  delay: number = 500
) {
  const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let timeout: ReturnType<typeof setTimeout> | null = null;

  const save = async () => {
    if (!pageId.value) return;
    saveStatus.value = 'saving';
    try {
      await updatePage(pageId.value, title.value, content.value);
      saveStatus.value = 'saved';
      setTimeout(() => {
        if (saveStatus.value === 'saved') saveStatus.value = 'idle';
      }, 2000);
    } catch (e) {
      saveStatus.value = 'error';
      console.error('Autosave failed:', e);
    }
  };

  const debouncedSave = () => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(save, delay);
  };

  watch([content, title], () => {
    if (pageId.value) debouncedSave();
  }, { deep: true });

  return { saveStatus, save };
}

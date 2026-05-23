<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import { usePagesStore } from '../stores/pages';
import { useAutosave } from '../composables/useAutosave';
import { useAppStore } from '../stores/app';
import { EditorState } from '@tiptap/pm/state';

const pagesStore = usePagesStore();
const appStore = useAppStore();

const pageId = computed(() => pagesStore.activePageId || '');
const title = ref(pagesStore.activePage?.title || '');
const content = ref(pagesStore.activePage?.content || '');

// Start autosave process automatically on content/title changes
useAutosave(pageId, title, content);

const handleKeyDown = () => {
  if (appStore.zenFocusEnabled && !document.documentElement.classList.contains('zen-active')) {
    document.documentElement.classList.add('zen-active');
  }
};

const handleMouseMove = () => {
  if (document.documentElement.classList.contains('zen-active')) {
    document.documentElement.classList.remove('zen-active');
  }
};

onMounted(() => {
  window.addEventListener('mousemove', handleMouseMove);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  document.documentElement.classList.remove('zen-active');
});

watch(() => appStore.zenFocusEnabled, (val) => {
  if (!val) {
    document.documentElement.classList.remove('zen-active');
  }
});

const editor = useEditor({
  content: pagesStore.activePage?.content || '',
  extensions: [
    StarterKit,
    Placeholder.configure({ placeholder: 'Start writing...' }),
    Underline,
  ],
  editorProps: {
    attributes: {
      class: 'tiptap',
      spellcheck: appStore.spellcheckEnabled ? 'true' : 'false',
    },
    handleKeyDown() {
      handleKeyDown();
      return false; // let the editor handle the keystroke normally
    }
  },
  onUpdate({ editor: e }) {
    content.value = e.getHTML();
    pagesStore.updateActivePage(title.value, content.value);
  },
});

const isReadonly = computed(() => appStore.activeView === 'review');

watch(isReadonly, (val) => {
  editor.value?.setEditable(!val);
});

watch(() => appStore.spellcheckEnabled, (val) => {
  if (editor.value) {
    editor.value.setOptions({
      editorProps: {
        attributes: {
          spellcheck: val ? 'true' : 'false',
        },
      },
    });
  }
});

const lastPageId = ref(pagesStore.activePageId || '');

watch(() => pagesStore.activePage, (page) => {
  if (page) {
    title.value = page.title;
    const pageChanged = page.id !== lastPageId.value;
    if (editor.value && (page.content !== content.value || pageChanged)) {
      content.value = page.content;
      editor.value.commands.setContent(page.content || '', false);
    }
    if (editor.value && pageChanged) {
      const state = editor.value.state;
      const newState = EditorState.create({
        doc: state.doc,
        plugins: state.plugins,
        selection: state.selection,
      });
      editor.value.view.updateState(newState);
      lastPageId.value = page.id;
    }
  }
});
</script>

<template>
  <div class="editor-container">
    <div v-if="isReadonly" class="readonly-banner">
      <span>📖 Read-only during review</span>
    </div>
    <div class="editor-scroll">
      <EditorContent :editor="editor" />
    </div>
  </div>
</template>

<style scoped>
.editor-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.readonly-banner {
  padding: var(--space-2) var(--space-4);
  background: var(--color-pending-bg);
  border-bottom: 1px solid var(--color-pending-border);
  font-size: var(--font-size-xs);
  color: var(--accent-muted);
  text-align: center;
  flex-shrink: 0;
}

.editor-scroll {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4) 0;
}
</style>

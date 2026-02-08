<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import Editor from '@toast-ui/editor';
import '@toast-ui/editor/dist/toastui-editor.css';
import { 
  FileText, 
  Save, 
  Plus,
  RefreshCw,
  Download,
  Bot,
  Loader2,
  Sparkles,
  FolderOpen
} from 'lucide-vue-next';

// ==================== State ====================

const currentFile = ref('untitled');
const docContent = ref('# 新文档\n\n在这里开始编写...');
const savedFiles = ref<string[]>([]);
const isSaving = ref(false);
const isGenerating = ref(false);
const editorContainer = ref<HTMLElement | null>(null);
const editor = ref<Editor | null>(null);
const aiInput = ref('');
const showAiPanel = ref(false);

// ==================== Editor ====================

const initEditor = () => {
  if (!editorContainer.value) return;
  
  if (editor.value) {
    editor.value.destroy();
  }
  
  editor.value = new Editor({
    el: editorContainer.value,
    height: '100%',
    initialEditType: 'wysiwyg', // 纯所见即所得模式
    previewStyle: 'tab',
    initialValue: docContent.value,
    theme: 'light',
    toolbarItems: [
      ['heading', 'bold', 'italic'],
      ['ul', 'ol', 'task'],
      ['image', 'link'],
    ],
    autofocus: false,
    useCommandShortcut: true,
    events: {
      change: () => {
        if (editor.value) {
          docContent.value = editor.value.getMarkdown();
        }
      }
    }
  });
};

// ==================== Actions ====================

const createNewDocument = () => {
  if (confirm('创建新文档？当前内容将丢失')) {
    currentFile.value = 'untitled_' + Date.now().toString().slice(-4);
    const newContent = '# 新文档\n\n在这里开始编写...';
    docContent.value = newContent;
    nextTick(() => {
      if (editor.value) {
        editor.value.setMarkdown(newContent);
      }
    });
  }
};

const handleSave = async () => {
  if (!currentFile.value.trim()) {
    const name = prompt('请输入文件名:', 'document');
    if (!name) return;
    currentFile.value = name;
  }
  
  isSaving.value = true;
  try {
    await invoke('save_document', { 
      filename: currentFile.value,
      content: docContent.value 
    });
    await loadSavedFiles();
    alert('保存成功');
  } catch (err) {
    console.error(err);
    alert('保存失败: ' + err);
  } finally {
    isSaving.value = false;
  }
};

const exportDoc = () => {
  const blob = new Blob([docContent.value], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${currentFile.value || 'document'}.md`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

const loadSavedFiles = async () => {
  try {
    const files = await invoke('list_documents') as string[];
    savedFiles.value = files;
  } catch (err) {
    console.error('加载失败:', err);
  }
};

const loadDocument = async (filename: string) => {
  try {
    const content = await invoke('load_document', { filename }) as string;
    currentFile.value = filename;
    docContent.value = content;
    nextTick(() => {
      if (editor.value) {
        editor.value.setMarkdown(content);
      }
    });
  } catch (err) {
    alert('加载失败: ' + err);
  }
};

// ==================== AI PlantUML ====================

const generatePlantUML = async () => {
  if (!aiInput.value.trim()) {
    alert('请输入需求描述');
    return;
  }
  
  isGenerating.value = true;
  try {
    // 调用后端 AI 接口生成 PlantUML
    const result = await invoke('generate_plantuml', { 
      description: aiInput.value 
    }) as string;
    
    // 将生成的 PlantUML 插入到文档中
    const plantumlBlock = `\n\n## 自动生成图表\n\n\`\`\`plantuml\n${result}\n\`\`\`\n`;
    
    const currentContent = editor.value?.getMarkdown() || '';
    const newContent = currentContent + plantumlBlock;
    
    docContent.value = newContent;
    if (editor.value) {
      editor.value.setMarkdown(newContent);
    }
    
    aiInput.value = '';
    showAiPanel.value = false;
  } catch (err) {
    console.error(err);
    // 演示模式：生成示例 PlantUML
    const demoPlantUML = `@startuml
!theme plain
skinparam backgroundColor #FEFEFE

actor 用户
participant "前端界面" as UI
participant "AI服务" as AI
database "文档存储" as DB

用户 -> UI: 输入需求描述
UI -> AI: 调用生成接口
AI -> AI: 分析需求
AI -> UI: 返回PlantUML代码
UI -> DB: 保存文档
UI -> 用户: 显示生成结果

@enduml`;
    
    const plantumlBlock = `\n\n## 自动生成图表\n\n\`\`\`plantuml\n${demoPlantUML}\n\`\`\`\n`;
    
    const currentContent = editor.value?.getMarkdown() || '';
    const newContent = currentContent + plantumlBlock;
    
    docContent.value = newContent;
    if (editor.value) {
      editor.value.setMarkdown(newContent);
    }
    
    aiInput.value = '';
    showAiPanel.value = false;
  } finally {
    isGenerating.value = false;
  }
};

// ==================== Lifecycle ====================

onMounted(() => {
  loadSavedFiles();
  nextTick(() => {
    initEditor();
  });
});

onBeforeUnmount(() => {
  if (editor.value) {
    editor.value.destroy();
    editor.value = null;
  }
});
</script>

<template>
  <div class="flex h-full bg-gray-50 text-gray-800 font-sans overflow-hidden">
    <!-- 左侧 Sidebar -->
    <aside class="w-64 bg-white border-r border-gray-200 flex flex-col flex-shrink-0 shadow-sm">
      <!-- Logo -->
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center gap-2">
          <div class="bg-gradient-to-br from-blue-500 to-purple-600 p-2 rounded-lg">
            <Sparkles :size="18" class="text-white" />
          </div>
          <div>
            <h1 class="font-bold text-gray-800">设计工作室</h1>
            <p class="text-xs text-gray-500">AI 辅助设计</p>
          </div>
        </div>
      </div>

      <!-- 三个主要功能按钮 -->
      <div class="p-3 grid grid-cols-3 gap-2 border-b border-gray-200">
        <button 
          @click="createNewDocument"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors"
          title="新建文档"
        >
          <Plus :size="18" class="text-gray-600" />
          <span class="text-[10px] text-gray-600">新建</span>
        </button>
        
        <button 
          @click="handleSave"
          :disabled="isSaving"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors disabled:opacity-50"
          title="保存文档"
        >
          <Save :size="18" class="text-blue-600" />
          <span class="text-[10px] text-gray-600">保存</span>
        </button>
        
        <button 
          @click="exportDoc"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors"
          title="导出 Markdown"
        >
          <Download :size="18" class="text-green-600" />
          <span class="text-[10px] text-gray-600">导出</span>
        </button>
      </div>

      <!-- AI PlantUML 生成 -->
      <div class="p-3 border-b border-gray-200">
        <button 
          @click="showAiPanel = !showAiPanel"
          class="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 text-white hover:opacity-90 transition-opacity"
        >
          <Bot :size="16" />
          <span class="text-sm font-medium">AI 生成图表</span>
        </button>
        
        <!-- AI 输入面板 -->
        <div v-if="showAiPanel" class="mt-2 space-y-2">
          <textarea 
            v-model="aiInput"
            placeholder="描述你想要的图表，例如：用户登录流程图..."
            class="w-full h-20 p-2 text-xs border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-purple-500"
          />
          <button 
            @click="generatePlantUML"
            :disabled="isGenerating"
            class="w-full py-1.5 px-3 rounded-lg bg-purple-600 text-white text-xs font-medium hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-1"
          >
            <Loader2 v-if="isGenerating" :size="12" class="animate-spin" />
            <span>{{ isGenerating ? '生成中...' : '生成 PlantUML' }}</span>
          </button>
        </div>
      </div>

      <!-- 文件列表 -->
      <div class="flex-1 overflow-hidden flex flex-col">
        <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200">
          <span class="text-xs font-medium text-gray-500">文档列表</span>
          <button @click="loadSavedFiles" class="p-1 hover:bg-gray-100 rounded">
            <RefreshCw :size="12" class="text-gray-400" />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <button 
            v-for="file in savedFiles" 
            :key="file"
            @click="loadDocument(file)"
            :class="[
              'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors',
              currentFile === file 
                ? 'bg-blue-50 text-blue-700 border border-blue-200' 
                : 'hover:bg-gray-100 text-gray-700'
            ]"
          >
            <FileText :size="14" :class="currentFile === file ? 'text-blue-500' : 'text-gray-400'" />
            <span class="truncate">{{ file }}</span>
          </button>
          <div v-if="savedFiles.length === 0" class="text-center py-4 text-xs text-gray-400">
            暂无文档
          </div>
        </div>
      </div>
    </aside>

    <!-- 主编辑区 -->
    <main class="flex-1 flex flex-col bg-white min-w-0">
      <!-- 顶部栏 -->
      <header class="h-12 border-b border-gray-200 flex items-center justify-between px-4 bg-white">
        <div class="flex items-center gap-2">
          <FolderOpen :size="16" class="text-gray-400" />
          <input 
            v-model="currentFile"
            type="text"
            placeholder="输入文件名"
            class="px-2 py-1 text-sm border border-gray-300 rounded hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 w-48"
          />
          <span class="text-xs text-gray-400">.md</span>
        </div>
        <div class="text-xs text-gray-400">
          {{ docContent.length }} 字符
        </div>
      </header>

      <!-- 编辑器 -->
      <div class="flex-1 overflow-hidden p-4">
        <div ref="editorContainer" class="h-full rounded-lg border border-gray-200 overflow-hidden" />
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Toast UI Editor Light Theme Customization */
:deep(.toastui-editor-defaultUI) {
  border: none;
}

:deep(.toastui-editor-toolbar) {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

:deep(.toastui-editor-toolbar button) {
  color: #4b5563;
}

:deep(.toastui-editor-toolbar button:hover) {
  background: #e5e7eb;
}

:deep(.toastui-editor-main) {
  background: #ffffff;
}

:deep(.ProseMirror) {
  padding: 1rem;
  color: #1f2937;
}

:deep(.toastui-editor-contents) {
  font-size: 14px;
  line-height: 1.6;
}

:deep(.toastui-editor-contents h1) {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 1rem 0 0.5rem;
  color: #111827;
}

:deep(.toastui-editor-contents h2) {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0.875rem 0 0.5rem;
  color: #1f2937;
}

:deep(.toastui-editor-contents h3) {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0.75rem 0 0.5rem;
  color: #1f2937;
}

:deep(.toastui-editor-contents p) {
  margin: 0.5rem 0;
  color: #374151;
}

:deep(.toastui-editor-contents ul),
:deep(.toastui-editor-contents ol) {
  margin: 0.5rem 0;
  padding-left: 1.5rem;
}

:deep(.toastui-editor-contents li) {
  margin: 0.25rem 0;
}

:deep(.toastui-editor-contents code) {
  background: #f3f4f6;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-family: monospace;
  font-size: 0.875em;
  color: #ef4444;
}

:deep(.toastui-editor-contents pre) {
  background: #1f2937;
  color: #f9fafb;
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
  margin: 0.75rem 0;
}

:deep(.toastui-editor-contents pre code) {
  background: transparent;
  color: inherit;
  padding: 0;
}

:deep(.toastui-editor-contents blockquote) {
  border-left: 4px solid #e5e7eb;
  padding-left: 1rem;
  margin: 0.75rem 0;
  color: #6b7280;
}

:deep(.toastui-editor-contents a) {
  color: #3b82f6;
  text-decoration: none;
}

:deep(.toastui-editor-contents a:hover) {
  text-decoration: underline;
}

:deep(.toastui-editor-contents table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0.75rem 0;
}

:deep(.toastui-editor-contents th),
:deep(.toastui-editor-contents td) {
  border: 1px solid #e5e7eb;
  padding: 0.5rem;
  text-align: left;
}

:deep(.toastui-editor-contents th) {
  background: #f9fafb;
  font-weight: 600;
}

:deep(.toastui-editor-contents img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}
</style>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Download,
  RefreshCw,
  Search,
  FileText,
  X,
  FolderOpen,
  Terminal,
  AlertCircle,
  CheckCircle
} from 'lucide-vue-next';

export interface RemoteTemplate {
  id: string;
  name: string;
  description: string;
  content: string;
}

const props = defineProps<{
  show: boolean;
  templateUrl: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select', template: RemoteTemplate): void;
}>();

const templates = ref<RemoteTemplate[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const searchQuery = ref('');
const selectedTemplate = ref<RemoteTemplate | null>(null);
const previewMode = ref(false);
const useCommandLine = ref(false);
const connectionStatus = ref<'idle' | 'testing' | 'success' | 'error'>('idle');

const loadTemplates = async () => {
  isLoading.value = true;
  error.value = null;
  
  try {
    if (!props.templateUrl || !props.templateUrl.trim()) {
      error.value = '未配置远程模板 URL，请在设置中配置';
      templates.value = [];
      return;
    }
    
    const result = await invoke('fetch_templates_via_command', { 
      url: props.templateUrl,
      useCommandLine: useCommandLine.value
    }) as string;
    
    try {
      const parsed = JSON.parse(result);
      if (Array.isArray(parsed)) {
        templates.value = parsed;
        error.value = null;
      } else {
        error.value = '远程模板格式不正确';
        templates.value = [];
      }
    } catch (parseErr) {
      error.value = '解析模板数据失败: ' + result;
      templates.value = [];
    }
  } catch (err: any) {
    console.error('加载模板失败:', err);
    error.value = err.message || '加载模板失败';
    templates.value = [];
  } finally {
    isLoading.value = false;
  }
};

const testConnection = async () => {
  if (!props.templateUrl) {
    alert('请先配置远程模板 URL');
    return;
  }
  
  connectionStatus.value = 'testing';
  try {
    const result = await invoke('test_api_connection', { url: props.templateUrl }) as string;
    if (result === 'success') {
      connectionStatus.value = 'success';
      alert('连接测试成功！');
    } else {
      connectionStatus.value = 'error';
      alert('连接测试失败: ' + result);
    }
  } catch (err: any) {
    connectionStatus.value = 'error';
    alert('连接测试失败: ' + err.message);
  }
};

const filteredTemplates = computed(() => {
  if (!searchQuery.value) {
    return templates.value;
  }
  
  const query = searchQuery.value.toLowerCase();
  return templates.value.filter(t => 
    t.name.toLowerCase().includes(query) ||
    t.description.toLowerCase().includes(query) ||
    t.id.toLowerCase().includes(query)
  );
});

const selectTemplate = (template: RemoteTemplate) => {
  selectedTemplate.value = template;
  previewMode.value = true;
};

const confirmUseTemplate = () => {
  if (selectedTemplate.value) {
    emit('select', selectedTemplate.value);
    closeModal();
  }
};

const closeModal = () => {
  previewMode.value = false;
  selectedTemplate.value = null;
  searchQuery.value = '';
  emit('close');
};

onMounted(() => {
  const saved = localStorage.getItem('app-config');
  if (saved) {
    try {
      const config = JSON.parse(saved);
      useCommandLine.value = config.useCommandLine || false;
    } catch (e) {
      console.error('加载配置失败:', e);
    }
  }
});

watch(() => props.show, (newVal) => {
  if (newVal) {
    loadTemplates();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="closeModal" />
        
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-blue-50 to-indigo-50">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl">
                <FolderOpen :size="20" class="text-white" />
              </div>
              <div>
                <h2 class="text-lg font-bold text-gray-800">模板库</h2>
                <p class="text-xs text-gray-500">选择模板快速开始</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="testConnection"
                :disabled="!templateUrl || isLoading"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
                title="测试连接"
              >
                <Terminal :size="18" :class="{
                  'text-green-500': connectionStatus === 'success',
                  'text-red-500': connectionStatus === 'error',
                  'text-gray-400': connectionStatus === 'idle' || connectionStatus === 'testing'
                }" />
              </button>
              <button
                @click="loadTemplates"
                :disabled="isLoading"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
                title="刷新"
              >
                <RefreshCw :size="18" :class="isLoading ? 'animate-spin' : ''" class="text-gray-500" />
              </button>
              <button @click="closeModal" class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors">
                <X :size="20" class="text-gray-500" />
              </button>
            </div>
          </div>
          
          <div class="p-4 border-b border-gray-200 bg-gray-50/50">
            <div class="relative">
              <Search :size="18" class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
              <input
                v-model="searchQuery"
                type="text"
                placeholder="搜索模板..."
                class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              />
            </div>
          </div>
          
          <div class="flex-1 overflow-hidden flex">
            <div :class="['flex-1 overflow-y-auto p-4', previewMode ? '' : 'w-full']">
              <div v-if="isLoading" class="flex flex-col items-center justify-center h-64">
                <RefreshCw :size="32" class="animate-spin text-blue-500 mb-4" />
                <p class="text-gray-500">加载模板中...</p>
                <p class="text-xs text-gray-400 mt-2" v-if="useCommandLine">正在通过命令行调用 API...</p>
              </div>
              
              <div v-else-if="error" class="flex flex-col items-center justify-center h-64 text-center">
                <AlertCircle :size="32" class="text-amber-500 mb-4" />
                <p class="text-gray-800 mb-2">{{ error }}</p>
                <p class="text-xs text-gray-400 mb-4" v-if="useCommandLine">请检查命令行调用配置</p>
                <button
                  @click="loadTemplates"
                  class="text-blue-600 hover:underline"
                >
                  重试
                </button>
              </div>
              
              <div v-else-if="filteredTemplates.length === 0" class="flex flex-col items-center justify-center h-64 text-center">
                <FileText :size="32" class="text-gray-300 mb-4" />
                <p class="text-gray-500">没有找到模板</p>
                <p class="text-xs text-gray-400 mt-2">请检查远程模板 URL 配置</p>
              </div>
              
              <div v-else class="grid grid-cols-2 gap-3">
                <div
                  v-for="template in filteredTemplates"
                  :key="template.id"
                  @click="selectTemplate(template)"
                  :class="[
                    'p-4 rounded-xl border-2 cursor-pointer transition-all hover:shadow-md',
                    selectedTemplate?.id === template.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-blue-300'
                  ]"
                >
                  <h3 class="font-semibold text-gray-800 mb-1">{{ template.name }}</h3>
                  <p class="text-xs text-gray-400 mb-2">ID: {{ template.id }}</p>
                  <p class="text-sm text-gray-500 line-clamp-2">{{ template.description }}</p>
                </div>
              </div>
            </div>
            
            <div 
              v-if="previewMode && selectedTemplate"
              class="w-96 border-l border-gray-200 bg-gray-50 flex flex-col animate-slide-in"
            >
              <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white">
                <h3 class="font-semibold text-gray-800">预览</h3>
                <button @click="previewMode = false" class="text-gray-400 hover:text-gray-600">
                  <X :size="18" />
                </button>
              </div>
              <div class="flex-1 overflow-y-auto p-4">
                <h2 class="text-lg font-bold text-gray-800 mb-1">{{ selectedTemplate.name }}</h2>
                <p class="text-xs text-gray-400 mb-3">ID: {{ selectedTemplate.id }}</p>
                <p class="text-sm text-gray-500 mb-4">{{ selectedTemplate.description }}</p>
                
                <div class="bg-white rounded-xl border border-gray-200 p-4 mb-4">
                  <pre class="text-xs text-gray-600 overflow-auto max-h-64 whitespace-pre-wrap">{{ selectedTemplate.content.substring(0, 1000) }}{{ selectedTemplate.content.length > 1000 ? '\n...(内容截断)' : '' }}</pre>
                </div>
                
                <button
                  @click="confirmUseTemplate"
                  class="w-full py-2.5 bg-gradient-to-r from-blue-500 to-indigo-600 text-white rounded-xl font-medium hover:opacity-90 transition-all flex items-center justify-center gap-2"
                >
                  <CheckCircle :size="18" />
                  使用此模板
                </button>
              </div>
            </div>
          </div>
          
          <div v-if="!templateUrl" class="px-4 py-3 border-t border-gray-200 bg-amber-50">
            <p class="text-xs text-amber-700 flex items-center gap-2">
              <AlertCircle :size="14" />
              未配置远程模板 URL，请先在设置中配置
            </p>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .relative,
.modal-leave-to .relative {
  transform: scale(0.95);
  opacity: 0;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.animate-slide-in {
  animation: slideIn 0.3s ease-out;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>

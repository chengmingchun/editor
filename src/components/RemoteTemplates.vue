<script setup lang="ts">
/**
 * RemoteTemplates.vue - 远程模板库组件
 * 
 * 接口格式：
 * {
 *   id: string,          // 模板ID
 *   name: string,        // 模板名称
 *   description: string, // 模板简述
 *   content: string      // 模板内容
 * }
 */

import { ref, onMounted, watch, computed } from 'vue';
import { 
  Download,
  RefreshCw,
  Search,
  FileText,
  X,
  FolderOpen,
} from 'lucide-vue-next';

// ==================== 类型定义 ====================

export interface RemoteTemplate {
  id: string;
  name: string;
  description: string;
  content: string;
}

// ==================== Props & Emits ====================

const props = defineProps<{
  show: boolean;
  templateUrl: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select', template: RemoteTemplate): void;
}>();

// ==================== 响应式状态 ====================

const templates = ref<RemoteTemplate[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const searchQuery = ref('');
const selectedTemplate = ref<RemoteTemplate | null>(null);
const previewMode = ref(false);

// ==================== 方法定义 ====================

/**
 * 从远程加载模板列表
 */
const loadTemplates = async () => {
  isLoading.value = true;
  error.value = null;
  
  try {
    let allTemplates: RemoteTemplate[] = [];
    
    // 如果配置了远程模板源，尝试加载
    if (props.templateUrl && props.templateUrl.trim()) {
      try {
        const response = await fetch(props.templateUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const remoteTemplates = await response.json() as RemoteTemplate[];
        // 确保是数组
        if (Array.isArray(remoteTemplates)) {
          allTemplates = [...remoteTemplates];
        } else {
          console.warn('远程模板格式不正确，期望数组');
        }
      } catch (e) {
        console.log('加载远程模板失败:', e);
        error.value = '加载远程模板失败，请检查 URL 配置';
      }
    } else {
      error.value = '未配置远程模板 URL';
    }
    
    templates.value = allTemplates;
    
  } catch (err) {
    console.error('加载模板失败:', err);
    error.value = '加载模板失败: ' + err;
    templates.value = [];
  } finally {
    isLoading.value = false;
  }
};

// ==================== 计算属性 ====================

/**
 * 根据搜索关键词过滤模板
 */
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

// ==================== 工具函数 ====================

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

// ==================== 生命周期 ====================

onMounted(() => {
  if (props.show) {
    loadTemplates();
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
        <!-- 背景遮罩 -->
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm" 
          @click="closeModal"
        />
        
        <!-- 弹窗主体 -->
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          
          <!-- 头部 -->
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
                @click="loadTemplates"
                :disabled="isLoading"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
                title="刷新"
              >
                <RefreshCw :size="18" :class="isLoading ? 'animate-spin' : ''" class="text-gray-500" />
              </button>
              <button 
                @click="closeModal"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
              >
                <X :size="20" class="text-gray-500" />
              </button>
            </div>
          </div>
          
          <!-- 搜索区域 -->
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
          
          <!-- 内容区域 -->
          <div class="flex-1 overflow-hidden flex">
            
            <!-- 模板列表 -->
            <div :class="['flex-1 overflow-y-auto p-4', previewMode ? '' : 'w-full']">
              
              <!-- 加载中 -->
              <div v-if="isLoading" class="flex flex-col items-center justify-center h-64">
                <RefreshCw :size="32" class="animate-spin text-blue-500 mb-4" />
                <p class="text-gray-500">加载模板中...</p>
              </div>
              
              <!-- 错误状态 -->
              <div v-else-if="error" class="flex flex-col items-center justify-center h-64 text-center">
                <FileText :size="32" class="text-gray-300 mb-4" />
                <p class="text-red-600 mb-2">{{ error }}</p>
                <button
                  @click="loadTemplates"
                  class="text-blue-600 hover:underline"
                >
                  重试
                </button>
              </div>
              
              <!-- 空状态 -->
              <div v-else-if="filteredTemplates.length === 0" class="flex flex-col items-center justify-center h-64 text-center">
                <FileText :size="32" class="text-gray-300 mb-4" />
                <p class="text-gray-500">没有找到模板</p>
                <p class="text-xs text-gray-400 mt-2">请检查远程模板 URL 配置</p>
              </div>
              
              <!-- 模板列表 -->
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
                  <!-- 模板名称 -->
                  <h3 class="font-semibold text-gray-800 mb-1">{{ template.name }}</h3>
                  <!-- 模板ID -->
                  <p class="text-xs text-gray-400 mb-2">ID: {{ template.id }}</p>
                  <!-- 模板描述 -->
                  <p class="text-sm text-gray-500 line-clamp-2">{{ template.description }}</p>
                </div>
              </div>
            </div>
            
            <!-- 预览面板 -->
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
                
                <!-- 内容预览 -->
                <div class="bg-white rounded-xl border border-gray-200 p-4 mb-4">
                  <pre class="text-xs text-gray-600 overflow-auto max-h-64 whitespace-pre-wrap">{{ selectedTemplate.content.substring(0, 500) }}{{ selectedTemplate.content.length > 500 ? '...' : '' }}</pre>
                </div>
                
                <!-- 使用按钮 -->
                <button
                  @click="confirmUseTemplate"
                  class="w-full py-2.5 bg-gradient-to-r from-blue-500 to-indigo-600 text-white rounded-xl font-medium hover:opacity-90 transition-all flex items-center justify-center gap-2"
                >
                  <Download :size="18" />
                  使用此模板
                </button>
              </div>
            </div>
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

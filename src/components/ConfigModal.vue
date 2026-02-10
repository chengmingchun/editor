<script setup lang="ts">
/**
 * ConfigModal.vue - 配置弹窗组件
 * 
 * 配置项：
 * 1. AI 配置：API 地址、API Key、模型
 * 2. 远程模板 URL
 * 3. 模板上传 URL
 */

import { ref, onMounted } from 'vue';
import { X, Bot, Globe, Save, Key, Sparkles, BookOpen, Upload } from 'lucide-vue-next';

export interface AIConfig {
  enabled: boolean;
  apiUrl: string;
  apiKey: string;
  model: string;
}

export interface AppConfig {
  ai: AIConfig;
  remoteTemplateUrl: string;
  uploadUrl: string;
}

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', config: AppConfig): void;
}>();

const activeTab = ref<'ai' | 'templates'>('ai');

const config = ref<AppConfig>({
  ai: {
    enabled: false,
    apiUrl: '',
    apiKey: '',
    model: 'gpt-4',
  },
  remoteTemplateUrl: '',
  uploadUrl: '',
});

const defaultModels = [
  'gpt-4',
  'gpt-4-turbo',
  'gpt-3.5-turbo',
  'claude-3-opus',
  'claude-3-sonnet',
  'gemini-pro',
];

onMounted(() => {
  const saved = localStorage.getItem('app-config');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      config.value = { ...config.value, ...parsed };
    } catch (e) {
      console.error('加载配置失败:', e);
    }
  }
});

const saveConfig = () => {
  localStorage.setItem('app-config', JSON.stringify(config.value));
  emit('save', config.value);
  emit('close');
};

const resetConfig = () => {
  if (confirm('确定要重置配置吗？')) {
    config.value = {
      ai: {
        enabled: false,
        apiUrl: '',
        apiKey: '',
        model: 'gpt-4',
      },
      remoteTemplateUrl: '',
      uploadUrl: '',
    };
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="$emit('close')" />
        
        <!-- 弹窗主体 -->
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-xl max-h-[90vh] overflow-hidden flex flex-col">
          
          <!-- 头部 -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-indigo-50 to-purple-50">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl">
                <Sparkles :size="20" class="text-white" />
              </div>
              <h2 class="text-lg font-bold text-gray-800">配置</h2>
            </div>
            <button @click="$emit('close')" class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors">
              <X :size="20" class="text-gray-500" />
            </button>
          </div>
          
          <!-- Tab 导航 -->
          <div class="flex border-b border-gray-200 px-6">
            <button
              v-for="tab in [
                { key: 'ai', label: 'AI 配置', icon: Bot },
                { key: 'templates', label: '模板配置', icon: BookOpen },
              ]"
              :key="tab.key"
              @click="activeTab = tab.key as any"
              :class="[
                'flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors',
                activeTab === tab.key
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              ]"
            >
              <component :is="tab.icon" :size="16" />
              {{ tab.label }}
            </button>
          </div>
          
          <!-- 内容区域 -->
          <div class="flex-1 overflow-y-auto p-6">
            
            <!-- AI 配置 Tab -->
            <div v-if="activeTab === 'ai'" class="space-y-5">
              
              <!-- 启用 AI 开关 -->
              <label class="flex items-center justify-between p-4 bg-indigo-50 border border-indigo-200 rounded-xl cursor-pointer hover:bg-indigo-100 transition-colors">
                <div class="flex items-center gap-3">
                  <Bot :size="20" class="text-indigo-600" />
                  <div>
                    <div class="font-medium text-gray-800">启用 AI 功能</div>
                    <div class="text-xs text-gray-600">选中文本后生成 PlantUML</div>
                  </div>
                </div>
                <div class="relative">
                  <input type="checkbox" v-model="config.ai.enabled" class="sr-only peer">
                  <div class="w-11 h-6 bg-gray-300 peer-focus:ring-2 peer-focus:ring-indigo-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                </div>
              </label>
              
              <!-- AI 配置表单 -->
              <div v-if="config.ai.enabled" class="space-y-4 pt-2">
                
                <!-- API 地址 -->
                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-2">
                    <span class="flex items-center gap-1">
                      <Globe :size="14" />
                      API 地址
                    </span>
                  </label>
                  <input
                    v-model="config.ai.apiUrl"
                    type="text"
                    placeholder="https://api.openai.com/v1/chat/completions"
                    class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all font-mono text-sm"
                  />
                </div>
                
                <!-- API Key -->
                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-2">
                    <span class="flex items-center gap-1">
                      <Key :size="14" />
                      API Key
                    </span>
                  </label>
                  <input
                    v-model="config.ai.apiKey"
                    type="password"
                    placeholder="sk-..."
                    class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                  />
                  <p class="text-xs text-gray-500 mt-1">您的 API Key 仅存储在本地</p>
                </div>
                
                <!-- 模型选择 -->
                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-2">模型</label>
                  <div class="flex gap-2">
                    <select
                      v-model="config.ai.model"
                      class="flex-1 px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all bg-white"
                    >
                      <option v-for="model in defaultModels" :key="model" :value="model">
                        {{ model }}
                      </option>
                    </select>
                    <input
                      v-model="config.ai.model"
                      type="text"
                      placeholder="自定义模型"
                      class="flex-1 px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                    />
                  </div>
                </div>
                
              </div>
              
              <!-- 未启用 AI 时的提示 -->
              <div v-else class="p-4 bg-gray-50 rounded-xl border border-gray-200">
                <p class="text-sm text-gray-500">启用 AI 功能后，您可以选中文本并调用 AI 生成 PlantUML 图表。</p>
              </div>
            </div>
            
            <!-- 模板配置 Tab -->
            <div v-else-if="activeTab === 'templates'" class="space-y-5">
              
              <!-- 远程模板 URL -->
              <div class="p-4 bg-blue-50 rounded-xl border border-blue-200">
                <label class="flex items-center gap-2 text-sm font-medium text-gray-800 mb-2">
                  <BookOpen :size="16" class="text-blue-600" />
                  远程模板加载 URL
                </label>
                <input
                  v-model="config.remoteTemplateUrl"
                  type="text"
                  placeholder="https://api.example.com/templates"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all font-mono text-sm"
                />
                <p class="text-xs text-gray-500 mt-2">
                  配置后可以从远程服务器加载模板列表
                </p>
              </div>
              
              <!-- 模板上传 URL -->
              <div class="p-4 bg-indigo-50 rounded-xl border border-indigo-200">
                <label class="flex items-center gap-2 text-sm font-medium text-gray-800 mb-2">
                  <Upload :size="16" class="text-indigo-600" />
                  模板上传 URL
                </label>
                <input
                  v-model="config.uploadUrl"
                  type="text"
                  placeholder="https://api.example.com/templates/upload"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all font-mono text-sm"
                />
                <p class="text-xs text-gray-500 mt-2">
                  配置后可以将当前文档作为新模板上传到服务器
                </p>
              </div>
              
              <!-- 上传格式说明 -->
              <div class="p-4 bg-gray-50 rounded-xl border border-gray-200">
                <p class="text-sm text-gray-600 font-medium mb-2">模板接口格式说明：</p>
                <pre class="text-xs text-gray-500 bg-gray-100 p-3 rounded-lg overflow-x-auto">// GET 远程模板接口返回格式
[
  {
    "id": "模板ID",
    "name": "模板名称", 
    "description": "模板简述",
    "content": "模板内容"
  }
]

// POST 上传模板接口请求格式
{
  "id": "模板ID",
  "name": "模板名称",
  "description": "模板简述", 
  "content": "模板内容"
}</pre>
              </div>
            </div>
          </div>
          
          <!-- 底部按钮 -->
          <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 bg-gray-50">
            <button
              @click="resetConfig"
              class="text-sm text-gray-500 hover:text-gray-700 transition-colors"
            >
              重置
            </button>
            <div class="flex gap-3">
              <button
                @click="$emit('close')"
                class="px-5 py-2.5 text-gray-700 hover:bg-gray-200 rounded-xl transition-colors font-medium"
              >
                取消
              </button>
              <button
                @click="saveConfig"
                class="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-600 text-white rounded-xl hover:opacity-90 transition-all font-medium"
              >
                <Save :size="18" />
                保存配置
              </button>
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
</style>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { X, Bot, Globe, Save, Key, Sparkles, BookOpen, Upload, Terminal } from 'lucide-vue-next';

export interface AIConfig {
  enabled: boolean;
  apiUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  maxTokens: number;
  timeout: number;
}

export interface AppConfig {
  ai: AIConfig;
  remoteTemplateUrl: string;
  uploadUrl: string;
  useCommandLine: boolean;
}

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', config: AppConfig): void;
}>();

const activeTab = ref<'ai' | 'templates' | 'advanced'>('ai');

const config = ref<AppConfig>({
  ai: {
    enabled: false,
    apiUrl: '',
    apiKey: '',
    model: 'gpt-4',
    temperature: 0.7,
    maxTokens: 2000,
    timeout: 30,
  },
  remoteTemplateUrl: '',
  uploadUrl: '',
  useCommandLine: false,
});

const defaultModels = [
  'gpt-4',
  'gpt-4-turbo',
  'gpt-4o',
  'gpt-3.5-turbo',
  'claude-3-opus-2024-05-20',
  'claude-3-sonnet-2024-02-29',
  'claude-3-haiku-2024-03-20',
  'gemini-1.5-pro',
  'gemini-1.5-flash',
];

const testConnection = async () => {
  if (!config.value.ai.apiUrl || !config.value.ai.apiKey) {
    alert('请先填写 API 地址和 Key');
    return;
  }
  
  alert('测试连接功能需要使用命令行调用，请确保已配置正确的 API 信息');
};

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
        temperature: 0.7,
        maxTokens: 2000,
        timeout: 30,
      },
      remoteTemplateUrl: '',
      uploadUrl: '',
      useCommandLine: false,
    };
  }
};

const testApiConnection = async () => {
  alert('API 测试功能将通过 Rust 命令行调用 curl 实现');
};
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="$emit('close')" />
        
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-indigo-50 to-purple-50">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl">
                <Sparkles :size="20" class="text-white" />
              </div>
              <h2 class="text-lg font-bold text-gray-800">编辑器设置</h2>
            </div>
            <button @click="$emit('close')" class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors">
              <X :size="20" class="text-gray-500" />
            </button>
          </div>
          
          <div class="flex border-b border-gray-200 px-6">
            <button
              v-for="tab in [
                { key: 'ai', label: 'AI 配置', icon: Bot },
                { key: 'templates', label: '模板配置', icon: BookOpen },
                { key: 'advanced', label: '高级选项', icon: Terminal },
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
          
          <div class="flex-1 overflow-y-auto p-6">
            <div v-if="activeTab === 'ai'" class="space-y-6">
              <label class="flex items-center justify-between p-4 bg-indigo-50 border border-indigo-200 rounded-xl cursor-pointer hover:bg-indigo-100 transition-colors">
                <div class="flex items-center gap-3">
                  <Bot :size="20" class="text-indigo-600" />
                  <div>
                    <div class="font-medium text-gray-800">启用 AI 功能</div>
                    <div class="text-xs text-gray-600">启用后可在编辑器中调用 AI 生成图表</div>
                  </div>
                </div>
                <div class="relative">
                  <input type="checkbox" v-model="config.ai.enabled" class="sr-only peer">
                  <div class="w-11 h-6 bg-gray-300 peer-focus:ring-2 peer-focus:ring-indigo-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                </div>
              </label>
              
              <div v-if="config.ai.enabled" class="space-y-4">
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
                  <p class="text-xs text-gray-500 mt-1">API Key 仅存储在本地，不会发送给任何第三方</p>
                </div>
                
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">模型</label>
                    <select
                      v-model="config.ai.model"
                      class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all bg-white"
                    >
                      <option v-for="model in defaultModels" :key="model" :value="model">
                        {{ model }}
                      </option>
                    </select>
                  </div>
                  
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Temperature</label>
                    <input
                      v-model.number="config.ai.temperature"
                      type="number"
                      min="0"
                      max="2"
                      step="0.1"
                      class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                    />
                  </div>
                </div>
                
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">最大 Token</label>
                    <input
                      v-model.number="config.ai.maxTokens"
                      type="number"
                      min="100"
                      max="128000"
                      step="100"
                      class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                    />
                  </div>
                  
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">超时时间(秒)</label>
                    <input
                      v-model.number="config.ai.timeout"
                      type="number"
                      min="5"
                      max="300"
                      class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                    />
                  </div>
                </div>
                
                <button
                  @click="testApiConnection"
                  class="w-full py-2.5 px-4 rounded-xl bg-indigo-100 text-indigo-700 hover:bg-indigo-200 transition-colors flex items-center justify-center gap-2"
                >
                  <Terminal :size="16" />
                  测试 API 连接
                </button>
              </div>
            </div>
            
            <div v-else-if="activeTab === 'templates'" class="space-y-6">
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
              
              <div class="p-4 bg-gray-50 rounded-xl border border-gray-200">
                <p class="text-sm text-gray-600 font-medium mb-2">模板接口格式说明：</p>
                <pre class="text-xs text-gray-500 bg-gray-100 p-3 rounded-lg overflow-x-auto">GET 远程模板接口返回格式:
[
  {"id": "模板ID", "name": "名称", "description": "描述", "content": "内容"}
]

POST 上传模板接口请求格式:
{"id": "ID", "name": "名称", "description": "描述", "content": "内容"}</pre>
              </div>
            </div>
            
            <div v-else-if="activeTab === 'advanced'" class="space-y-6">
              <div class="p-4 bg-amber-50 rounded-xl border border-amber-200">
                <h3 class="text-sm font-medium text-gray-800 mb-2">命令行调用配置</h3>
                <p class="text-xs text-gray-600 mb-4">
                  由于 Tauri 安全限制，无法直接发送 HTTP 请求。启用此选项后，将通过命令行调用 curl 等工具发送 API 请求。
                </p>
                <label class="flex items-center gap-3">
                  <input 
                    type="checkbox" 
                    v-model="config.useCommandLine"
                    class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500"
                  />
                  <span class="text-sm text-gray-700">使用命令行调用 HTTP 请求</span>
                </label>
              </div>
              
              <div class="p-4 bg-gray-50 rounded-xl border border-gray-200">
                <h3 class="text-sm font-medium text-gray-800 mb-3">系统要求</h3>
                <ul class="text-xs text-gray-600 space-y-2">
                  <li class="flex items-center gap-2">
                    <span class="w-1.5 h-1.5 bg-green-500 rounded-full"></span>
                    Windows: 已内置 curl
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="w-1.5 h-1.5 bg-green-500 rounded-full"></span>
                    macOS: 已内置 curl
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="w-1.5 h-1.5 bg-green-500 rounded-full"></span>
                    Linux: 已内置 curl
                  </li>
                </ul>
              </div>
              
              <div class="p-4 bg-blue-50 rounded-xl border border-blue-200">
                <h3 class="text-sm font-medium text-gray-800 mb-2">当前配置摘要</h3>
                <div class="text-xs text-gray-600 space-y-1">
                  <p>AI 功能: {{ config.ai.enabled ? '已启用' : '已禁用' }}</p>
                  <p>使用的模型: {{ config.ai.model || '未配置' }}</p>
                  <p>模板 URL: {{ config.remoteTemplateUrl || '未配置' }}</p>
                  <p>请求方式: {{ config.useCommandLine ? '命令行 (curl)' : '直接 HTTP' }}</p>
                </div>
              </div>
            </div>
          </div>
          
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

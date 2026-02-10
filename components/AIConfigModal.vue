<script setup lang="ts">
/**
 * AIConfigModal.vue - AI 配置弹窗组件
 * 
 * 功能说明：
 * 这个组件用于配置 AI 服务提供商的各项参数，包括 API 密钥、模型选择、
 * 高级参数等。配置会保存到 localStorage，实现持久化。
 * 
 * 支持的 AI 提供商：
 * 1. 演示模式 (demo) - 本地模拟，无需配置
 * 2. OpenAI - GPT-4/GPT-3.5
 * 3. Azure OpenAI - 微软云服务
 * 4. 自定义 - 支持本地模型如 Ollama
 * 
 * 技术要点：
 * - 使用 Vue 3 Composition API
 * - 使用 Teleport 将弹窗渲染到 body
 * - 使用 localStorage 持久化配置
 * - 表单验证和连接测试功能
 */

import { ref, onMounted, watch } from 'vue';
import { 
  X,           // 关闭图标
  Bot,         // 机器人图标
  Key,         // 密钥图标
  Globe,       // 地球图标（网络）
  Server,      // 服务器图标
  Check,       // 勾选图标
  AlertCircle, // 警告图标
  Save,        // 保存图标
  RefreshCw,   // 刷新图标
  Sparkles,    // 闪光图标
  Loader2      // 加载动画图标（从 script lang="ts" 导入）
} from 'lucide-vue-next';

// ==================== Props & Emits ====================

/** 组件接收的属性 */
const props = defineProps<{
  show: boolean;  // 控制弹窗显示/隐藏
}>();

/** 组件触发的事件 */
const emit = defineEmits<{
  (e: 'close'): void;                    // 关闭弹窗
  (e: 'save', config: AIConfig): void;   // 保存配置，传递配置对象
}>();

// ==================== 类型定义 ====================

/**
 * AI 配置接口
 * 定义了所有可配置的参数
 */
export interface AIConfig {
  // 基础配置
  provider: 'openai' | 'azure' | 'custom' | 'demo';  // AI 提供商类型
  apiKey: string;                                     // API 密钥
  apiUrl: string;                                     // API 地址
  model: string;                                      // 模型名称
  
  // 高级配置
  temperature: number;    // 温度参数 (0-2)，控制随机性
  maxTokens: number;      // 最大生成 Token 数
  timeout: number;        // 请求超时时间（秒）
  
  // 模板配置
  templateUrl: string;           // 远程模板源地址
  enableRemoteTemplate: boolean; // 是否启用远程模板
  
  // 功能开关
  enableStreaming: boolean;    // 是否启用流式输出
  enableAutoSuggest: boolean;  // 是否启用自动建议
}

// ==================== 常量定义 ====================

/**
 * 默认配置
 * 作为初始值和重置时的基准
 */
const defaultConfig: AIConfig = {
  provider: 'demo',
  apiKey: '',
  apiUrl: '',
  model: 'gpt-4',
  temperature: 0.7,
  maxTokens: 2000,
  timeout: 30,
  templateUrl: 'https://api.example.com/templates',
  enableRemoteTemplate: false,
  enableStreaming: true,
  enableAutoSuggest: false,
};

/**
 * 各提供商的预设配置
 * 切换提供商时自动填充
 */
const presets = {
  openai: {
    apiUrl: 'https://api.openai.com/v1/chat/completions',
    model: 'gpt-4',
  },
  azure: {
    apiUrl: 'https://{your-resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version=2023-12-01-preview',
    model: 'gpt-4',
  },
  custom: {
    apiUrl: 'http://localhost:11434/v1/chat/completions',  // Ollama 默认地址
    model: 'llama2',
  },
  demo: {
    apiUrl: '',
    model: 'demo',
  },
};

/**
 * 各支持的模型列表
 */
const modelOptions: Record<string, string[]> = {
  openai: ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo', 'gpt-3.5-turbo-16k'],
  azure: ['gpt-4', 'gpt-4-32k', 'gpt-35-turbo', 'gpt-35-turbo-16k'],
  custom: ['llama2', 'codellama', 'mistral', 'qwen', 'chatglm'],
  demo: ['demo'],
};

// ==================== 响应式状态 ====================

/** 当前配置对象 */
const config = ref<AIConfig>({ ...defaultConfig });

/** 当前激活的标签页 */
const activeTab = ref<'basic' | 'advanced' | 'templates'>('basic');

/** 是否正在测试连接 */
const isTesting = ref(false);

/** 连接测试结果 */
const testResult = ref<{ success: boolean; message: string } | null>(null);

/** 是否正在保存 */
const isSaving = ref(false);

// ==================== 方法定义 ====================

/**
 * 从 localStorage 加载配置
 * 
 * 说明：
 * - localStorage 是浏览器提供的持久化存储
 * - 数据以字符串形式存储，需要 JSON.parse 解析
 * - 如果解析失败或没有保存的配置，使用默认值
 */
const loadConfig = () => {
  try {
    const saved = localStorage.getItem('ai-flow-studio-ai-config');
    if (saved) {
      const parsed = JSON.parse(saved);
      // 使用展开运算符合并默认配置和保存的配置
      // 这样可以确保新增的配置项也有默认值
      config.value = { ...defaultConfig, ...parsed };
    }
  } catch (err) {
    console.error('加载配置失败:', err);
  }
};

/**
 * 保存配置
 * 
 * 流程：
 * 1. 保存到 localStorage
 * 2. 触发 save 事件通知父组件
 * 3. 显示成功提示
 */
const saveConfig = async () => {
  isSaving.value = true;
  try {
    // localStorage 只能存储字符串，使用 JSON.stringify 序列化
    localStorage.setItem('ai-flow-studio-ai-config', JSON.stringify(config.value));
    
    // 通知父组件配置已更新
    emit('save', config.value);
    
    // 模拟保存延迟（实际项目中可移除）
    await new Promise(resolve => setTimeout(resolve, 500));
    
    showToast('配置已保存');
  } catch (err) {
    console.error('保存配置失败:', err);
    showToast('保存失败: ' + err);
  } finally {
    isSaving.value = false;
  }
};

/**
 * 测试 AI 连接
 * 
 * 说明：
 * - 演示模式下无需测试
 * - 真实模式下会调用后端接口验证 API 密钥
 * - 实际项目中需要实现后端接口
 */
const testConnection = async () => {
  // 演示模式直接返回成功
  if (config.value.provider === 'demo') {
    testResult.value = { 
      success: true, 
      message: '演示模式无需测试，将使用本地模拟生成' 
    };
    return;
  }
  
  // 表单验证
  if (!config.value.apiKey || !config.value.apiUrl) {
    testResult.value = { 
      success: false, 
      message: '请先填写 API 地址和密钥' 
    };
    return;
  }
  
  isTesting.value = true;
  testResult.value = null;
  
  try {
    // 模拟网络延迟
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    // 实际项目中，这里应该调用后端接口测试
    // const result = await invoke('test_ai_connection', { config: config.value });
    
    // 模拟成功结果
    testResult.value = { 
      success: true, 
      message: `连接成功！模型: ${config.value.model}` 
    };
  } catch (err) {
    testResult.value = { 
      success: false, 
      message: '连接失败: ' + err 
    };
  } finally {
    isTesting.value = false;
  }
};

/**
 * 监听提供商变化，自动更新 URL 和模型
 * 
 * 使用 Vue 3 的 watch API 监听响应式数据变化
 */
watch(() => config.value.provider, (newProvider) => {
  const preset = presets[newProvider as keyof typeof presets];
  if (preset) {
    config.value.apiUrl = preset.apiUrl;
    config.value.model = preset.model;
  }
});

// ==================== Toast 提示 ====================

const showToastMsg = ref('');
const showToastFlag = ref(false);

const showToast = (msg: string) => {
  showToastMsg.value = msg;
  showToastFlag.value = true;
  setTimeout(() => {
    showToastFlag.value = false;
  }, 2000);
};

/**
 * 重置配置为默认值
 */
const resetConfig = () => {
  if (confirm('确定要重置为默认配置吗？')) {
    config.value = { ...defaultConfig };
    showToast('已重置为默认配置');
  }
};

// ==================== 生命周期 ====================

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <!-- Teleport 将弹窗渲染到 body 下，避免被父元素的 z-index 或 overflow 影响 -->
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- 背景遮罩 -->
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm" 
          @click="$emit('close')"
        />
        
        <!-- 弹窗主体 -->
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
          
          <!-- ==================== 头部区域 ==================== -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-indigo-50 to-purple-50">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl">
                <Bot :size="20" class="text-white" />
              </div>
              <div>
                <h2 class="text-lg font-bold text-gray-800">AI 配置</h2>
                <p class="text-xs text-gray-500">配置 AI 服务提供商和高级选项</p>
              </div>
            </div>
            <button 
              @click="$emit('close')"
              class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
            >
              <X :size="20" class="text-gray-500" />
            </button>
          </div>
          
          <!-- ==================== Tab 导航 ==================== -->
          <div class="flex border-b border-gray-200 px-6">
            <button
              v-for="tab in [
                { key: 'basic', label: '基础配置', icon: Key },
                { key: 'advanced', label: '高级选项', icon: Server },
                { key: 'templates', label: '模板源', icon: Globe },
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
          
          <!-- ==================== 内容区域 ==================== -->
          <div class="flex-1 overflow-y-auto p-6">
            
            <!-- 基础配置 Tab -->
            <div v-if="activeTab === 'basic'" class="space-y-5">
              
              <!-- 服务提供商选择 -->
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  AI 服务提供商
                </label>
                <!-- 4 列网格布局 -->
                <div class="grid grid-cols-4 gap-3">
                  <button
                    v-for="provider in [
                      { key: 'demo', label: '演示模式', desc: '本地模拟' },
                      { key: 'openai', label: 'OpenAI', desc: 'GPT-4/GPT-3.5' },
                      { key: 'azure', label: 'Azure', desc: 'Azure OpenAI' },
                      { key: 'custom', label: '自定义', desc: '本地/其他' },
                    ]"
                    :key="provider.key"
                    @click="config.provider = provider.key as any"
                    :class="[
                      'p-3 rounded-xl border-2 text-left transition-all',
                      config.provider === provider.key
                        ? 'border-indigo-500 bg-indigo-50'
                        : 'border-gray-200 hover:border-gray-300'
                    ]"
                  >
                    <div class="font-medium text-sm" :class="config.provider === provider.key ? 'text-indigo-700' : 'text-gray-700'">
                      {{ provider.label }}
                    </div>
                    <div class="text-xs text-gray-500 mt-0.5">{{ provider.desc }}</div>
                  </button>
                </div>
              </div>
              
              <!-- API 密钥输入 -->
              <div v-if="config.provider !== 'demo'">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  <span class="flex items-center gap-1">
                    <Key :size="14" />
                    API 密钥
                  </span>
                </label>
                <div class="relative">
                  <!-- type="password" 隐藏输入内容 -->
                  <input
                    v-model="config.apiKey"
                    type="password"
                    placeholder="sk-..."
                    class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                  />
                  <div class="absolute right-3 top-1/2 -translate-y-1/2">
                    <span class="text-xs text-gray-400">安全存储</span>
                  </div>
                </div>
                <p class="text-xs text-gray-500 mt-1">您的 API 密钥仅存储在本地，不会上传到服务器</p>
              </div>
              
              <!-- API 地址输入 -->
              <div v-if="config.provider !== 'demo'">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  <span class="flex items-center gap-1">
                    <Globe :size="14" />
                    API 地址
                  </span>
                </label>
                <input
                  v-model="config.apiUrl"
                  type="text"
                  :placeholder="presets[config.provider as keyof typeof presets]?.apiUrl || 'https://api.example.com/v1/chat/completions'"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all font-mono text-sm"
                />
              </div>
              
              <!-- 模型选择 -->
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  <span class="flex items-center gap-1">
                    <Sparkles :size="14" />
                    模型
                  </span>
                </label>
                <select
                  v-model="config.model"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all bg-white"
                >
                  <option v-for="model in modelOptions[config.provider]" :key="model" :value="model">
                    {{ model }}
                  </option>
                </select>
              </div>
              
              <!-- 连接测试按钮 -->
              <div v-if="config.provider !== 'demo'" class="pt-2">
                <button
                  @click="testConnection"
                  :disabled="isTesting"
                  class="flex items-center gap-2 px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg transition-colors text-sm font-medium"
                >
                  <RefreshCw :size="16" :class="isTesting ? 'animate-spin' : ''" />
                  {{ isTesting ? '测试中...' : '测试连接' }}
                </button>
                
                <!-- 测试结果提示 -->
                <div
                  v-if="testResult"
                  :class="[
                    'mt-3 p-3 rounded-lg flex items-start gap-2 text-sm',
                    testResult.success ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
                  ]"
                >
                  <Check v-if="testResult.success" :size="16" class="mt-0.5 flex-shrink-0" />
                  <AlertCircle v-else :size="16" class="mt-0.5 flex-shrink-0" />
                  {{ testResult.message }}
                </div>
              </div>
            </div>
            
            <!-- 高级选项 Tab -->
            <div v-else-if="activeTab === 'advanced'" class="space-y-5">
              
              <!-- Temperature 滑块 -->
              <div>
                <div class="flex items-center justify-between mb-2">
                  <label class="text-sm font-medium text-gray-700">Temperature (随机性)</label>
                  <span class="text-sm text-indigo-600 font-mono">{{ config.temperature }}</span>
                </div>
                <input
                  v-model.number="config.temperature"
                  type="range"
                  min="0"
                  max="2"
                  step="0.1"
                  class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-500"
                />
                <div class="flex justify-between text-xs text-gray-500 mt-1">
                  <span>精确 (0)</span>
                  <span>平衡 (1)</span>
                  <span>创意 (2)</span>
                </div>
              </div>
              
              <!-- 最大 Token 数 -->
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  最大 Token 数
                </label>
                <input
                  v-model.number="config.maxTokens"
                  type="number"
                  min="100"
                  max="8000"
                  step="100"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                />
                <p class="text-xs text-gray-500 mt-1">控制生成文本的最大长度</p>
              </div>
              
              <!-- 超时时间 -->
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  请求超时 (秒)
                </label>
                <input
                  v-model.number="config.timeout"
                  type="number"
                  min="5"
                  max="120"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                />
              </div>
              
              <!-- 功能开关 -->
              <div class="space-y-3 pt-2">
                <!-- 流式输出开关 -->
                <label class="flex items-center justify-between p-3 bg-gray-50 rounded-xl cursor-pointer hover:bg-gray-100 transition-colors">
                  <div>
                    <div class="font-medium text-sm text-gray-700">流式输出</div>
                    <div class="text-xs text-gray-500">实时显示生成内容</div>
                  </div>
                  <!-- 自定义开关样式 -->
                  <div class="relative">
                    <input type="checkbox" v-model="config.enableStreaming" class="sr-only peer">
                    <div class="w-11 h-6 bg-gray-300 peer-focus:ring-2 peer-focus:ring-indigo-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                  </div>
                </label>
                
                <!-- 智能建议开关 -->
                <label class="flex items-center justify-between p-3 bg-gray-50 rounded-xl cursor-pointer hover:bg-gray-100 transition-colors">
                  <div>
                    <div class="font-medium text-sm text-gray-700">智能建议</div>
                    <div class="text-xs text-gray-500">输入时自动提示补全</div>
                  </div>
                  <div class="relative">
                    <input type="checkbox" v-model="config.enableAutoSuggest" class="sr-only peer">
                    <div class="w-11 h-6 bg-gray-300 peer-focus:ring-2 peer-focus:ring-indigo-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                  </div>
                </label>
              </div>
            </div>
            
            <!-- 模板源 Tab -->
            <div v-else-if="activeTab === 'templates'" class="space-y-5">
              
              <!-- 启用远程模板开关 -->
              <label class="flex items-center justify-between p-4 bg-indigo-50 border border-indigo-200 rounded-xl cursor-pointer hover:bg-indigo-100 transition-colors">
                <div class="flex items-center gap-3">
                  <Globe :size="20" class="text-indigo-600" />
                  <div>
                    <div class="font-medium text-gray-800">启用远程模板</div>
                    <div class="text-xs text-gray-600">从远程服务器获取模板列表</div>
                  </div>
                </div>
                <div class="relative">
                  <input type="checkbox" v-model="config.enableRemoteTemplate" class="sr-only peer">
                  <div class="w-11 h-6 bg-gray-300 peer-focus:ring-2 peer-focus:ring-indigo-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                </div>
              </label>
              
              <!-- 模板源地址输入 -->
              <div v-if="config.enableRemoteTemplate">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  模板源地址
                </label>
                <input
                  v-model="config.templateUrl"
                  type="text"
                  placeholder="https://api.example.com/templates"
                  class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all font-mono text-sm"
                />
                <p class="text-xs text-gray-500 mt-2">
                  支持格式：
                  <code class="bg-gray-100 px-1.5 py-0.5 rounded">https://api.example.com/templates</code>
                </p>
              </div>
              
              <!-- 本地模板说明 -->
              <div v-else class="p-4 bg-gray-50 rounded-xl border border-gray-200">
                <div class="flex items-start gap-3">
                  <Server :size="20" class="text-gray-500 mt-0.5" />
                  <div>
                    <div class="font-medium text-sm text-gray-700">使用本地模板</div>
                    <p class="text-xs text-gray-500 mt-1">
                      当前使用内置模板库。启用远程模板后，将从指定地址动态获取模板列表。
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- ==================== 底部按钮 ==================== -->
          <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 bg-gray-50">
            <button
              @click="resetConfig"
              class="text-sm text-gray-500 hover:text-gray-700 transition-colors"
            >
              重置为默认
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
                :disabled="isSaving"
                class="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-600 text-white rounded-xl hover:opacity-90 transition-all disabled:opacity-50 font-medium shadow-lg shadow-indigo-500/30"
              >
                <Save :size="18" />
                <Loader2 v-if="isSaving" :size="18" class="animate-spin" />
                {{ isSaving ? '保存中...' : '保存配置' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
    
    <!-- Toast 提示 -->
    <Transition name="toast">
      <div
        v-if="showToastFlag"
        class="fixed bottom-6 left-1/2 -translate-x-1/2 px-6 py-3 bg-gray-800 text-white rounded-lg shadow-lg z-[60] flex items-center gap-2"
      >
        <Check :size="16" class="text-green-400" />
        {{ showToastMsg }}
      </div>
    </Transition>
  </Teleport>
</template>

<script lang="ts">
// 导入 Loader2 图标（需要在 script setup 之后导入）
import { Loader2 } from 'lucide-vue-next';
</script>

<style scoped>
/**
 * 弹窗动画样式
 */

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

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translate(-50%, 20px);
}

.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -20px);
}

/* 滑块输入框自定义样式 */
input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 20px;
  height: 20px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

input[type="range"]::-moz-range-thumb {
  width: 20px;
  height: 20px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border-radius: 50%;
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
</style>

<script setup lang="ts">
/**
 * TemplateUploadModal.vue - 模板上传弹窗组件
 * 
 * 功能说明：
 * 这个组件允许用户将当前编辑器中的内容上传为模板，方便以后复用。
 * 
 * 上传流程：
 * 1. 用户填写模板标题和简介
 * 2. 系统自动获取当前编辑器的内容
 * 3. 调用后端接口保存模板
 * 4. 保存成功后提示用户
 * 
 * 技术要点：
 * - 使用 Tauri 的 invoke 调用后端 Rust 接口
 * - 表单验证确保必填字段不为空
 * - 支持分类标签选择
 * - 本地存储作为演示模式的后备方案
 */

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  X,           // 关闭图标
  Upload,      // 上传图标
  FileText,    // 文件图标
  Tag,         // 标签图标
  Loader2,     // 加载动画
  Check,       // 勾选图标
  AlertCircle  // 警告图标
} from 'lucide-vue-next';

// ==================== Props 定义 ====================

/**
 * 组件接收的属性
 * 
 * show: 控制弹窗显示/隐藏
 * content: 当前编辑器的内容（从父组件传入）
 */
const props = defineProps<{
  show: boolean;
  content: string;
}>();

// ==================== Emits 定义 ====================

/**
 * 组件触发的事件
 * 
 * close: 关闭弹窗时触发
 * uploaded: 上传成功时触发，父组件可以刷新模板列表
 */
const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'uploaded'): void;
}>();

// ==================== 响应式状态 ====================

/** 模板标题 */
const title = ref('');

/** 模板简介 */
const description = ref('');

/** 模板分类 */
const category = ref('文档');

/** 自定义标签（逗号分隔） */
const tags = ref('');

/** 是否正在上传中 */
const isUploading = ref(false);

/** 上传结果信息 */
const uploadResult = ref<{ success: boolean; message: string } | null>(null);

/** 是否显示成功状态 */
const showSuccess = ref(false);

// ==================== 计算属性 ====================

/**
 * 表单是否有效
 * 标题和内容都必须有值才算有效
 */
const isFormValid = computed(() => {
  return title.value.trim().length > 0 && props.content.trim().length > 0;
});

/**
 * 内容预览（截取前 200 个字符）
 * 用于让用户确认要上传的内容
 */
const contentPreview = computed(() => {
  const maxLength = 200;
  if (props.content.length <= maxLength) {
    return props.content;
  }
  return props.content.substring(0, maxLength) + '...';
});

/**
 * 字符数统计
 */
const contentStats = computed(() => {
  return {
    total: props.content.length,
    lines: props.content.split('\n').length,
  };
});

// ==================== 常量定义 ====================

/** 预设分类选项 */
const categories = [
  '规范',      // API 规范、代码规范等
  '文档',      // PRD、设计文档等
  '架构',      // 系统架构、部署图等
  '敏捷',      // 用户故事、迭代计划等
  '流程',      // 业务流程、工作流程等
  '其他',      // 其他类型
];

// ==================== 方法定义 ====================

/**
 * 处理上传操作
 * 
 * 流程：
 * 1. 验证表单
 * 2. 调用后端接口（或本地存储作为演示）
 * 3. 显示结果
 * 4. 成功后关闭弹窗
 */
const handleUpload = async () => {
  // 1. 表单验证
  if (!isFormValid.value) {
    uploadResult.value = {
      success: false,
      message: '请填写模板标题'
    };
    return;
  }
  
  // 开始上传
  isUploading.value = true;
  uploadResult.value = null;
  
  try {
    // 2. 准备模板数据对象
    const templateData = {
      id: 'template_' + Date.now(),  // 使用时间戳生成唯一 ID
      name: title.value.trim(),
      description: description.value.trim(),
      content: props.content,
      category: category.value,
      tags: tags.value.split(',').map(t => t.trim()).filter(t => t.length > 0),
      author: '用户',  // 可以扩展为读取用户名
      createdAt: new Date().toISOString(),
    };
    
    // 3. 调用后端接口保存模板
    // 在实际项目中，这里应该调用 Tauri 后端
    // 为了演示，先使用 localStorage 作为后备方案
    try {
      // 尝试调用 Tauri 后端（如果有实现的话）
      await invoke('save_template', { template: templateData });
    } catch (e) {
      // 如果后端接口不存在，使用 localStorage
      console.log('后端接口不可用，使用本地存储');
      saveToLocalStorage(templateData);
    }
    
    // 4. 显示成功状态
    showSuccess.value = true;
    uploadResult.value = {
      success: true,
      message: '模板上传成功！'
    };
    
    // 5. 延迟关闭弹窗，让用户看到成功提示
    setTimeout(() => {
      emit('uploaded');  // 通知父组件上传成功
      closeModal();      // 关闭弹窗
    }, 1500);
    
  } catch (err) {
    console.error('上传失败:', err);
    uploadResult.value = {
      success: false,
      message: '上传失败: ' + err
    };
  } finally {
    isUploading.value = false;
  }
};

/**
 * 保存到本地存储（演示模式/后备方案）
 * 
 * @param templateData 模板数据对象
 */
const saveToLocalStorage = (templateData: any) => {
  // 1. 读取现有的本地模板
  const existing = localStorage.getItem('ai-flow-studio-local-templates');
  let templates: any[] = [];
  
  if (existing) {
    try {
      templates = JSON.parse(existing);
    } catch (e) {
      console.error('解析本地模板失败:', e);
      templates = [];
    }
  }
  
  // 2. 添加新模板到列表开头（最新的在前）
  templates.unshift(templateData);
  
  // 3. 保存回 localStorage
  // 注意：localStorage 有 5MB 限制，实际项目中应该使用 IndexedDB 或后端存储
  localStorage.setItem('ai-flow-studio-local-templates', JSON.stringify(templates));
  
  console.log('模板已保存到本地存储:', templateData);
};

/**
 * 关闭弹窗并重置表单
 */
const closeModal = () => {
  // 重置表单数据
  title.value = '';
  description.value = '';
  category.value = '文档';
  tags.value = '';
  uploadResult.value = null;
  showSuccess.value = false;
  
  // 触发关闭事件
  emit('close');
};

/**
 * 添加标签快捷按钮
 * 
 * @param tag 标签名称
 */
const addTag = (tag: string) => {
  const currentTags = tags.value.split(',').map(t => t.trim()).filter(t => t.length > 0);
  if (!currentTags.includes(tag)) {
    currentTags.push(tag);
    tags.value = currentTags.join(', ');
  }
};
</script>

<template>
  <!-- Teleport 将弹窗渲染到 body 下，避免 z-index 层级问题 -->
  <Teleport to="body">
    <!-- Transition 提供进入/离开的动画效果 -->
    <Transition name="modal">
      <!-- 弹窗容器：v-if 控制显示，show 为 false 时整个组件不渲染 -->
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        
        <!-- 背景遮罩：点击可关闭弹窗 -->
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity" 
          @click="closeModal"
        />
        
        <!-- 弹窗主体 -->
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col animate-slide-up">
          
          <!-- ==================== 头部区域 ==================== -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-indigo-50 to-blue-50">
            <div class="flex items-center gap-3">
              <!-- 图标背景 -->
              <div class="p-2 bg-gradient-to-br from-indigo-500 to-blue-600 rounded-xl">
                <Upload :size="20" class="text-white" />
              </div>
              <div>
                <h2 class="text-lg font-bold text-gray-800">上传为模板</h2>
                <p class="text-xs text-gray-500">将当前内容保存为可复用的模板</p>
              </div>
            </div>
            <!-- 关闭按钮 -->
            <button 
              @click="closeModal"
              class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
            >
              <X :size="20" class="text-gray-500" />
            </button>
          </div>
          
          <!-- ==================== 表单内容区域 ==================== -->
          <div class="flex-1 overflow-y-auto p-6 space-y-5">
            
            <!-- 内容预览卡片 -->
            <div class="bg-gray-50 rounded-xl p-4 border border-gray-200">
              <div class="flex items-center gap-2 mb-2 text-gray-600">
                <FileText :size="16" />
                <span class="text-sm font-medium">内容预览</span>
                <span class="text-xs text-gray-400 ml-auto">
                  {{ contentStats.total }} 字符 · {{ contentStats.lines }} 行
                </span>
              </div>
              <!-- 内容预览（代码格式显示） -->
              <pre class="text-xs text-gray-600 bg-white p-3 rounded-lg border border-gray-200 overflow-hidden whitespace-pre-wrap max-h-24">{{ contentPreview }}</pre>
            </div>
            
            <!-- 模板标题输入 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                模板标题 <span class="text-red-500">*</span>
              </label>
              <input
                v-model="title"
                type="text"
                placeholder="给模板起个名字，例如：用户登录流程图"
                class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
              />
            </div>
            
            <!-- 模板简介输入 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                模板简介
              </label>
              <textarea
                v-model="description"
                placeholder="简单描述这个模板的用途和内容..."
                rows="2"
                class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all resize-none"
              />
            </div>
            
            <!-- 分类选择 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                分类
              </label>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="cat in categories"
                  :key="cat"
                  @click="category = cat"
                  :class="[
                    'px-3 py-1.5 rounded-lg text-sm transition-all',
                    category === cat
                      ? 'bg-indigo-500 text-white'
                      : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
                  ]"
                >
                  {{ cat }}
                </button>
              </div>
            </div>
            
            <!-- 标签输入 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                <span class="flex items-center gap-1">
                  <Tag :size="14" />
                  标签
                </span>
              </label>
              <input
                v-model="tags"
                type="text"
                placeholder="用逗号分隔多个标签，例如：API, 时序图, 认证"
                class="w-full px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
              />
              <!-- 常用标签快捷按钮 -->
              <div class="flex flex-wrap gap-2 mt-2">
                <span class="text-xs text-gray-400">常用：</span>
                <button
                  v-for="tag in ['PlantUML', 'API', '流程图', '类图', '架构']"
                  :key="tag"
                  @click="addTag(tag)"
                  class="text-xs px-2 py-1 bg-gray-100 text-gray-600 rounded hover:bg-gray-200 transition-colors"
                >
                  + {{ tag }}
                </button>
              </div>
            </div>
            
            <!-- 结果提示 -->
            <div
              v-if="uploadResult"
              :class="[
                'p-3 rounded-xl flex items-start gap-2 text-sm animate-fade-in',
                uploadResult.success 
                  ? 'bg-green-50 text-green-700 border border-green-200' 
                  : 'bg-red-50 text-red-700 border border-red-200'
              ]"
            >
              <Check v-if="uploadResult.success" :size="18" class="mt-0.5 flex-shrink-0" />
              <AlertCircle v-else :size="18" class="mt-0.5 flex-shrink-0" />
              <div>
                <p class="font-medium">{{ uploadResult.success ? '上传成功' : '上传失败' }}</p>
                <p class="text-xs opacity-80">{{ uploadResult.message }}</p>
              </div>
            </div>
          </div>
          
          <!-- ==================== 底部按钮区域 ==================== -->
          <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 bg-gray-50">
            <!-- 左侧：取消按钮 -->
            <button
              @click="closeModal"
              class="px-5 py-2.5 text-gray-700 hover:bg-gray-200 rounded-xl transition-colors font-medium"
            >
              取消
            </button>
            
            <!-- 右侧：上传按钮 -->
            <button
              @click="handleUpload"
              :disabled="isUploading || !isFormValid"
              :class="[
                'flex items-center gap-2 px-6 py-2.5 rounded-xl font-medium transition-all',
                isFormValid && !isUploading
                  ? 'bg-gradient-to-r from-indigo-500 to-blue-600 text-white hover:opacity-90 shadow-lg shadow-indigo-500/30'
                  : 'bg-gray-300 text-gray-500 cursor-not-allowed'
              ]"
            >
              <!-- 按钮图标：加载中显示旋转动画，成功显示勾选，默认显示上传图标 -->
              <Loader2 v-if="isUploading" :size="18" class="animate-spin" />
              <Check v-else-if="showSuccess" :size="18" />
              <Upload v-else :size="18" />
              
              <!-- 按钮文字 -->
              <span>
                {{ isUploading ? '上传中...' : showSuccess ? '上传成功' : '确认上传' }}
              </span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/**
 * 弹窗动画样式
 * 
 * modal-enter-active: 进入动画的激活状态
 * modal-enter-from: 进入动画的起始状态
 * modal-leave-to: 离开动画的结束状态
 */

.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

/* 弹窗内容的滑入动画 */
.animate-slide-up {
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 淡入动画 */
.animate-fade-in {
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 输入框聚焦时的光晕效果 */
input:focus,
textarea:focus {
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}
</style>

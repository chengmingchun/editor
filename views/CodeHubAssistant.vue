<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Code2, 
  ExternalLink, 
  X, 
  Download, 
  Trash2, 
  Brain,
  AlertCircle,
  CheckCircle,
  MessageSquare,
  FileCode,
  GitPullRequest,
  RefreshCw,
  Sparkles
} from 'lucide-vue-next';

// ==================== Types ====================

interface ReviewComment {
  id: string;
  author: string;
  content: string;
  file_path?: string;
  line_number?: number;
  severity: 'critical' | 'warning' | 'suggestion';
  created_at: string;
}

interface RAGReviewPair {
  error_logic: string;
  fix_suggestion: string;
  source_comment: ReviewComment;
}

// ==================== State ====================

const mrUrl = ref('');
const isWebviewOpen = ref(false);
const isCapturing = ref(false);
const isProcessingRAG = ref(false);
const reviewComments = ref<ReviewComment[]>([]);
const ragPairs = ref<RAGReviewPair[]>([]);
const activeTab = ref<'comments' | 'rag'>('comments');
const selectedComment = ref<ReviewComment | null>(null);

// ==================== Computed ====================

const severityStats = computed(() => {
  const stats = { critical: 0, warning: 0, suggestion: 0 };
  reviewComments.value.forEach(c => {
    stats[c.severity]++;
  });
  return stats;
});

const severityColor = (severity: string) => {
  switch (severity) {
    case 'critical': return 'text-nord-11 bg-nord-11/10 border-nord-11/30';
    case 'warning': return 'text-nord-12 bg-nord-12/10 border-nord-12/30';
    case 'suggestion': return 'text-nord-14 bg-nord-14/10 border-nord-14/30';
    default: return 'text-nord-3 bg-nord-3/10 border-nord-3/30';
  }
};

// ==================== Actions ====================

const openCodeHubWindow = async () => {
  if (!mrUrl.value.trim()) {
    alert('请输入 MR 链接');
    return;
  }
  
  try {
    await invoke('open_codehub_window', { mr_url: mrUrl.value });
    isWebviewOpen.value = true;
  } catch (err) {
    console.error('打开窗口失败:', err);
    alert('打开 CodeHub 窗口失败: ' + err);
  }
};

const closeCodeHubWindow = async () => {
  try {
    await invoke('close_codehub_window');
    isWebviewOpen.value = false;
  } catch (err) {
    console.error('关闭窗口失败:', err);
  }
};

const captureComments = async () => {
  if (!isWebviewOpen.value) {
    alert('请先打开 CodeHub 窗口');
    return;
  }
  
  isCapturing.value = true;
  try {
    const comments = await invoke('capture_review_comments') as ReviewComment[];
    reviewComments.value = comments;
    alert(`成功抓取 ${comments.length} 条检视意见`);
  } catch (err) {
    console.error('抓取失败:', err);
    alert('抓取检视意见失败，请确保 CodeHub 页面已加载完成');
  } finally {
    isCapturing.value = false;
  }
};

const processRAG = async () => {
  if (reviewComments.value.length === 0) {
    alert('没有检视意见可供处理');
    return;
  }
  
  isProcessingRAG.value = true;
  try {
    const pairs = await invoke('process_rag_pairs') as RAGReviewPair[];
    ragPairs.value = pairs;
    activeTab.value = 'rag';
    alert(`成功生成 ${pairs.length} 个 RAG 训练对`);
  } catch (err) {
    console.error('RAG 处理失败:', err);
    alert('RAG 处理失败: ' + err);
  } finally {
    isProcessingRAG.value = false;
  }
};

const exportRAGData = async () => {
  if (ragPairs.value.length === 0) {
    alert('没有 RAG 数据可供导出');
    return;
  }
  
  try {
    const filePath = await invoke('export_rag_data') as string;
    alert(`RAG 数据已导出至: ${filePath}`);
  } catch (err) {
    console.error('导出失败:', err);
    alert('导出失败: ' + err);
  }
};

const clearComments = async () => {
  if (!confirm('确定要清空所有检视意见吗？')) return;
  
  try {
    await invoke('clear_review_comments');
    reviewComments.value = [];
    ragPairs.value = [];
  } catch (err) {
    console.error('清空失败:', err);
  }
};

const addManualComment = () => {
  const content = prompt('请输入检视意见内容:');
  if (!content) return;
  
  const comment: ReviewComment = {
    id: 'manual_' + Date.now(),
    author: 'User',
    content,
    severity: 'suggestion',
    created_at: new Date().toISOString()
  };
  
  invoke('add_review_comment', { comment })
    .then(() => {
      reviewComments.value.push(comment);
    })
    .catch(err => {
      console.error('添加失败:', err);
    });
};

// ==================== Lifecycle ====================

onMounted(async () => {
  // 尝试加载已有的检视意见
  try {
    const comments = await invoke('get_review_comments') as ReviewComment[];
    reviewComments.value = comments;
  } catch (err) {
    // 忽略错误
  }
});
</script>

<template>
  <div class="flex h-screen bg-nord-0 text-nord-4 font-sans overflow-hidden">
    <!-- 左侧控制面板 -->
    <aside class="w-96 bg-nord-1 border-r border-nord-2 flex flex-col">
      <!-- Header -->
      <div class="p-6 border-b border-nord-2">
        <div class="flex items-center gap-3 mb-1">
          <div class="bg-gradient-to-br from-nord-9 to-nord-10 p-2 rounded-xl">
            <Code2 :size="22" class="text-white" />
          </div>
          <div>
            <h1 class="font-bold text-lg text-nord-6">CodeHub 助手</h1>
            <p class="text-xs text-nord-3">智能检视意见捕获</p>
          </div>
        </div>
      </div>

      <!-- MR URL Input -->
      <div class="p-4 space-y-3">
        <label class="text-xs font-medium text-nord-5 uppercase tracking-wider">MR 链接</label>
        <div class="flex gap-2">
          <input 
            v-model="mrUrl" 
            type="text" 
            placeholder="https://codehub.example.com/mr/123"
            class="flex-1 bg-nord-0 border border-nord-2 rounded-lg px-3 py-2 text-sm
                   focus:outline-none focus:border-nord-8 transition-colors"
          />
        </div>
        <div class="flex gap-2">
          <button 
            @click="openCodeHubWindow" 
            :disabled="isWebviewOpen"
            class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
                   bg-nord-10 hover:bg-nord-9 disabled:opacity-50 disabled:cursor-not-allowed
                   text-white text-sm font-medium transition-all active:scale-95"
          >
            <ExternalLink :size="14" />
            {{ isWebviewOpen ? '窗口已打开' : '打开 CodeHub' }}
          </button>
          <button 
            v-if="isWebviewOpen"
            @click="closeCodeHubWindow"
            class="p-2 rounded-lg bg-nord-11/20 text-nord-11 hover:bg-nord-11/30 transition-colors"
          >
            <X :size="16" />
          </button>
        </div>
      </div>

      <!-- Stats -->
      <div class="px-4 pb-4">
        <div class="glass-card p-4">
          <div class="grid grid-cols-3 gap-4 text-center">
            <div>
              <div class="text-2xl font-bold text-nord-11">{{ severityStats.critical }}</div>
              <div class="text-[10px] text-nord-3 uppercase">严重</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-nord-12">{{ severityStats.warning }}</div>
              <div class="text-[10px] text-nord-3 uppercase">警告</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-nord-14">{{ severityStats.suggestion }}</div>
              <div class="text-[10px] text-nord-3 uppercase">建议</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="px-4 pb-4 space-y-2">
        <button 
          @click="captureComments" 
          :disabled="!isWebviewOpen || isCapturing"
          class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl
                 bg-nord-8 text-nord-0 font-medium text-sm
                 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed
                 transition-all active:scale-95 shadow-lg shadow-nord-8/20"
        >
          <RefreshCw :size="16" :class="isCapturing ? 'animate-spin' : ''" />
          {{ isCapturing ? '抓取中...' : '抓取检视意见' }}
        </button>

        <button 
          @click="processRAG" 
          :disabled="reviewComments.length === 0 || isProcessingRAG"
          class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl
                 bg-gradient-to-r from-nord-15 to-nord-11 text-white font-medium text-sm
                 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed
                 transition-all active:scale-95 shadow-lg shadow-nord-15/20"
        >
          <Brain :size="16" :class="isProcessingRAG ? 'animate-pulse' : ''" />
          {{ isProcessingRAG ? '处理中...' : 'RAG 智能处理' }}
        </button>

        <div class="flex gap-2">
          <button 
            @click="addManualComment"
            class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
                   bg-nord-2 text-nord-4 text-sm hover:bg-nord-3 transition-colors"
          >
            <MessageSquare :size="14" />
            手动添加
          </button>
          <button 
            @click="exportRAGData"
            :disabled="ragPairs.length === 0"
            class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
                   bg-nord-2 text-nord-4 text-sm hover:bg-nord-3 
                   disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Download :size="14" />
            导出 JSON
          </button>
        </div>
      </div>

      <!-- Tips -->
      <div class="flex-1 px-4">
        <div class="bg-nord-10/5 border border-nord-10/20 rounded-xl p-4">
          <div class="flex items-start gap-2">
            <Sparkles :size="16" class="text-nord-10 mt-0.5 flex-shrink-0" />
            <div class="text-xs text-nord-5 leading-relaxed">
              <p class="font-medium mb-1">RAG 预处理说明</p>
              <p class="text-nord-3">系统会自动将检视意见解析为「错误逻辑-修复建议」对，用于 AI 上下文学习。</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="p-4 border-t border-nord-2">
        <button 
          @click="clearComments"
          class="w-full flex items-center justify-center gap-2 py-2 rounded-lg
                 text-nord-11 hover:bg-nord-11/10 transition-colors text-sm"
        >
          <Trash2 :size="14" />
          清空所有数据
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <main class="flex-1 flex flex-col bg-nord-0 overflow-hidden">
      <!-- Tabs -->
      <div class="h-14 border-b border-nord-2 flex items-center px-6 bg-nord-1/50">
        <div class="flex gap-1 p-1 bg-nord-1 rounded-lg">
          <button 
            @click="activeTab = 'comments'"
            :class="['tab-btn', activeTab === 'comments' ? 'active' : '']"
          >
            <MessageSquare :size="14" />
            <span>检视意见 ({{ reviewComments.length }})</span>
          </button>
          <button 
            @click="activeTab = 'rag'"
            :class="['tab-btn', activeTab === 'rag' ? 'active' : '']"
          >
            <Brain :size="14" />
            <span>RAG 训练对 ({{ ragPairs.length }})</span>
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- Comments Tab -->
        <div v-if="activeTab === 'comments'" class="space-y-4">
          <div v-if="reviewComments.length === 0" class="empty-state">
            <GitPullRequest :size="48" class="text-nord-3 mb-4" />
            <p class="text-nord-5 font-medium">暂无检视意见</p>
            <p class="text-sm text-nord-3 mt-1">输入 MR 链接并打开 CodeHub 窗口以开始抓取</p>
          </div>

          <div 
            v-for="comment in reviewComments" 
            :key="comment.id"
            @click="selectedComment = comment"
            :class="['comment-card', selectedComment?.id === comment.id ? 'selected' : '']"
          >
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-full bg-nord-10/20 flex items-center justify-center">
                  <span class="text-sm font-medium text-nord-9">{{ comment.author[0]?.toUpperCase() }}</span>
                </div>
                <div>
                  <div class="text-sm font-medium text-nord-5">{{ comment.author }}</div>
                  <div class="text-xs text-nord-3">{{ new Date(comment.created_at).toLocaleString() }}</div>
                </div>
              </div>
              <span :class="['severity-badge', severityColor(comment.severity)]">
                {{ comment.severity }}
              </span>
            </div>
            <p class="text-sm text-nord-4 leading-relaxed">{{ comment.content }}</p>
            <div v-if="comment.file_path" class="mt-3 flex items-center gap-2 text-xs text-nord-3">
              <FileCode :size="12" />
              <span>{{ comment.file_path }}</span>
              <span v-if="comment.line_number">:{{ comment.line_number }}</span>
            </div>
          </div>
        </div>

        <!-- RAG Tab -->
        <div v-else class="space-y-4">
          <div v-if="ragPairs.length === 0" class="empty-state">
            <Brain :size="48" class="text-nord-3 mb-4" />
            <p class="text-nord-5 font-medium">暂无 RAG 数据</p>
            <p class="text-sm text-nord-3 mt-1">先抓取检视意见，然后点击「RAG 智能处理」</p>
          </div>

          <div 
            v-for="(pair, index) in ragPairs" 
            :key="index"
            class="rag-card"
          >
            <div class="flex items-center gap-2 mb-3">
              <AlertCircle :size="16" class="text-nord-11" />
              <span class="text-sm font-medium text-nord-5">错误逻辑</span>
            </div>
            <p class="text-sm text-nord-4 bg-nord-11/5 border border-nord-11/20 rounded-lg p-3 mb-4">
              {{ pair.error_logic }}
            </p>
            
            <div class="flex items-center gap-2 mb-3">
              <CheckCircle :size="16" class="text-nord-14" />
              <span class="text-sm font-medium text-nord-5">修复建议</span>
            </div>
            <p class="text-sm text-nord-4 bg-nord-14/5 border border-nord-14/20 rounded-lg p-3">
              {{ pair.fix_suggestion }}
            </p>

            <div class="mt-4 pt-3 border-t border-nord-2/50 flex items-center gap-2 text-xs text-nord-3">
              <span>来源: {{ pair.source_comment.author }}</span>
              <span>·</span>
              <span>{{ new Date(pair.source_comment.created_at).toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.glass-card {
  @apply bg-nord-1/40 backdrop-blur-md border border-nord-2/50 rounded-xl;
}

.tab-btn {
  @apply flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium
         text-nord-4 hover:text-nord-5 transition-all;
}

.tab-btn.active {
  @apply bg-nord-2 text-nord-6 shadow-sm;
}

.empty-state {
  @apply flex flex-col items-center justify-center h-96 text-center;
}

.comment-card {
  @apply bg-nord-1/50 border border-nord-2/50 rounded-xl p-4 cursor-pointer
         hover:border-nord-3 hover:bg-nord-1 transition-all;
}

.comment-card.selected {
  @apply border-nord-8 bg-nord-8/5;
}

.severity-badge {
  @apply px-2 py-1 rounded-full text-[10px] font-medium uppercase border;
}

.rag-card {
  @apply bg-nord-1/50 border border-nord-2/50 rounded-xl p-5;
}
</style>

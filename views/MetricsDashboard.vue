<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  BarChart3, 
  Clock, 
  Code2, 
  TrendingUp, 
  CheckCircle,
  RefreshCw,
  Plus,
  Trash2,
  Download,
  Target,
  GitPullRequest,
  X
} from 'lucide-vue-next';

// ==================== Types ====================

interface MetricData {
  date: string;
  ai_generate_time_minutes: number;
  ai_lines_of_code: number;
  manual_lines_of_code: number;
  review_comments_count: number;
  resolved_comments_count: number;
}

interface MetricsSummary {
  total_sessions: number;
  avg_ai_time: number;
  total_ai_lines: number;
  total_manual_lines: number;
  ai_efficiency: number;
  total_reviews: number;
  resolved_rate: number;
}

// ==================== State ====================

const metrics = ref<MetricData[]>([]);
const summary = ref<MetricsSummary | null>(null);
const isLoading = ref(false);
const showAddModal = ref(false);

// 新增指标表单
const newMetric = ref<Partial<MetricData>>({
  date: new Date().toISOString().split('T')[0],
  ai_generate_time_minutes: 0,
  ai_lines_of_code: 0,
  manual_lines_of_code: 0,
  review_comments_count: 0,
  resolved_comments_count: 0
});

// ==================== Computed ====================

const chartData = computed(() => {
  // 按日期排序
  const sorted = [...metrics.value].sort((a, b) => 
    new Date(a.date).getTime() - new Date(b.date).getTime()
  );
  
  // 最近 7 条数据
  return sorted.slice(-7);
});

const maxLines = computed(() => {
  if (chartData.value.length === 0) return 100;
  const max = Math.max(
    ...chartData.value.map(m => Math.max(m.ai_lines_of_code, m.manual_lines_of_code))
  );
  return Math.ceil(max / 100) * 100;
});

const maxTime = computed(() => {
  if (chartData.value.length === 0) return 60;
  const max = Math.max(...chartData.value.map(m => m.ai_generate_time_minutes));
  return Math.ceil(max / 10) * 10;
});

const maxReviewComments = computed(() => {
  if (chartData.value.length === 0) return 1;
  return Math.max(...chartData.value.map(m => m.review_comments_count)) || 1;
});

const safeSummary = computed<MetricsSummary>(() => {
  return summary.value ?? {
    total_sessions: 0,
    avg_ai_time: 0,
    total_ai_lines: 0,
    total_manual_lines: 0,
    ai_efficiency: 0,
    total_reviews: 0,
    resolved_rate: 0
  };
});

// ==================== Actions ====================

const loadMetrics = async () => {
  isLoading.value = true;
  try {
    const data = await invoke('get_metrics') as MetricData[];
    metrics.value = data;
    
    const sum = await invoke('get_metrics_summary') as MetricsSummary;
    summary.value = sum;
  } catch (err) {
    console.error('加载数据失败:', err);
  } finally {
    isLoading.value = false;
  }
};

const addMetric = async () => {
  if (!newMetric.value.date) return;
  
  const metric: MetricData = {
    date: newMetric.value.date,
    ai_generate_time_minutes: newMetric.value.ai_generate_time_minutes || 0,
    ai_lines_of_code: newMetric.value.ai_lines_of_code || 0,
    manual_lines_of_code: newMetric.value.manual_lines_of_code || 0,
    review_comments_count: newMetric.value.review_comments_count || 0,
    resolved_comments_count: newMetric.value.resolved_comments_count || 0
  };
  
  try {
    await invoke('add_metric', { metric });
    showAddModal.value = false;
    await loadMetrics();
    
    // 重置表单
    newMetric.value = {
      date: new Date().toISOString().split('T')[0],
      ai_generate_time_minutes: 0,
      ai_lines_of_code: 0,
      manual_lines_of_code: 0,
      review_comments_count: 0,
      resolved_comments_count: 0
    };
  } catch (err) {
    console.error('添加失败:', err);
    alert('添加失败: ' + err);
  }
};

const clearMetrics = async () => {
  if (!confirm('确定要清空所有效能数据吗？')) return;
  
  try {
    await invoke('clear_metrics');
    await loadMetrics();
  } catch (err) {
    console.error('清空失败:', err);
  }
};

const exportData = () => {
  const dataStr = JSON.stringify(metrics.value, null, 2);
  const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
  
  const exportFileDefaultName = `metrics_${new Date().toISOString().split('T')[0]}.json`;
  
  const linkElement = document.createElement('a');
  linkElement.setAttribute('href', dataUri);
  linkElement.setAttribute('download', exportFileDefaultName);
  linkElement.click();
};

// 计算柱状图高度
const getBarHeight = (value: number, max: number) => {
  if (max === 0) return '0%';
  return `${(value / max) * 100}%`;
};

// ==================== Lifecycle ====================

onMounted(() => {
  loadMetrics();
});
</script>

<template>
  <div class="flex flex-col h-screen bg-nord-0 text-nord-4 font-sans overflow-hidden">
    <!-- Header -->
    <header class="h-16 border-b border-nord-2 flex items-center justify-between px-8 bg-nord-1/50">
      <div class="flex items-center gap-3">
        <div class="bg-gradient-to-br from-nord-14 to-nord-7 p-2 rounded-xl">
          <BarChart3 :size="22" class="text-white" />
        </div>
        <div>
          <h1 class="font-bold text-lg text-nord-6">效能数字化看板</h1>
          <p class="text-xs text-nord-3">AI 生成效能追踪与分析</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <button 
          @click="exportData"
          :disabled="metrics.length === 0"
          class="flex items-center gap-2 px-4 py-2 rounded-lg bg-nord-2 text-nord-4
                 hover:bg-nord-3 disabled:opacity-50 transition-colors text-sm"
        >
          <Download :size="16" />
          导出数据
        </button>
        <button 
          @click="showAddModal = true"
          class="flex items-center gap-2 px-4 py-2 rounded-lg bg-nord-8 text-nord-0
                 hover:opacity-90 transition-colors text-sm font-medium"
        >
          <Plus :size="16" />
          添加记录
        </button>
        <button 
          @click="loadMetrics"
          :disabled="isLoading"
          class="p-2 rounded-lg bg-nord-2 text-nord-4 hover:bg-nord-3 transition-colors"
        >
          <RefreshCw :size="16" :class="isLoading ? 'animate-spin' : ''" />
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto p-8">
      <!-- Summary Cards -->
      <div class="grid grid-cols-4 gap-6 mb-8">
        <!-- AI 效率 -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="p-2 bg-nord-8/20 rounded-lg">
              <TrendingUp :size="20" class="text-nord-8" />
            </div>
            <span class="text-xs text-nord-14 font-medium flex items-center gap-1">
              <TrendingUp :size="12" />
              {{ safeSummary.ai_efficiency.toFixed(1) }}%
            </span>
          </div>
          <div class="text-3xl font-bold text-nord-6 mb-1">
            {{ safeSummary.total_ai_lines.toLocaleString() }}
          </div>
          <div class="text-sm text-nord-3">AI 生成代码行数</div>
        </div>

        <!-- 平均生成时间 -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="p-2 bg-nord-9/20 rounded-lg">
              <Clock :size="20" class="text-nord-9" />
            </div>
            <span class="text-xs text-nord-3">分钟</span>
          </div>
          <div class="text-3xl font-bold text-nord-6 mb-1">
            {{ safeSummary.avg_ai_time.toFixed(1) }}
          </div>
          <div class="text-sm text-nord-3">平均生成时间</div>
        </div>

        <!-- 检视意见 -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="p-2 bg-nord-15/20 rounded-lg">
              <GitPullRequest :size="20" class="text-nord-15" />
            </div>
            <span class="text-xs text-nord-14 font-medium">
              {{ safeSummary.total_reviews > 0 
                ? (safeSummary.resolved_rate).toFixed(1) 
                : '0' }}% 解决率
            </span>
          </div>
          <div class="text-3xl font-bold text-nord-6 mb-1">
            {{ safeSummary.total_reviews.toLocaleString() }}
          </div>
          <div class="text-sm text-nord-3">检视意见总数</div>
        </div>

        <!-- 会话数 -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="p-2 bg-nord-14/20 rounded-lg">
              <Target :size="20" class="text-nord-14" />
            </div>
            <span class="text-xs text-nord-3">累计</span>
          </div>
          <div class="text-3xl font-bold text-nord-6 mb-1">
            {{ safeSummary.total_sessions }}
          </div>
          <div class="text-sm text-nord-3">生成会话数</div>
        </div>
      </div>

      <!-- Charts -->
      <div class="grid grid-cols-2 gap-6 mb-8">
        <!-- Code Lines Chart -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Code2 :size="20" class="text-nord-8" />
              <h3 class="font-semibold text-nord-5">代码生成对比</h3>
            </div>
            <div class="flex items-center gap-4 text-xs">
              <div class="flex items-center gap-2">
                <div class="w-3 h-3 rounded bg-nord-8"></div>
                <span class="text-nord-4">AI 生成</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-3 h-3 rounded bg-nord-3"></div>
                <span class="text-nord-4">人工编写</span>
              </div>
            </div>
          </div>
          
          <div v-if="chartData.length === 0" class="h-64 flex items-center justify-center text-nord-3">
            暂无数据
          </div>
          <div v-else class="h-64 flex items-end justify-between gap-2 pt-8">
            <div 
              v-for="(item, index) in chartData" 
              :key="index"
              class="flex-1 flex flex-col items-center gap-2 group"
            >
              <div class="w-full flex gap-1 h-48 items-end">
                <div 
                  class="flex-1 bg-nord-8 rounded-t transition-all duration-500 hover:opacity-80 relative group"
                  :style="{ height: getBarHeight(item.ai_lines_of_code, maxLines) }"
                >
                  <div class="opacity-0 group-hover:opacity-100 absolute -top-8 left-1/2 -translate-x-1/2 
                              bg-nord-1 text-nord-5 text-xs px-2 py-1 rounded whitespace-nowrap transition-opacity">
                    {{ item.ai_lines_of_code }}
                  </div>
                </div>
                <div 
                  class="flex-1 bg-nord-3 rounded-t transition-all duration-500 hover:opacity-80 relative group"
                  :style="{ height: getBarHeight(item.manual_lines_of_code, maxLines) }"
                >
                  <div class="opacity-0 group-hover:opacity-100 absolute -top-8 left-1/2 -translate-x-1/2 
                              bg-nord-1 text-nord-5 text-xs px-2 py-1 rounded whitespace-nowrap transition-opacity">
                    {{ item.manual_lines_of_code }}
                  </div>
                </div>
              </div>
              <span class="text-xs text-nord-3">{{ item.date.slice(5) }}</span>
            </div>
          </div>
        </div>

        <!-- Time Chart -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Clock :size="20" class="text-nord-9" />
              <h3 class="font-semibold text-nord-5">AI 生成时间趋势</h3>
            </div>
            <span class="text-xs text-nord-3">分钟</span>
          </div>
          
          <div v-if="chartData.length === 0" class="h-64 flex items-center justify-center text-nord-3">
            暂无数据
          </div>
          <div v-else class="h-64 relative pt-8">
            <!-- Grid lines -->
            <div class="absolute inset-0 flex flex-col justify-between pointer-events-none">
              <div class="border-b border-nord-2/30 h-0"></div>
              <div class="border-b border-nord-2/30 h-0"></div>
              <div class="border-b border-nord-2/30 h-0"></div>
              <div class="border-b border-nord-2/30 h-0"></div>
              <div class="border-b border-nord-2 h-0"></div>
            </div>
            
            <!-- Line chart -->
            <svg class="w-full h-full" viewBox="0 0 100 100" preserveAspectRatio="none">
              <defs>
                <linearGradient id="timeGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" style="stop-color:#88C0D0;stop-opacity:0.3" />
                  <stop offset="100%" style="stop-color:#88C0D0;stop-opacity:0" />
                </linearGradient>
              </defs>
              
              <!-- Area -->
              <polygon 
                :points="chartData.map((item, i) => {
                  const x = (i / (chartData.length - 1 || 1)) * 100;
                  const y = 100 - (item.ai_generate_time_minutes / maxTime) * 100;
                  return `${x},${y}`;
                }).join(' ') + ` 100,100 0,100`"
                fill="url(#timeGradient)"
              />
              
              <!-- Line -->
              <polyline 
                :points="chartData.map((item, i) => {
                  const x = (i / (chartData.length - 1 || 1)) * 100;
                  const y = 100 - (item.ai_generate_time_minutes / maxTime) * 100;
                  return `${x},${y}`;
                }).join(' ')"
                fill="none"
                stroke="#88C0D0"
                stroke-width="2"
                vector-effect="non-scaling-stroke"
              />
              
              <!-- Points -->
              <circle 
                v-for="(item, i) in chartData" 
                :key="i"
                :cx="(i / (chartData.length - 1 || 1)) * 100 + '%'"
                :cy="(100 - (item.ai_generate_time_minutes / maxTime) * 100) + '%'"
                r="4"
                fill="#88C0D0"
                class="hover:r-6 transition-all cursor-pointer"
              >
                <title>{{ item.date }}: {{ item.ai_generate_time_minutes }}分钟</title>
              </circle>
            </svg>
          </div>
          
          <!-- X-axis labels -->
          <div v-if="chartData.length > 0" class="flex justify-between mt-2 px-2">
            <span 
              v-for="(item, index) in chartData" 
              :key="index"
              class="text-xs text-nord-3"
            >
              {{ item.date.slice(5) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Review Trend Chart -->
      <div class="glass-card p-6 mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-3">
            <CheckCircle :size="20" class="text-nord-14" />
            <h3 class="font-semibold text-nord-5">检视意见趋势</h3>
          </div>
          <div class="flex items-center gap-4 text-xs">
            <div class="flex items-center gap-2">
              <div class="w-3 h-3 rounded bg-nord-11"></div>
              <span class="text-nord-4">新增意见</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-3 h-3 rounded bg-nord-14"></div>
              <span class="text-nord-4">已解决</span>
            </div>
          </div>
        </div>
        
        <div v-if="chartData.length === 0" class="h-48 flex items-center justify-center text-nord-3">
          暂无数据
        </div>
        <div v-else class="h-48 flex items-end justify-between gap-4">
          <div 
            v-for="(item, index) in chartData" 
            :key="index"
            class="flex-1 flex flex-col items-center gap-2"
          >
            <div class="w-full flex flex-col gap-1 h-40 justify-end">
              <div 
                class="w-full bg-nord-11 rounded-t transition-all duration-500 relative group"
                :style="{ height: getBarHeight(item.review_comments_count, maxReviewComments) }"
              >
                <div class="opacity-0 group-hover:opacity-100 absolute -top-6 left-1/2 -translate-x-1/2 
                            bg-nord-1 text-nord-5 text-xs px-2 py-0.5 rounded whitespace-nowrap transition-opacity">
                  {{ item.review_comments_count }}
                </div>
              </div>
              <div 
                class="w-full bg-nord-14 rounded-b transition-all duration-500 relative group"
                :style="{ height: getBarHeight(item.resolved_comments_count, maxReviewComments) }"
              >
                <div class="opacity-0 group-hover:opacity-100 absolute -top-6 left-1/2 -translate-x-1/2 
                            bg-nord-1 text-nord-5 text-xs px-2 py-0.5 rounded whitespace-nowrap transition-opacity">
                  {{ item.resolved_comments_count }}
                </div>
              </div>
            </div>
            <span class="text-xs text-nord-3">{{ item.date.slice(5) }}</span>
          </div>
        </div>
      </div>

      <!-- Data Table -->
      <div class="glass-card overflow-hidden">
        <div class="px-6 py-4 border-b border-nord-2 flex items-center justify-between">
          <h3 class="font-semibold text-nord-5">详细记录</h3>
          <button 
            @click="clearMetrics"
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-nord-11 
                   hover:bg-nord-11/10 transition-colors text-sm"
          >
            <Trash2 :size="14" />
            清空数据
          </button>
        </div>
        
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="bg-nord-1/50">
                <th class="text-left px-6 py-3 text-xs font-medium text-nord-3 uppercase">日期</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">AI 生成时间</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">AI 代码行数</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">人工代码行数</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">检视意见</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">已解决</th>
                <th class="text-right px-6 py-3 text-xs font-medium text-nord-3 uppercase">解决率</th>
              </tr>
            </thead>
            <tbody>
              <tr 
                v-for="(item, index) in [...metrics].reverse()" 
                :key="index"
                class="border-b border-nord-2/50 hover:bg-nord-1/30 transition-colors"
              >
                <td class="px-6 py-3 text-sm text-nord-5">{{ item.date }}</td>
                <td class="px-6 py-3 text-sm text-nord-4 text-right">{{ item.ai_generate_time_minutes }} min</td>
                <td class="px-6 py-3 text-sm text-nord-8 text-right">{{ item.ai_lines_of_code.toLocaleString() }}</td>
                <td class="px-6 py-3 text-sm text-nord-4 text-right">{{ item.manual_lines_of_code.toLocaleString() }}</td>
                <td class="px-6 py-3 text-sm text-nord-11 text-right">{{ item.review_comments_count }}</td>
                <td class="px-6 py-3 text-sm text-nord-14 text-right">{{ item.resolved_comments_count }}</td>
                <td class="px-6 py-3 text-right">
                  <span 
                    :class="[
                      'text-xs font-medium px-2 py-1 rounded-full',
                      item.review_comments_count > 0 && item.resolved_comments_count / item.review_comments_count >= 0.8
                        ? 'bg-nord-14/10 text-nord-14'
                        : 'bg-nord-12/10 text-nord-12'
                    ]"
                  >
                    {{ item.review_comments_count > 0 
                      ? ((item.resolved_comments_count / item.review_comments_count) * 100).toFixed(0) 
                      : 0 }}%
                  </span>
                </td>
              </tr>
              <tr v-if="metrics.length === 0">
                <td colspan="7" class="px-6 py-12 text-center text-nord-3">
                  暂无记录，点击右上角「添加记录」开始追踪
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </main>

    <!-- Add Modal -->
    <div v-if="showAddModal" class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div class="glass-card w-full max-w-lg mx-4 p-6 animate-fade-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="font-bold text-lg text-nord-6">添加效能记录</h3>
          <button @click="showAddModal = false" class="p-2 hover:bg-nord-2 rounded-lg transition-colors">
            <X :size="18" class="text-nord-3" />
          </button>
        </div>
        
        <div class="space-y-4">
          <div>
            <label class="text-xs text-nord-3 uppercase font-medium block mb-2">日期</label>
            <input 
              v-model="newMetric.date"
              type="date"
              class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                     focus:outline-none focus:border-nord-8 transition-colors"
            />
          </div>
          
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-xs text-nord-3 uppercase font-medium block mb-2">AI 生成时间 (分钟)</label>
              <input 
                v-model.number="newMetric.ai_generate_time_minutes"
                type="number"
                min="0"
                step="0.1"
                class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                       focus:outline-none focus:border-nord-8 transition-colors"
              />
            </div>
            <div>
              <label class="text-xs text-nord-3 uppercase font-medium block mb-2">AI 代码行数</label>
              <input 
                v-model.number="newMetric.ai_lines_of_code"
                type="number"
                min="0"
                class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                       focus:outline-none focus:border-nord-8 transition-colors"
              />
            </div>
          </div>
          
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-xs text-nord-3 uppercase font-medium block mb-2">人工代码行数</label>
              <input 
                v-model.number="newMetric.manual_lines_of_code"
                type="number"
                min="0"
                class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                       focus:outline-none focus:border-nord-8 transition-colors"
              />
            </div>
            <div>
              <label class="text-xs text-nord-3 uppercase font-medium block mb-2">检视意见数</label>
              <input 
                v-model.number="newMetric.review_comments_count"
                type="number"
                min="0"
                class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                       focus:outline-none focus:border-nord-8 transition-colors"
              />
            </div>
          </div>
          
          <div>
            <label class="text-xs text-nord-3 uppercase font-medium block mb-2">已解决意见数</label>
            <input 
              v-model.number="newMetric.resolved_comments_count"
              type="number"
              min="0"
              class="w-full bg-nord-0 border border-nord-2 rounded-lg px-4 py-2.5 text-sm
                     focus:outline-none focus:border-nord-8 transition-colors"
            />
          </div>
        </div>
        
        <div class="flex gap-3 mt-6">
          <button 
            @click="showAddModal = false"
            class="flex-1 py-2.5 rounded-lg bg-nord-2 text-nord-4 hover:bg-nord-3 transition-colors text-sm font-medium"
          >
            取消
          </button>
          <button 
            @click="addMetric"
            class="flex-1 py-2.5 rounded-lg bg-nord-8 text-nord-0 hover:opacity-90 transition-colors text-sm font-medium"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-card {
  @apply bg-nord-1/40 backdrop-blur-md border border-nord-2/50 rounded-2xl;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.animate-fade-in {
  animation: fadeIn 0.2s ease-out;
}
</style>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  PenTool, 
  Code2, 
  BarChart3, 
  Settings,
  Sparkles
} from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();

const activeRoute = ref(route.path);

watch(() => route.path, (newPath) => {
  activeRoute.value = newPath;
});

const navigate = (path: string) => {
  router.push(path);
};

const isTauri = ref(false);
onMounted(() => {
  isTauri.value = !!(window as any).__TAURI__;
});
</script>

<template>
  <div class="flex h-screen bg-gray-50 text-gray-800 font-sans overflow-hidden">
    <!-- 全局侧边导航栏 -->
    <aside class="w-16 bg-white border-r border-gray-200 flex flex-col items-center py-4 shadow-sm z-50">
      <!-- Logo -->
      <div class="mb-6">
        <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl 
                    flex items-center justify-center shadow-lg">
          <Sparkles :size="20" class="text-white" />
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 flex flex-col gap-1 w-full px-2">
        <button 
          @click="navigate('/design')"
          :class="[
            'nav-item',
            activeRoute === '/design' || activeRoute === '/' ? 'active' : ''
          ]"
          title="设计工作室"
        >
          <PenTool :size="20" />
        </button>

        <button 
          @click="navigate('/codehub')"
          :class="['nav-item', activeRoute === '/codehub' ? 'active' : '']"
          title="代码检视"
        >
          <Code2 :size="20" />
        </button>

        <button 
          @click="navigate('/metrics')"
          :class="['nav-item', activeRoute === '/metrics' ? 'active' : '']"
          title="效能看板"
        >
          <BarChart3 :size="20" />
        </button>
      </nav>

      <!-- Footer -->
      <div class="mt-auto">
        <button class="nav-item" title="设置">
          <Settings :size="18" />
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-hidden relative bg-gray-50">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </transition>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.nav-item {
  @apply w-full flex items-center justify-center p-3 rounded-xl
         text-gray-500 hover:text-gray-700 hover:bg-gray-100 
         transition-all duration-200;
}

.nav-item.active {
  @apply bg-blue-50 text-blue-600;
}

/* Page transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateX(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  PenTool, 
  Settings,
  Sparkles
} from 'lucide-vue-next';
import ConfigModal from './components/ConfigModal.vue';

const route = useRoute();
const router = useRouter();

const activeRoute = ref(route.path);

watch(() => route.path, (newPath) => {
  activeRoute.value = newPath;
});

const navigate = (path: string) => {
  router.push(path);
};

const showConfigModal = ref(false);

const isTauri = ref(false);
onMounted(() => {
  isTauri.value = !!(window as any).__TAURI__;
});
</script>

<template>
  <div class="flex h-screen bg-gray-50 text-gray-800 font-sans overflow-hidden">
    <aside class="w-16 bg-white border-r border-gray-200 flex flex-col items-center py-4 shadow-sm z-50">
      <div class="mb-6">
        <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl 
                    flex items-center justify-center shadow-lg">
          <Sparkles :size="20" class="text-white" />
        </div>
      </div>

      <nav class="flex-1 flex flex-col gap-1 w-full px-2">
        <button 
          @click="navigate('/editor')"
          :class="[
            'nav-item',
            activeRoute === '/editor' || activeRoute === '/' ? 'active' : ''
          ]"
          title="编辑器"
        >
          <PenTool :size="20" />
        </button>
      </nav>

      <div class="mt-auto">
        <button @click="showConfigModal = true" class="nav-item" title="设置">
          <Settings :size="18" />
        </button>
      </div>
    </aside>

    <main class="flex-1 overflow-hidden relative bg-gray-50">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </transition>
      </router-view>
    </main>

    <ConfigModal 
      :show="showConfigModal" 
      @close="showConfigModal = false" 
    />
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

<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { applyTheme } from './domain/theme'
import { useProgressStore } from './stores/progress'
import { useToastStore } from './stores/toast'

const TOAST_MS = 4000

const toast = useToastStore()
const progress = useProgressStore()

watch(
  () => toast.messages.length,
  () => {
    const latest = toast.messages[toast.messages.length - 1]
    if (latest) setTimeout(() => toast.dismiss(latest), TOAST_MS)
  },
)

// 偏好為 system 時跟著系統深淺色切換
const darkQuery = window.matchMedia?.('(prefers-color-scheme: dark)')
const onSystemTheme = () => {
  if (progress.prefs.theme === 'system') applyTheme('system')
}
onMounted(() => darkQuery?.addEventListener?.('change', onSystemTheme))
onBeforeUnmount(() => darkQuery?.removeEventListener?.('change', onSystemTheme))
</script>

<template>
  <RouterView />
  <div class="toasts" aria-live="polite">
    <p v-for="m in toast.messages" :key="m" class="toast" role="status">{{ m }}</p>
  </div>
</template>

<style scoped>
.toasts {
  position: fixed;
  left: 50%;
  bottom: 24px;
  transform: translateX(-50%);
  z-index: 60;
  display: grid;
  gap: 8px;
  pointer-events: none;
}
.toast {
  margin: 0;
  padding: 10px 18px;
  border-radius: var(--radius-pill);
  background: var(--text);
  color: var(--bg);
  box-shadow: var(--shadow);
  font-weight: 700;
}
</style>

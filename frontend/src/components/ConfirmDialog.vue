<script setup lang="ts">
import { ref } from 'vue'
import { useModalFocus } from '../composables/useModalFocus'

// 海洋繪本風格確認框（看提示、重設）
defineProps<{
  message: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const root = ref<HTMLElement | null>(null)
const { trapTab } = useModalFocus(root)

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('cancel')
    return
  }
  trapTab(event)
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('cancel')">
    <div
      ref="root"
      class="modal confirm-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-message"
      data-testid="confirm-dialog"
      tabindex="-1"
      @keydown="onKeydown"
    >
      <p id="confirm-message" class="confirm-message" data-testid="confirm-message">{{ message }}</p>
      <div class="modal-actions">
        <button type="button" class="btn is-secondary" data-testid="confirm-cancel" @click="emit('cancel')">取消</button>
        <button type="button" class="btn is-primary" data-testid="confirm-ok" @click="emit('confirm')">確定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.confirm-dialog {
  max-width: 420px;
}
.confirm-message {
  margin: 0 0 20px;
  line-height: 1.7;
}
</style>

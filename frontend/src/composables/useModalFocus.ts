import { nextTick, onBeforeUnmount, onMounted } from 'vue'
import type { Ref } from 'vue'

const FOCUSABLE = 'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

// Modal 焦點：開啟時移入、Tab 循環、關閉後回到觸發元素
export function useModalFocus(root: Ref<HTMLElement | null>) {
  let previous: HTMLElement | null = null

  const focusables = () => Array.from(root.value?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? [])

  onMounted(async () => {
    previous = document.activeElement instanceof HTMLElement ? document.activeElement : null
    await nextTick()
    const first = focusables()[0]
    ;(first ?? root.value)?.focus()
  })

  onBeforeUnmount(() => {
    if (previous && document.contains(previous)) previous.focus()
  })

  function trapTab(event: KeyboardEvent) {
    if (event.key !== 'Tab') return
    const items = focusables()
    if (items.length === 0) return
    const first = items[0]
    const last = items[items.length - 1]
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault()
      first.focus()
    }
  }

  return { trapTab }
}

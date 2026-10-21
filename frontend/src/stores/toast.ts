import { defineStore } from 'pinia'

export const useToastStore = defineStore('toast', {
  state: () => ({
    messages: [] as string[],
  }),
  actions: {
    show(message: string) {
      this.messages.push(message)
    },
    dismiss(message: string) {
      this.messages = this.messages.filter((m) => m !== message)
    },
  },
})

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    // 只取 Monaco 核心 API；編輯器功能與 rust 語言在 CodeEditor 另外註冊
    alias: [{ find: /^monaco-editor$/, replacement: 'monaco-editor/editor/editor.api' }],
  },
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3000',
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/__tests__/**/*.spec.ts'],
    restoreMocks: true,
  },
})

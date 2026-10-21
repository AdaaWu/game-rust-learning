import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createWebHistory } from 'vue-router'
import EditorWorker from 'monaco-editor/editor/editor.worker?worker'
import './style.css'
import App from './App.vue'
import { createAppRouter } from './router'

// Monaco 只需要基本 editor worker
;(self as unknown as { MonacoEnvironment: { getWorker: () => Worker } }).MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
}

const app = createApp(App)
app.use(createPinia())
app.use(createAppRouter(createWebHistory()))
app.mount('#app')

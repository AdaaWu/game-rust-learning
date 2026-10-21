<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as monaco from 'monaco-editor'
import 'monaco-editor/features/register.all'
import 'monaco-editor/languages/definitions/rust/register'
import type { Diagnostic } from '../api/types'
import { markerHover, panicMarker, toMarkers } from '../domain/editor/markers'
import type { EditorMarker } from '../domain/editor/markers'

const props = defineProps<{
  modelValue: string
  // 編輯器內以 Monaco model 轉成 marker
  diagnostics: Diagnostic[]
  // Run panic 的 stderr；含 main.rs:行:欄 時整行畫 marker
  panicStderr: string | null
  // MemoryView 同步的高亮行；null 表示移除
  highlightLine: number | null
  theme: 'light' | 'dark'
}>()

const emit = defineEmits<{
  'update:modelValue': [code: string]
  run: []
  submit: []
  'gutter-click': [line: number]
}>()

const MARKER_OWNER = 'rustc'
const FONT = "'JetBrains Mono', ui-monospace, monospace"

// 只覆寫背景、行高亮與字型，對齊海洋繪本 token
const THEMES = {
  'ocean-light': { base: 'vs', bg: '#fffdf8', line: '#fff1dc', gutter: '#c9ae93' },
  'ocean-dark': { base: 'vs-dark', bg: '#16384b', line: '#1d465c', gutter: '#7fa3b3' },
} as const

let themesDefined = false
function defineThemes() {
  if (themesDefined) return
  themesDefined = true
  for (const [name, t] of Object.entries(THEMES)) {
    monaco.editor.defineTheme(name, {
      base: t.base,
      inherit: true,
      rules: [],
      colors: {
        'editor.background': t.bg,
        'editorGutter.background': t.bg,
        'editor.lineHighlightBackground': t.line,
        'editorLineNumber.foreground': t.gutter,
      },
    })
  }
}

const host = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let decorationIds: string[] = []
let flashIds: string[] = []
let applyingExternal = false
let currentMarkers: EditorMarker[] = []
let hoverProvider: monaco.IDisposable | null = null

function toMonacoMarker(m: EditorMarker): monaco.editor.IMarkerData {
  const code = m.code === null ? undefined : m.explanationUrl ? { value: m.code, target: monaco.Uri.parse(m.explanationUrl) } : m.code
  return {
    startLineNumber: m.startLineNumber,
    startColumn: m.startColumn,
    endLineNumber: m.endLineNumber,
    endColumn: m.endColumn,
    severity: m.severity as monaco.MarkerSeverity,
    message: m.message,
    code,
  }
}

function setMarkers(markers: EditorMarker[]) {
  const model = editor?.getModel()
  if (!model) return
  currentMarkers = markers
  monaco.editor.setModelMarkers(model, MARKER_OWNER, markers.map(toMonacoMarker))
}

function refreshMarkers() {
  const model = editor?.getModel()
  if (!model) return
  const markers = toMarkers(props.diagnostics, model)
  if (props.panicStderr) {
    const panic = panicMarker(props.panicStderr, model)
    if (panic) markers.push(panic)
  }
  setMarkers(markers)
}

function applyHighlight(line: number | null) {
  if (!editor) return
  const next =
    line === null
      ? []
      : [
          {
            range: new monaco.Range(line, 1, line, 1),
            options: { isWholeLine: true, className: 'rq-line-active', glyphMarginClassName: 'rq-line-glyph' },
          },
        ]
  decorationIds = editor.deltaDecorations(decorationIds, next)
  if (line !== null) editor.revealLineInCenterIfOutsideViewport(line)
}

onMounted(() => {
  if (!host.value) return
  // e2e 讀取 Monaco marker、游標與裝飾用，只在開發模式掛上
  if (import.meta.env.DEV) (window as unknown as { __rqMonaco?: typeof monaco }).__rqMonaco = monaco
  defineThemes()
  const mobile = typeof window.matchMedia === 'function' && window.matchMedia('(max-width: 767px)').matches
  editor = monaco.editor.create(host.value, {
    value: props.modelValue,
    language: 'rust',
    theme: props.theme === 'dark' ? 'ocean-dark' : 'ocean-light',
    automaticLayout: true,
    minimap: { enabled: !mobile },
    fontSize: 14,
    fontFamily: FONT,
    lineNumbersMinChars: mobile ? 2 : 4,
    glyphMargin: true,
    scrollBeyondLastLine: false,
    tabSize: 4,
    fixedOverflowWidgets: true,
  })
  monaco.editor.setTheme(props.theme === 'dark' ? 'ocean-dark' : 'ocean-light')

  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => emit('run'))
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter, () => emit('submit'))

  editor.onDidChangeModelContent(() => {
    if (applyingExternal || !editor) return
    // 使用者再次編輯時清空 markers
    setMarkers([])
    emit('update:modelValue', editor.getValue())
  })

  editor.onMouseDown((event) => {
    const target = event.target
    if (target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN && target.position) {
      emit('gutter-click', target.position.lineNumber)
    }
  })

  // marker 的 hover 改由這裡提供 markdown（Monaco 內建 marker hover 只顯示純文字）
  hoverProvider = monaco.languages.registerHoverProvider('rust', {
    provideHover(model, position) {
      if (model !== editor?.getModel()) return null
      const hit = currentMarkers.filter(
        (m) =>
          (position.lineNumber > m.startLineNumber || (position.lineNumber === m.startLineNumber && position.column >= m.startColumn)) &&
          (position.lineNumber < m.endLineNumber || (position.lineNumber === m.endLineNumber && position.column <= m.endColumn)),
      )
      if (hit.length === 0) return null
      return { contents: hit.flatMap(markerHover).map((value) => ({ value })) }
    },
  })

  refreshMarkers()
  applyHighlight(props.highlightLine)
})

watch(
  () => props.modelValue,
  (value) => {
    if (!editor || editor.getValue() === value) return
    applyingExternal = true
    editor.setValue(value)
    applyingExternal = false
  },
)

watch(() => [props.diagnostics, props.panicStderr], refreshMarkers)
watch(() => props.highlightLine, applyHighlight)
watch(
  () => props.theme,
  (theme) => monaco.editor.setTheme(theme === 'dark' ? 'ocean-dark' : 'ocean-light'),
)

onBeforeUnmount(() => {
  hoverProvider?.dispose()
  hoverProvider = null
  editor?.dispose()
  editor = null
})

// 點錯誤卡片：捲到該行、游標移到該欄、該行閃一下
function revealPosition(line: number, column: number) {
  if (!editor) return
  editor.revealLineInCenter(line)
  editor.setPosition({ lineNumber: line, column })
  editor.focus()
  flashIds = editor.deltaDecorations(flashIds, [
    { range: new monaco.Range(line, 1, line, 1), options: { isWholeLine: true, className: 'rq-line-flash' } },
  ])
  setTimeout(() => {
    if (editor) flashIds = editor.deltaDecorations(flashIds, [])
  }, 900)
}

defineExpose({ revealPosition })
</script>

<template>
  <div ref="host" class="code-editor" />
</template>

<style scoped>
.code-editor {
  width: 100%;
  height: 100%;
  min-height: 0;
}
/* 內建 marker hover 只有純文字，改由 hover provider 顯示 */
.code-editor :deep(.monaco-hover .marker.hover-contents) {
  display: none;
}
.code-editor :deep(.rq-line-active) {
  background: var(--line-active);
}
.code-editor :deep(.rq-line-glyph)::before {
  content: '▶';
  color: var(--info);
  font-size: 11px;
  padding-left: 4px;
}
.code-editor :deep(.rq-line-flash) {
  background: color-mix(in srgb, var(--warning) 30%, transparent);
  animation: line-flash 900ms ease-out forwards;
}
@keyframes line-flash {
  to {
    background: transparent;
  }
}
@media (prefers-reduced-motion: reduce) {
  .code-editor :deep(.rq-line-flash) {
    animation: none;
  }
}
</style>

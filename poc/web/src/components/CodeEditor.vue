<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { FONTS, FONT_SIZE, monaco, themeName } from '../lib/monaco'
import type { Diagnostic } from '../lib/api'
import { humanize } from '../lib/errorHints'

const props = defineProps<{ modelValue: string; look: 'A' | 'B' | 'C'; dark: boolean; diagnostics: Diagnostic[] }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void; (e: 'run'): void; (e: 'submit'): void }>()

const host = ref<HTMLDivElement>()
let editor: monaco.editor.IStandaloneCodeEditor | undefined
let decorations: monaco.editor.IEditorDecorationsCollection | undefined

onMounted(() => {
  editor = monaco.editor.create(host.value!, {
    value: props.modelValue,
    language: 'rust',
    theme: themeName(props.look, props.dark),
    fontFamily: FONTS[props.look],
    fontSize: FONT_SIZE[props.look],
    fontLigatures: props.look === 'C',
    minimap: { enabled: false },
    automaticLayout: true,
    scrollBeyondLastLine: false,
    padding: { top: 12 },
    renderLineHighlight: 'all',
    tabSize: 4,
  })
  editor.onDidChangeModelContent(() => {
    const v = editor!.getValue()
    if (v !== props.modelValue) emit('update:modelValue', v)
    monaco.editor.setModelMarkers(editor!.getModel()!, 'rustc', [])
  })
  // 覆寫 Monaco 預設的 Ctrl+Enter
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => emit('run'))
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter, () => emit('submit'))
  decorations = editor.createDecorationsCollection()
})

watch(
  () => props.modelValue,
  (v) => {
    if (editor && v !== editor.getValue()) editor.setValue(v)
  },
)
watch(
  () => [props.look, props.dark],
  () => {
    monaco.editor.setTheme(themeName(props.look, props.dark))
    editor?.updateOptions({ fontFamily: FONTS[props.look], fontSize: FONT_SIZE[props.look], fontLigatures: props.look === 'C' })
    document.fonts?.ready.then(() => monaco.editor.remeasureFonts())
  },
)
watch(
  () => props.diagnostics,
  (diags) => {
    const model = editor?.getModel()
    if (!model) return
    const markers = diags
      .filter((d) => d.line !== null && (d.level === 'error' || d.level === 'warning'))
      .map((d) => {
        const line = d.line!
        const col = d.column ?? 1
        let endLine = d.end_line ?? line
        let endCol = d.end_column ?? model.getLineMaxColumn(endLine)
        if (endLine === line && endCol <= col) {
          const word = model.getWordAtPosition({ lineNumber: line, column: col })
          endCol = word ? word.endColumn : model.getLineMaxColumn(line)
        }
        const human = humanize(d.code, d.message)
        return {
          severity: d.level === 'error' ? monaco.MarkerSeverity.Error : monaco.MarkerSeverity.Warning,
          message: human ? `${human}\n\n${d.message}` : d.message,
          code: d.code && d.explanation_url ? { value: d.code, target: monaco.Uri.parse(d.explanation_url) } : (d.code ?? undefined),
          startLineNumber: line,
          startColumn: col,
          endLineNumber: endLine,
          endColumn: endCol,
          source: 'rustc',
        }
      })
    monaco.editor.setModelMarkers(model, 'rustc', markers)
  },
)

function reveal(line: number, column: number) {
  if (!editor) return
  editor.revealLineInCenter(line)
  editor.setPosition({ lineNumber: line, column })
  editor.focus()
  decorations?.set([{ range: new monaco.Range(line, 1, line, 1), options: { isWholeLine: true, className: 'line-flash' } }])
  window.setTimeout(() => decorations?.clear(), 900)
}
defineExpose({ reveal })
onBeforeUnmount(() => editor?.dispose())
</script>

<template>
  <div ref="host" class="code-editor" />
</template>

<style scoped>
.code-editor { width: 100%; height: 100%; min-height: 200px; }
:deep(.line-flash) { background: var(--brand-soft); animation: lf .9s; }
@keyframes lf { from { background: var(--brand); } }
</style>

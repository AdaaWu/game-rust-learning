import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { Diagnostic } from '../../api/types'
import { diag } from '../../test/fixtures'

// Monaco 在 jsdom 下無法運作，整個 mock
const m = vi.hoisted(() => {
  const state = {
    value: '',
    changeListener: null as null | (() => void),
    mouseDownListener: null as null | ((e: unknown) => void),
    hoverProvider: null as null | { provideHover: (model: unknown, position: { lineNumber: number; column: number }) => unknown },
  }
  const model = { getLineMaxColumn: vi.fn(() => 21), getWordAtPosition: vi.fn(() => null) }
  const editor = {
    getModel: vi.fn(() => model),
    getValue: vi.fn(() => state.value),
    setValue: vi.fn((v: string) => {
      state.value = v
    }),
    addCommand: vi.fn(),
    onDidChangeModelContent: vi.fn((cb: () => void) => {
      state.changeListener = cb
      return { dispose: vi.fn() }
    }),
    onMouseDown: vi.fn((cb: (e: unknown) => void) => {
      state.mouseDownListener = cb
      return { dispose: vi.fn() }
    }),
    deltaDecorations: vi.fn((_old: string[], next: unknown[]) => next.map((_, i) => `d${i}`)),
    revealLineInCenterIfOutsideViewport: vi.fn(),
    revealLineInCenter: vi.fn(),
    setPosition: vi.fn(),
    focus: vi.fn(),
    dispose: vi.fn(),
  }
  const monaco = {
    editor: {
      create: vi.fn((_el: HTMLElement, opts: { value: string }) => {
        state.value = opts.value
        return editor
      }),
      setModelMarkers: vi.fn(),
      setTheme: vi.fn(),
      defineTheme: vi.fn(),
      MouseTargetType: { GUTTER_GLYPH_MARGIN: 2, CONTENT_TEXT: 6 },
    },
    languages: {
      registerHoverProvider: vi.fn((_lang: string, provider: typeof state.hoverProvider) => {
        state.hoverProvider = provider
        return { dispose: vi.fn() }
      }),
    },
    KeyMod: { CtrlCmd: 2048, Shift: 1024 },
    KeyCode: { Enter: 3 },
    MarkerSeverity: { Hint: 1, Info: 2, Warning: 4, Error: 8 },
    Range: class {
      startLineNumber: number
      startColumn: number
      endLineNumber: number
      endColumn: number
      constructor(startLineNumber: number, startColumn: number, endLineNumber: number, endColumn: number) {
        this.startLineNumber = startLineNumber
        this.startColumn = startColumn
        this.endLineNumber = endLineNumber
        this.endColumn = endColumn
      }
    },
    Uri: { parse: (s: string) => ({ toString: () => s }) },
  }
  return { state, model, editor, monaco }
})

vi.mock('monaco-editor', () => ({ ...m.monaco, default: m.monaco }))
vi.mock('monaco-editor/features/register.all', () => ({}))
vi.mock('monaco-editor/languages/definitions/rust/register', () => ({}))

import CodeEditor from '../CodeEditor.vue'

type EditorProps = {
  modelValue: string
  diagnostics: Diagnostic[]
  panicStderr: string | null
  highlightLine: number | null
  theme: 'light' | 'dark'
}

function mountEditor(props: Partial<EditorProps> = {}) {
  return mount(CodeEditor, {
    props: { modelValue: 'fn main() {}\n', diagnostics: [], panicStderr: null, highlightLine: null, theme: 'light', ...props },
  })
}


function commandFor(keybinding: number) {
  const call = m.editor.addCommand.mock.calls.find((c) => c[0] === keybinding)
  return call?.[1] as (() => void) | undefined
}

describe('F6 CodeEditor（Monaco 包裝）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    m.state.changeListener = null
    m.state.mouseDownListener = null
  })

  it('建立 rust 語言的 Monaco 並帶入程式碼', () => {
    mountEditor({ modelValue: 'fn main() { 1 }' })
    expect(m.monaco.editor.create).toHaveBeenCalledTimes(1)
    const opts = m.monaco.editor.create.mock.calls[0][1] as unknown as { value: string; language: string }
    expect(opts.value).toBe('fn main() { 1 }')
    expect(opts.language).toBe('rust')
  })

  it('覆寫 Ctrl/Cmd+Enter 為 Run', () => {
    const w = mountEditor()
    const run = commandFor(m.monaco.KeyMod.CtrlCmd | m.monaco.KeyCode.Enter)
    expect(run).toBeTypeOf('function')
    run?.()
    expect(w.emitted('run')).toHaveLength(1)
  })

  it('覆寫 Ctrl/Cmd+Shift+Enter 為 Submit', () => {
    const w = mountEditor()
    const submit = commandFor(m.monaco.KeyMod.CtrlCmd | m.monaco.KeyMod.Shift | m.monaco.KeyCode.Enter)
    expect(submit).toBeTypeOf('function')
    submit?.()
    expect(w.emitted('submit')).toHaveLength(1)
  })

  it('diagnostics 轉成 Monaco marker 並帶錯誤碼連結', async () => {
    const w = mountEditor()
    await w.setProps({ diagnostics: [diag()] })
    const last = m.monaco.editor.setModelMarkers.mock.calls.at(-1)
    expect(last?.[0]).toBe(m.model)
    const data = last?.[2] as Array<Record<string, unknown>>
    expect(data).toHaveLength(1)
    expect(data[0]).toMatchObject({
      startLineNumber: 10,
      startColumn: 17,
      endLineNumber: 10,
      endColumn: 20,
      severity: 8,
      message: '`bag` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
    })
    const code = data[0].code as { value: string; target: { toString(): string } }
    expect(code.value).toBe('E0382')
    expect(code.target.toString()).toBe('https://doc.rust-lang.org/error_codes/E0382.html')
  })

  it('滑鼠移到 marker 上：hover 顯示人話 markdown 與錯誤碼連結', async () => {
    const w = mountEditor()
    await w.setProps({ diagnostics: [diag()] })
    expect(m.monaco.languages.registerHoverProvider).toHaveBeenCalledWith('rust', expect.anything())
    const hover = m.state.hoverProvider?.provideHover(m.model, { lineNumber: 10, column: 18 }) as {
      contents: { value: string }[]
    }
    expect(hover.contents.map((c) => c.value)).toEqual([
      '`bag` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
      '[E0382](https://doc.rust-lang.org/error_codes/E0382.html)',
    ])
    expect(m.state.hoverProvider?.provideHover(m.model, { lineNumber: 9, column: 18 })).toBeNull()
  })

  it('沒有錯誤碼網址時 code 只有字串或省略', async () => {
    const w = mountEditor()
    await w.setProps({ diagnostics: [diag({ level: 'warning', code: 'unused_variables', message: 'unused variable: `x`', explanation_url: null })] })
    const data = m.monaco.editor.setModelMarkers.mock.calls.at(-1)?.[2] as Array<Record<string, unknown>>
    expect(data[0].severity).toBe(4)
    expect(typeof data[0].code === 'object').toBe(false)
  })

  it('玩家編輯時發出新程式碼並清除 markers', async () => {
    const w = mountEditor({ diagnostics: [diag()] })
    m.state.value = 'fn main() { }\n'
    m.state.changeListener?.()
    expect(w.emitted('update:modelValue')?.at(-1)).toEqual(['fn main() { }\n'])
    expect(m.monaco.editor.setModelMarkers.mock.calls.at(-1)?.[2]).toEqual([])
  })

  it('line = null 的診斷不畫 marker', async () => {
    const w = mountEditor()
    await w.setProps({ diagnostics: [diag({ line: null, column: null, end_line: null, end_column: null })] })
    expect(m.monaco.editor.setModelMarkers.mock.calls.at(-1)?.[2]).toEqual([])
  })

  it('panicStderr 有 main.rs 位置時該行整行畫 error marker', async () => {
    const w = mountEditor()
    await w.setProps({ panicStderr: "thread 'main' panicked at /tmp/main.rs:15:11:\nboom\n" })
    const data = m.monaco.editor.setModelMarkers.mock.calls.at(-1)?.[2] as Array<Record<string, unknown>>
    expect(data).toHaveLength(1)
    expect(data[0]).toMatchObject({ startLineNumber: 15, startColumn: 1, endLineNumber: 15, endColumn: 21, severity: 8 })
  })

  it('外部改 modelValue（例如重設）時更新編輯器內容', async () => {
    const w = mountEditor({ modelValue: 'a' })
    await w.setProps({ modelValue: 'fn main() {}\n' })
    expect(m.editor.setValue).toHaveBeenCalledWith('fn main() {}\n')
  })

  it('modelValue 與編輯器內容相同時不呼叫 setValue（避免游標跳動）', async () => {
    const w = mountEditor({ modelValue: 'a' })
    m.state.value = 'ab'
    m.state.changeListener?.()
    await w.setProps({ modelValue: 'ab' })
    expect(m.editor.setValue).not.toHaveBeenCalled()
  })

  it('highlightLine 加整行裝飾並捲到可見範圍', async () => {
    const w = mountEditor()
    await w.setProps({ highlightLine: 8 })
    const [, decorations] = m.editor.deltaDecorations.mock.calls.at(-1) as [string[], Array<{ range: { startLineNumber: number }; options: { isWholeLine?: boolean } }>]
    expect(decorations).toHaveLength(1)
    expect(decorations[0].range.startLineNumber).toBe(8)
    expect(decorations[0].options.isWholeLine).toBe(true)
    expect(m.editor.revealLineInCenterIfOutsideViewport).toHaveBeenCalledWith(8)
  })

  it('換行時移除上一行的裝飾', async () => {
    const w = mountEditor()
    await w.setProps({ highlightLine: 8 })
    await w.setProps({ highlightLine: 3 })
    const [old] = m.editor.deltaDecorations.mock.calls.at(-1) as [string[], unknown[]]
    expect(old).toEqual(['d0'])
  })

  it('highlightLine 為 null 時移除所有裝飾', async () => {
    const w = mountEditor()
    await w.setProps({ highlightLine: 8 })
    await w.setProps({ highlightLine: null })
    expect(m.editor.deltaDecorations.mock.calls.at(-1)?.[1]).toEqual([])
  })

  it('點行號左側 glyph 區發出 gutter-click', () => {
    const w = mountEditor()
    m.state.mouseDownListener?.({ target: { type: 2, position: { lineNumber: 4, column: 1 } } })
    expect(w.emitted('gutter-click')).toEqual([[4]])
  })

  it('點程式碼內容不發出 gutter-click', () => {
    const w = mountEditor()
    m.state.mouseDownListener?.({ target: { type: 6, position: { lineNumber: 4, column: 3 } } })
    expect(w.emitted('gutter-click')).toBeUndefined()
  })

  it('revealPosition 捲到該行並把游標移到該欄', () => {
    const w = mountEditor()
    w.vm.revealPosition(10, 17)
    expect(m.editor.revealLineInCenter).toHaveBeenCalledWith(10)
    expect(m.editor.setPosition).toHaveBeenCalledWith({ lineNumber: 10, column: 17 })
    expect(m.editor.focus).toHaveBeenCalled()
  })

  it('深淺色切換 Monaco 主題', async () => {
    const w = mountEditor({ theme: 'light' })
    expect(m.monaco.editor.setTheme).toHaveBeenLastCalledWith('ocean-light')
    await w.setProps({ theme: 'dark' })
    expect(m.monaco.editor.setTheme).toHaveBeenLastCalledWith('ocean-dark')
  })

  it('卸載時釋放 Monaco', () => {
    const w = mountEditor()
    w.unmount()
    expect(m.editor.dispose).toHaveBeenCalled()
  })
})

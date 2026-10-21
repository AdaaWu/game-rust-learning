import * as monaco from 'monaco-editor/editor/editor.api'
import 'monaco-editor/languages/definitions/rust/register'
import EditorWorker from 'monaco-editor/editor/editor.worker?worker'

;(self as unknown as { MonacoEnvironment: unknown }).MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
}

type Look = 'A' | 'B' | 'C'
interface Pal { base: 'vs' | 'vs-dark'; bg: string; fg: string; line: string; sel: string; kw: string; str: string; num: string; com: string; fn: string; ty: string; gutter: string }

const PALETTES: Record<string, Pal> = {
  'A-light': { base: 'vs', bg: '#FFF8E1', fg: '#2B1D14', line: '#F3E3B5', sel: '#F5C77E', kw: '#B3261E', str: '#2E7D32', num: '#6A1B9A', com: '#8D7B68', fn: '#1565C0', ty: '#C25E00', gutter: '#B59F7B' },
  'A-dark': { base: 'vs-dark', bg: '#1A1A36', fg: '#F2E6C9', line: '#26264A', sel: '#44447A', kw: '#FF7A7A', str: '#9CE37D', num: '#D8A8FF', com: '#7E7EA8', fn: '#7CC8FF', ty: '#FFB547', gutter: '#6E6EA0' },
  'B-light': { base: 'vs', bg: '#FFFCF5', fg: '#4A3426', line: '#FFF1DC', sel: '#FFD9C2', kw: '#D9534F', str: '#3A9188', num: '#B5649B', com: '#B49A84', fn: '#2F7FA6', ty: '#C9762E', gutter: '#C9AE93' },
  'B-dark': { base: 'vs-dark', bg: '#123244', fg: '#FDEBD3', line: '#183E52', sel: '#235B74', kw: '#FF9B85', str: '#8FE3CF', num: '#F5B8DA', com: '#7FA3B3', fn: '#8CCBFF', ty: '#FFC98A', gutter: '#5F8DA0' },
  'C-light': { base: 'vs', bg: '#FFFFFF', fg: '#0F1720', line: '#F1F5FB', sel: '#CFE3FF', kw: '#C2185B', str: '#00897B', num: '#6A3FD8', com: '#8A97A8', fn: '#0062D1', ty: '#B35A00', gutter: '#9AA7B8' },
  'C-dark': { base: 'vs-dark', bg: '#0B1018', fg: '#C8D3E0', line: '#111A26', sel: '#1C3350', kw: '#FF4FA3', str: '#3DFFA2', num: '#C792EA', com: '#4D5B6E', fn: '#29D9FF', ty: '#FFB86B', gutter: '#3A4A5E' },
}

for (const [name, p] of Object.entries(PALETTES)) {
  const c = (h: string) => h.slice(1)
  monaco.editor.defineTheme(`rq-${name}`, {
    base: p.base,
    inherit: true,
    rules: [
      { token: '', foreground: c(p.fg) },
      { token: 'keyword', foreground: c(p.kw), fontStyle: 'bold' },
      { token: 'string', foreground: c(p.str) },
      { token: 'number', foreground: c(p.num) },
      { token: 'comment', foreground: c(p.com), fontStyle: 'italic' },
      { token: 'type', foreground: c(p.ty) },
      { token: 'type.identifier', foreground: c(p.ty) },
      { token: 'identifier', foreground: c(p.fg) },
      { token: 'delimiter', foreground: c(p.fg) },
      { token: 'variable', foreground: c(p.fn) },
    ],
    colors: {
      'editor.background': p.bg,
      'editor.foreground': p.fg,
      'editor.lineHighlightBackground': p.line,
      'editor.selectionBackground': p.sel,
      'editorLineNumber.foreground': p.gutter,
      'editorLineNumber.activeForeground': p.fg,
      'editorGutter.background': p.bg,
      'editorCursor.foreground': p.kw,
    },
  })
}

export function themeName(look: Look, dark: boolean) {
  return `rq-${look}-${dark ? 'dark' : 'light'}`
}

export const FONTS: Record<Look, string> = {
  A: "'VT323', 'JetBrains Mono', monospace",
  B: "'JetBrains Mono', ui-monospace, monospace",
  C: "'JetBrains Mono', ui-monospace, monospace",
}
export const FONT_SIZE: Record<Look, number> = { A: 19, B: 14, C: 14 }

export { monaco }

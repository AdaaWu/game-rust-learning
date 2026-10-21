import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

// 「mock 回應範例」直接取 api.md 中同名的 JSON 範例
const API_MD = fileURLToPath(new URL('../../../docs/specs/api.md', import.meta.url))

function section(md: string, start: string, end: string): string {
  const from = md.indexOf(start)
  const to = md.indexOf(end, from)
  if (from < 0 || to < 0) throw new Error(`api.md 找不到章節 ${start}`)
  return md.slice(from, to)
}

function examples(text: string): Map<string, unknown> {
  const map = new Map<string, unknown>()
  const re = /\*\*範例：(.+?)\*\*[\s\S]*?```json\n([\s\S]*?)\n```/g
  for (const m of text.matchAll(re)) map.set(m[1].trim(), JSON.parse(m[2]))
  return map
}

const md = readFileSync(API_MD, 'utf8')
const runExamples = examples(section(md, '## 3. POST /api/levels/:id/run', '## 4. POST'))
const submitExamples = examples(section(md, '## 4. POST /api/levels/:id/submit', '## 5. GET'))

export function apiExample(kind: 'Run' | 'Submit', name: string): unknown {
  const map = kind === 'Run' ? runExamples : submitExamples
  const value = map.get(name)
  if (value === undefined) throw new Error(`api.md 沒有 ${kind} 範例「${name}」，現有：${[...map.keys()].join('、')}`)
  return structuredClone(value)
}

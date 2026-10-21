import { expect, Step } from '../support/fixtures'
import { errorReply, ok } from '../support/mock'
import type { MockApi, Reply } from '../support/mock'
import { apiExample } from '../support/apiExamples'
import { diag, e0384, runResponse, runWithoutExecution, submitResponse, unescape } from '../support/responses'

type Kind = 'Run' | 'Submit'

function setReplies(mock: MockApi, kind: string, replies: Reply[], delaySec?: string) {
  if (kind === 'Run') {
    mock.runReplies = replies
    if (delaySec) mock.runDelayMs = Number(delaySec) * 1000
  } else {
    mock.submitReplies = replies
    if (delaySec) mock.submitDelayMs = Number(delaySec) * 1000
  }
}

// 「通關路線」：撿完 2 顆寶石並停在終點的 20 行
const CLEAR_ROUTE = [
  ...Array(4).fill('MOVE DOWN'),
  'MOVE RIGHT',
  'MOVE RIGHT',
  ...Array(4).fill('MOVE UP'),
  'PICK',
  ...Array(4).fill('MOVE DOWN'),
  'OPEN RIGHT',
  'MOVE RIGHT',
  'MOVE RIGHT',
  'PICK',
  'MOVE RIGHT',
]
  .map((l) => `${l}\n`)
  .join('')

// ---------- 範例 ----------

Step(/^mock API 的 (Run|Submit) (?:延遲 ([0-9]+) 秒後)?回「(.+)」範例$/, async ({ mock }, kind: string, delay: string | undefined, name: string) => {
  setReplies(mock, kind, [ok(apiExample(kind as Kind, name))], delay)
})

Step(/^mock API 的 Run 回應：$/, async ({ mock }, doc: string) => {
  setReplies(mock, 'Run', [ok(JSON.parse(doc))])
})

// ---------- compile_error ----------

Step(/^mock API 的 (Run|Submit) 回 compile_error，diagnostics 只有一筆 E0384 error 位於第 ([0-9]+) 行$/, async ({ mock }, kind: string, line: string) => {
  const body = kind === 'Run' ? runWithoutExecution('compile_error') : submitResponse('compile_error')
  body.compile.diagnostics = [e0384(Number(line))]
  setReplies(mock, kind, [ok(body)])
})

Step(/^mock API 的 Run 回 compile_error，diagnostics 只有一筆：$/, async ({ mock }, doc: string) => {
  const body = runWithoutExecution('compile_error')
  body.compile.diagnostics = [JSON.parse(doc)]
  setReplies(mock, 'Run', [ok(body)])
})

Step(
  /^mock API 的 Run 回 compile_error，diagnostics 只有一筆 error：line ([0-9]+)、column ([0-9]+)、end_line ([0-9]+)、end_column (\d+|null)$/,
  async ({ mock }, line: string, col: string, endLine: string, endCol: string) => {
    const body = runWithoutExecution('compile_error')
    body.compile.diagnostics = [
      diag({
        code: 'E0308',
        message: 'mismatched types',
        line: Number(line),
        column: Number(col),
        end_line: Number(endLine),
        end_column: endCol === 'null' ? null : Number(endCol),
        rendered: 'error[E0308]: mismatched types\n',
      }),
    ]
    setReplies(mock, 'Run', [ok(body)])
  },
)

const nullable = (v: string) => (v === 'null' ? null : v)

Step(/^mock API 的 Run 回 compile_error，diagnostics 依序為：$/, async ({ mock }, table: any) => {
  const body = runWithoutExecution('compile_error')
  body.compile.diagnostics = table.hashes().map((r: Record<string, string>) => {
    const num = (v: string) => (v === 'null' ? null : Number(v))
    const code = nullable(r.code)
    return diag({
      code,
      message: code ? `${code} message` : `${r.level} message`,
      level: r.level,
      line: num(r.line),
      column: num(r.column),
      end_line: num(r.end_line),
      end_column: num(r.end_column),
      rendered: nullable(r.rendered),
      explanation_url: code ? `https://doc.rust-lang.org/error_codes/${code}.html` : null,
    })
  })
  setReplies(mock, 'Run', [ok(body)])
})

Step(/^mock API 的 Run 回 compile_error，diagnostics 有 ([0-9]+) 筆 error$/, async ({ mock }, count: string) => {
  const body = runWithoutExecution('compile_error')
  body.compile.diagnostics = Array.from({ length: Number(count) }, (_, i) => e0384(i + 2))
  setReplies(mock, 'Run', [ok(body)])
})

// ---------- 依 status ----------

Step(/^mock API 的 Submit 回 test_failed，"tests" 為：$/, async ({ mock }, table: any) => {
  const body = submitResponse('test_failed')
  body.tests = table.hashes().map((r: Record<string, string>) => ({ name: r.name, status: r.status, message: nullable(r.message) }))
  setReplies(mock, 'Submit', [ok(body)])
})

Step(/^mock API 的 Submit (?:延遲 ([0-9]+) 秒後)?回 status "(\w+)"$/, async ({ mock }, delay: string | undefined, status: string) => {
  setReplies(mock, 'Submit', [ok(submitResponse(status))], delay)
})

Step(/^mock API 的 Submit 依序回：(.+)$/, async ({ mock }, list: string) => {
  const statuses = [...list.matchAll(/status "(\w+)"/g)].map((m) => m[1])
  setReplies(mock, 'Submit', statuses.map((s) => ok(submitResponse(s))))
})

Step(/^mock API 的 Run 回 status "(\w+)"$/, async ({ mock }, status: string) => {
  setReplies(mock, 'Run', [ok(status === 'compile_error' ? runWithoutExecution(status) : runResponse(status))])
})

Step(/^mock API 的 Run 回 status "(\w+)"，stdout 為「通關路線」$/, async ({ mock }, status: string) => {
  setReplies(mock, 'Run', [ok(runResponse(status, CLEAR_ROUTE))])
})

Step(/^mock API 的 Run 回 status "(\w+)"，stdout 為：$/, async ({ mock }, status: string, doc: string) => {
  setReplies(mock, 'Run', [ok(runResponse(status, `${doc}\n`))])
})

Step(
  /^mock API 的 Run 回 status "(\w+)"，stdout 為 "((?:[^"\\]|\\.)*)"(?:，stdout_truncated 為 (true|false))?$/,
  async ({ mock }, status: string, stdout: string, truncated: string | undefined) => {
    setReplies(mock, 'Run', [ok(runResponse(status, unescape(stdout), truncated === 'true'))])
  },
)

Step(/^mock API 的 Run 回 status "(\w+)"，execution 為 null$/, async ({ mock }, status: string) => {
  setReplies(mock, 'Run', [ok(runWithoutExecution(status))])
})

Step(/^mock API 的 Run 回 status "ok"，stdout 為 ([0-9]+) 行，依序交替 "MOVE DOWN"、"MOVE UP"$/, async ({ mock }, count: string) => {
  const lines = Array.from({ length: Number(count) }, (_, i) => (i % 2 === 0 ? 'MOVE DOWN' : 'MOVE UP'))
  setReplies(mock, 'Run', [ok(runResponse('ok', lines.join('\n') + '\n'))])
})

Step(/^mock API 的 Run 回 status "ok"，stdout 為 ([0-9]+) 行交替 "MOVE DOWN"、"MOVE UP"，再接 1 行 "JUMP"$/, async ({ mock }, count: string) => {
  const lines = Array.from({ length: Number(count) }, (_, i) => (i % 2 === 0 ? 'MOVE DOWN' : 'MOVE UP'))
  setReplies(mock, 'Run', [ok(runResponse('ok', [...lines, 'JUMP'].join('\n') + '\n'))])
})

Step('mock API 的 Run 改為延遲 2 秒後回應', async ({ mock }) => {
  mock.runDelayMs = 2000
})

// ---------- 請求失敗 ----------

Step(/^mock API 的 GET "\/api\/levels\/([^"]+)" 回 404，error\.code 為 "(\w+)"$/, async ({ mock }, id: string, code: string) => {
  mock.detailOverrides.set(id, errorReply(404, code))
})

Step(/^mock API 的 (Run|Submit) 回 (\d{3})，(.+)$/, async ({ mock, world }, kind: string, status: string, rest: string) => {
  const code = /error\.code 為 "(\w+)"/.exec(rest)?.[1]
  const retry = /標頭 "Retry-After" 為 "([^"]*)"/.exec(rest)?.[1]
  const text = /body 為純文字 "([^"]*)"/.exec(rest)?.[1]
  let reply: Reply
  if (text !== undefined) reply = { status: Number(status), text }
  else {
    if (!code) throw new Error(`看不懂的失敗回應：${rest}`)
    reply = errorReply(Number(status), code, retry !== undefined ? { 'Retry-After': retry } : undefined)
    // 忙碌倒數改由「經過 N 秒」推進時鐘
    if (code === 'JUDGE_BUSY') world.pauseClockOnRequest = true
  }
  setReplies(mock, kind, [reply])
})

Step(/^mock API 的 (Run|Submit) 連線中斷$/, async ({ mock }, kind: string) => {
  setReplies(mock, kind, [{ status: 0, abort: true }])
})

Step(/^mock API 的 (Run|Submit) 超過 60 秒沒有回應$/, async ({ mock, world }, kind: string) => {
  setReplies(mock, kind, [{ status: 0, hang: true }])
  await world.installClock()
  world.fastForwardMs = 61_000
})

// ---------- 請求計數 ----------

Step(/^mock API (?:共)?收到 ([0-9]+) 次 (Run|Submit) 請求$/, async ({ mock, page }, n: string, kind: string) => {
  const key = kind === 'Run' ? 'run' : 'submit'
  await expect.poll(() => mock.counts[key]).toBe(Number(n))
  await page.waitForTimeout(200)
  expect(mock.counts[key]).toBe(Number(n))
})

Step(/^mock API 共收到 ([0-9]+) 次 Run 請求、([0-9]+) 次 Submit 請求$/, async ({ mock }, run: string, submit: string) => {
  expect(mock.counts.run).toBe(Number(run))
  expect(mock.counts.submit).toBe(Number(submit))
})

Step(/^mock API 收到 ([0-9]+) 次 GET "\/api\/levels" 請求$/, async ({ mock }, n: string) => {
  expect(mock.counts.list).toBe(Number(n))
})

Step('等待請求完成', async ({ world }) => {
  await world.waitIdle()
})

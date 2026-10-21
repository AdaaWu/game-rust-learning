import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import type { Route } from '@playwright/test'

type Json = any

export interface Reply {
  status: number
  json?: Json
  text?: string
  headers?: Record<string, string>
  // 連線中斷
  abort?: boolean
  // 永不回應
  hang?: boolean
}

const LEVELS = JSON.parse(
  readFileSync(fileURLToPath(new URL('../fixtures/levels.json', import.meta.url)), 'utf8'),
) as { list: Json; details: Record<string, Json> }

export const MVP_IDS: string[] = LEVELS.list.levels.map((l: Json) => l.id)

export function levelDetail(id: string): Json {
  return structuredClone(LEVELS.details[id])
}

// ux.md §4.2 範例地圖
export const FIXTURE_GRID = {
  width: 6,
  height: 5,
  start: [0, 0],
  goal: [5, 4],
  walls: [
    [1, 0],
    [1, 1],
    [1, 2],
    [3, 2],
    [3, 3],
    [4, 3],
  ],
  gems: [
    [2, 0],
    [4, 4],
  ],
  doors: [[3, 4]],
}

export const ok = (json: Json): Reply => ({ status: 200, json })

export function errorReply(status: number, code: string, headers?: Record<string, string>): Reply {
  return { status, json: { error: { code, message: code } }, headers }
}

export class MockApi {
  // @docker：不攔截，經 Vite proxy 打真後端
  real = false
  list: Json = structuredClone(LEVELS.list)
  details: Record<string, Json> = structuredClone(LEVELS.details)
  detailOverrides = new Map<string, Reply>()
  health: 'ok' | 'degraded' = 'ok'
  runReplies: Reply[] = []
  submitReplies: Reply[] = []
  runDelayMs = 0
  submitDelayMs = 0
  counts = { run: 0, submit: 0, list: 0, health: 0 }
  inflight = 0

  useFixtureGrid() {
    this.list = {
      chapters: [{ chapter: 1, title: '沙灘甦醒' }],
      levels: [{ id: 'fixture-grid', chapter: 1, order: 1, title: '測試地圖', kind: 'build', visual: 'grid' }],
    }
    this.details = {
      'fixture-grid': {
        ...this.list.levels[0],
        story: '測試用地圖',
        concept: '測試用地圖',
        hints: ['提示一'],
        starter: 'fn main() {\n    println!("MOVE DOWN");\n}\n',
        grid: structuredClone(FIXTURE_GRID),
        memory_snapshots: null,
      },
    }
  }

  private next(queue: Reply[]): Reply | undefined {
    return queue.length > 1 ? queue.shift() : queue[0]
  }

  async handle(route: Route) {
    if (this.real) return route.fallback()
    const request = route.request()
    const path = new URL(request.url()).pathname
    if (path === '/api/health') {
      this.counts.health++
      const degraded = this.health === 'degraded'
      return this.fulfill(route, ok({ status: this.health, judge: { available: !degraded, reason: degraded ? 'docker_unreachable' : null } }))
    }
    if (path === '/api/levels' && request.method() === 'GET') {
      this.counts.list++
      return this.fulfill(route, ok(this.list))
    }
    const m = /^\/api\/levels\/([^/]+)(?:\/(run|submit))?$/.exec(path)
    if (!m) return this.fulfill(route, errorReply(404, 'NOT_FOUND'))
    const id = decodeURIComponent(m[1])
    const action = m[2] as 'run' | 'submit' | undefined
    if (!action) {
      const override = this.detailOverrides.get(id)
      if (override) return this.fulfill(route, override)
      const detail = this.details[id]
      return this.fulfill(route, detail ? ok(detail) : errorReply(404, 'LEVEL_NOT_FOUND'))
    }
    this.counts[action]++
    const reply = this.next(action === 'run' ? this.runReplies : this.submitReplies) ?? errorReply(500, 'NO_MOCK')
    const delay = action === 'run' ? this.runDelayMs : this.submitDelayMs
    // 永不回應：不計入進行中，由前端逾時中止
    if (reply.hang) return
    this.inflight++
    try {
      if (delay > 0) await new Promise((r) => setTimeout(r, delay))
      if (reply.abort) {
        await route.abort('failed').catch(() => {})
        return
      }
      await this.fulfill(route, reply)
    } finally {
      this.inflight--
    }
  }

  private async fulfill(route: Route, reply: Reply) {
    const body = reply.json !== undefined ? JSON.stringify(reply.json) : (reply.text ?? '')
    const contentType = reply.json !== undefined ? 'application/json; charset=utf-8' : 'text/plain; charset=utf-8'
    await route
      .fulfill({ status: reply.status, body, headers: { 'content-type': contentType, ...(reply.headers ?? {}) } })
      .catch(() => {})
  }
}

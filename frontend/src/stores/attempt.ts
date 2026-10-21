import { defineStore } from 'pinia'
import { api } from '../api'
import type { ApiResult } from '../api/client'
import type { RunResponse, SubmitResponse } from '../api/types'
import type { ConsoleResult } from '../domain/console'
import type { StarSet } from '../domain/progress'
import { useProgressStore } from './progress'

export type Phase = 'idle' | 'running' | 'submitting'

// 超過此毫秒數仍未回應時提示第一次編譯比較久
export const SLOW_HINT_MS = 3000

// Run／Submit 請求逾時，逾時即 abort 並歸為 network_error
export const REQUEST_TIMEOUT_MS = 60_000

// 進行中請求的控制物件不放進 state
let controller: AbortController | null = null
let busyTimer: ReturnType<typeof setInterval> | null = null

function stopBusyTimer() {
  if (busyTimer !== null) clearInterval(busyTimer)
  busyTimer = null
}

export const useAttemptStore = defineStore('attempt', {
  state: () => ({
    levelId: null as string | null,
    phase: 'idle' as Phase,
    result: null as ConsoleResult | null,
    lastRun: null as RunResponse | null,
    lastSubmit: null as SubmitResponse | null,
    slow: false,
    // server_busy 倒數剩餘秒數
    busySeconds: 0,
    // /api/health 回 degraded
    judgeDegraded: false,
    // 請求期間程式碼被改過
    codeChangedDuringRequest: false,
    // 本次通關拿到的星；null 表示沒有通關結算
    clearedStars: null as StarSet | null,
  }),
  getters: {
    canRequest: (state): boolean => state.phase === 'idle' && state.busySeconds === 0 && !state.judgeDegraded,
  },
  actions: {
    // 進入關卡：取消進行中的請求、清空結果
    enterLevel(levelId: string) {
      controller?.abort()
      controller = null
      stopBusyTimer()
      this.levelId = levelId
      this.phase = 'idle'
      this.result = null
      this.lastRun = null
      this.lastSubmit = null
      this.slow = false
      this.busySeconds = 0
      this.codeChangedDuringRequest = false
      this.clearedStars = null
    },
    async checkHealth() {
      const result = await api.health()
      this.judgeDegraded = result.ok && result.data.status === 'degraded'
    },
    async request<T>(
      action: 'run' | 'submit',
      call: (id: string, signal: AbortSignal) => Promise<ApiResult<T>>,
    ): Promise<{ ok: true; data: T } | null> {
      if (!this.canRequest || this.levelId === null) return null
      const levelId = this.levelId
      const own = new AbortController()
      controller = own
      this.phase = action === 'run' ? 'running' : 'submitting'
      this.result = null
      this.slow = false
      this.codeChangedDuringRequest = false
      const slowTimer = setTimeout(() => {
        if (controller === own) this.slow = true
      }, SLOW_HINT_MS)
      const timeoutTimer = setTimeout(() => own.abort(), REQUEST_TIMEOUT_MS)
      let result: ApiResult<T>
      try {
        result = await call(levelId, own.signal)
      } finally {
        clearTimeout(slowTimer)
        clearTimeout(timeoutTimer)
      }
      // 已切換關卡：忽略結果
      if (controller !== own) return null
      controller = null
      this.phase = 'idle'
      this.slow = false
      if (!result.ok) {
        this.result = { action, failure: result.failure }
        if (result.failure.kind === 'server_busy') this.startBusyCountdown(result.failure.retryAfterSeconds)
        return null
      }
      return result
    },
    startBusyCountdown(seconds: number) {
      stopBusyTimer()
      this.busySeconds = seconds
      busyTimer = setInterval(() => {
        this.busySeconds = Math.max(0, this.busySeconds - 1)
        if (this.busySeconds === 0) stopBusyTimer()
      }, 1000)
    },
    async run(code: string) {
      const result = await this.request<RunResponse>('run', (id, signal) => api.run(id, code, signal))
      if (!result) return
      this.lastRun = result.data
      this.result = { action: 'run', data: result.data }
    },
    async submit(code: string) {
      const result = await this.request<SubmitResponse>('submit', (id, signal) => api.submit(id, code, signal))
      if (!result || this.levelId === null) return
      const data = result.data
      this.lastSubmit = data
      this.result = { action: 'submit', data }
      const progress = useProgressStore()
      if (data.status === 'ok' && data.passed) {
        this.clearedStars = progress.recordPass(this.levelId)
      } else {
        progress.recordSubmit(this.levelId, data.status)
      }
    },
    noteCodeChange() {
      if (this.phase !== 'idle') this.codeChangedDuringRequest = true
    },
    dismissClear() {
      this.clearedStars = null
    },
  },
})

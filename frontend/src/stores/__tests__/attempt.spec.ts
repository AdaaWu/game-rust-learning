import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { runCompileError, runOk, submitResult } from '../../test/fixtures'

const api = vi.hoisted(() => ({
  listLevels: vi.fn(),
  getLevel: vi.fn(),
  run: vi.fn(),
  submit: vi.fn(),
  health: vi.fn(),
}))
vi.mock('../../api', () => ({ api }))

import { REQUEST_TIMEOUT_MS, SLOW_HINT_MS, useAttemptStore } from '../attempt'
import { useProgressStore } from '../progress'

const ID = '01-hello-ferris'

function deferred<T>() {
  let resolve!: (v: T) => void
  const promise = new Promise<T>((r) => {
    resolve = r
  })
  return { promise, resolve }
}

function setup() {
  const progress = useProgressStore()
  progress.load()
  const attempt = useAttemptStore()
  attempt.enterLevel(ID)
  return { attempt, progress }
}

describe('F7 useAttemptStore 狀態機', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
    vi.useFakeTimers()
    Object.values(api).forEach((f) => f.mockReset())
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('初始為 idle、可送出', () => {
    const { attempt } = setup()
    expect(attempt.phase).toBe('idle')
    expect(attempt.canRequest).toBe(true)
  })

  it('Run 請求中 phase 為 running，回來後為 idle 並保存結果', async () => {
    const d = deferred<unknown>()
    api.run.mockReturnValue(d.promise)
    const { attempt } = setup()
    const p = attempt.run('code')
    expect(attempt.phase).toBe('running')
    expect(attempt.canRequest).toBe(false)
    d.resolve({ ok: true, data: runOk('MOVE DOWN\n') })
    await p
    expect(attempt.phase).toBe('idle')
    expect(attempt.lastRun).toEqual(runOk('MOVE DOWN\n'))
    expect(attempt.result).toEqual({ action: 'run', data: runOk('MOVE DOWN\n') })
    expect(api.run).toHaveBeenCalledWith(ID, 'code', expect.any(AbortSignal))
  })

  it('Submit 請求中 phase 為 submitting', async () => {
    const d = deferred<unknown>()
    api.submit.mockReturnValue(d.promise)
    const { attempt } = setup()
    const p = attempt.submit('code')
    expect(attempt.phase).toBe('submitting')
    d.resolve({ ok: true, data: submitResult('test_failed') })
    await p
    expect(attempt.phase).toBe('idle')
  })

  it('同一時間只允許一個請求', async () => {
    const d = deferred<unknown>()
    api.run.mockReturnValue(d.promise)
    const { attempt } = setup()
    const p = attempt.run('a')
    await attempt.run('b')
    await attempt.submit('c')
    d.resolve({ ok: true, data: runOk('') })
    await p
    expect(api.run).toHaveBeenCalledTimes(1)
    expect(api.submit).not.toHaveBeenCalled()
  })

  it('超過 3 秒仍未回應時 slow 為 true，回來後清除', async () => {
    const d = deferred<unknown>()
    api.run.mockReturnValue(d.promise)
    const { attempt } = setup()
    const p = attempt.run('a')
    expect(SLOW_HINT_MS).toBe(3000)
    vi.advanceTimersByTime(2999)
    expect(attempt.slow).toBe(false)
    vi.advanceTimersByTime(1)
    expect(attempt.slow).toBe(true)
    d.resolve({ ok: true, data: runOk('') })
    await p
    expect(attempt.slow).toBe(false)
  })

  it('請求期間改程式碼，結果標示為修改前的結果；下一次請求清除', async () => {
    const d = deferred<unknown>()
    api.run.mockReturnValue(d.promise)
    const { attempt } = setup()
    const p = attempt.run('a')
    attempt.noteCodeChange()
    d.resolve({ ok: true, data: runOk('') })
    await p
    expect(attempt.codeChangedDuringRequest).toBe(true)
    api.run.mockResolvedValue({ ok: true, data: runOk('') })
    await attempt.run('b')
    expect(attempt.codeChangedDuringRequest).toBe(false)
  })

  it('idle 時改程式碼不標示為修改前的結果', () => {
    const { attempt } = setup()
    attempt.noteCodeChange()
    expect(attempt.codeChangedDuringRequest).toBe(false)
  })

  describe('失敗計數', () => {
    it.each(['compile_error', 'test_failed', 'timeout', 'memory_limit', 'output_limit'] as const)('Submit %s 算一次失敗', async (status) => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult(status) })
      const { attempt, progress } = setup()
      await attempt.submit('x')
      expect(progress.levelProgress(ID).current.failedSubmits).toBe(1)
      expect(attempt.clearedStars).toBeNull()
    })

    it.each([
      { kind: 'server_busy', retryAfterSeconds: 2 },
      { kind: 'judge_unavailable' },
      { kind: 'payload_too_large' },
      { kind: 'network_error', status: null, code: null, message: null },
    ])('Submit 請求失敗 %o 不算失敗', async (failure) => {
      api.submit.mockResolvedValue({ ok: false, failure })
      const { attempt, progress } = setup()
      await attempt.submit('x')
      expect(progress.levelProgress(ID).current.failedSubmits).toBe(0)
      expect(attempt.result).toEqual({ action: 'submit', failure })
    })

    it('Run 失敗不影響一次就過', async () => {
      api.run.mockResolvedValue({ ok: true, data: runCompileError() })
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { attempt } = setup()
      await attempt.run('x')
      await attempt.submit('x')
      expect(attempt.clearedStars).toEqual({ pass: true, noHint: true, firstTry: true })
    })

    it('Submit 通過：結算星等寫入進度並設定 clearedStars', async () => {
      api.submit.mockResolvedValueOnce({ ok: true, data: submitResult('test_failed') }).mockResolvedValueOnce({ ok: true, data: submitResult('ok') })
      const { attempt, progress } = setup()
      await attempt.submit('x')
      await attempt.submit('x')
      expect(attempt.clearedStars).toEqual({ pass: true, noHint: true, firstTry: false })
      expect(progress.levelProgress(ID).stars.pass).toBe(true)
      expect(progress.levelProgress(ID).current).toEqual({ hintsRevealed: 0, failedSubmits: 0 })
    })

    it('dismissClear 清除通關結算', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { attempt } = setup()
      await attempt.submit('x')
      attempt.dismissClear()
      expect(attempt.clearedStars).toBeNull()
    })
  })

  describe('server_busy 倒數', () => {
    it('依 Retry-After 倒數，期間 Run 與 Submit 都不能送', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'server_busy', retryAfterSeconds: 2 } })
      const { attempt } = setup()
      await attempt.run('x')
      expect(attempt.busySeconds).toBe(2)
      expect(attempt.canRequest).toBe(false)
      vi.advanceTimersByTime(1000)
      expect(attempt.busySeconds).toBe(1)
      vi.advanceTimersByTime(1000)
      expect(attempt.busySeconds).toBe(0)
      expect(attempt.canRequest).toBe(true)
    })

    it('倒數中呼叫 submit 不送出', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'server_busy', retryAfterSeconds: 10 } })
      const { attempt } = setup()
      await attempt.run('x')
      await attempt.submit('x')
      expect(api.submit).not.toHaveBeenCalled()
    })

    it('倒數結束不自動重送', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'server_busy', retryAfterSeconds: 2 } })
      const { attempt } = setup()
      await attempt.run('x')
      vi.advanceTimersByTime(5000)
      expect(api.run).toHaveBeenCalledTimes(1)
    })

    it('network_error 不倒數，可立即重試', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'network_error', status: 500, code: 'INTERNAL_ERROR', message: 'x' } })
      const { attempt } = setup()
      await attempt.run('x')
      expect(attempt.busySeconds).toBe(0)
      expect(attempt.canRequest).toBe(true)
    })

    it('judge_unavailable 不倒數，可立即重試', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'judge_unavailable' } })
      const { attempt } = setup()
      await attempt.run('x')
      expect(attempt.busySeconds).toBe(0)
      expect(attempt.canRequest).toBe(true)
    })
  })

  describe('請求逾時 60 秒', () => {
    // 模擬 client：signal 被 abort 時回 network_error
    function hangUntilAbort(_id: string, _code: string, signal: AbortSignal) {
      return new Promise((resolve) => {
        signal.addEventListener('abort', () =>
          resolve({ ok: false, failure: { kind: 'network_error', status: null, code: null, message: null } }),
        )
      })
    }

    it('逾時常數為 60 秒', () => {
      expect(REQUEST_TIMEOUT_MS).toBe(60_000)
    })

    it('Run 超過 60 秒 abort 並顯示 network_error', async () => {
      let signal: AbortSignal | undefined
      api.run.mockImplementation((id: string, code: string, s: AbortSignal) => {
        signal = s
        return hangUntilAbort(id, code, s)
      })
      const { attempt } = setup()
      const p = attempt.run('x')
      vi.advanceTimersByTime(59_999)
      expect(signal?.aborted).toBe(false)
      vi.advanceTimersByTime(1)
      await p
      expect(signal?.aborted).toBe(true)
      expect(attempt.phase).toBe('idle')
      expect(attempt.result).toEqual({
        action: 'run',
        failure: { kind: 'network_error', status: null, code: null, message: null },
      })
    })

    it('Submit 逾時不算失敗嘗試', async () => {
      api.submit.mockImplementation(hangUntilAbort)
      const { attempt, progress } = setup()
      const p = attempt.submit('x')
      vi.advanceTimersByTime(60_000)
      await p
      expect(attempt.result).toMatchObject({ action: 'submit', failure: { kind: 'network_error' } })
      expect(progress.levelProgress(ID).current.failedSubmits).toBe(0)
    })

    it('60 秒內回應時不 abort', async () => {
      let signal: AbortSignal | undefined
      api.run.mockImplementation(async (_id: string, _code: string, s: AbortSignal) => {
        signal = s
        return { ok: true, data: runOk('') }
      })
      const { attempt } = setup()
      await attempt.run('x')
      vi.advanceTimersByTime(120_000)
      expect(signal?.aborted).toBe(false)
    })
  })

  describe('健康檢查與切換關卡', () => {
    it('health degraded 時停用 Run／Submit', async () => {
      api.health.mockResolvedValue({ ok: true, data: { status: 'degraded', judge: { available: false, reason: 'runner_image_missing' } } })
      const { attempt } = setup()
      await attempt.checkHealth()
      expect(attempt.judgeDegraded).toBe(true)
      expect(attempt.canRequest).toBe(false)
      await attempt.run('x')
      expect(api.run).not.toHaveBeenCalled()
    })

    it('health ok 時可送出', async () => {
      api.health.mockResolvedValue({ ok: true, data: { status: 'ok', judge: { available: true, reason: null } } })
      const { attempt } = setup()
      await attempt.checkHealth()
      expect(attempt.judgeDegraded).toBe(false)
      expect(attempt.canRequest).toBe(true)
    })

    it('切換關卡時取消進行中的請求並忽略其結果', async () => {
      const d = deferred<unknown>()
      let signal: AbortSignal | undefined
      api.run.mockImplementation((_id: string, _code: string, s: AbortSignal) => {
        signal = s
        return d.promise
      })
      const { attempt } = setup()
      const p = attempt.run('x')
      attempt.enterLevel('02-mut-matters')
      expect(signal?.aborted).toBe(true)
      d.resolve({ ok: true, data: runOk('late') })
      await p
      expect(attempt.result).toBeNull()
      expect(attempt.phase).toBe('idle')
      expect(attempt.levelId).toBe('02-mut-matters')
    })
  })
})

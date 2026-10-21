import { describe, expect, it, vi } from 'vitest'
import { classifyFailure, createApiClient, DEFAULT_RETRY_AFTER_SECONDS, parseRetryAfter } from '../client'
import { gridLevel, levels, chapters, runOk, submitResult } from '../../test/fixtures'

function jsonResponse(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json', ...headers },
  })
}

function errorBody(code: string, message = '說明') {
  return { error: { code, message } }
}

function clientWith(response: Response | (() => Promise<Response>)) {
  const fetchMock = vi.fn<typeof fetch>(typeof response === 'function' ? response : async () => response)
  const client = createApiClient({ fetch: fetchMock })
  return { client, fetchMock }
}

describe('F1 parseRetryAfter', () => {
  it('整數秒數直接採用', () => {
    expect(parseRetryAfter('2')).toBe(2)
  })

  it('沒有 header 時用 10 秒', () => {
    expect(parseRetryAfter(null)).toBe(DEFAULT_RETRY_AFTER_SECONDS)
    expect(DEFAULT_RETRY_AFTER_SECONDS).toBe(10)
  })

  it.each(['abc', '', '-1', '1.5', '0'])('無法解析的值 %j 用 10 秒', (value) => {
    expect(parseRetryAfter(value)).toBe(10)
  })

  it('HTTP-date 格式不處理，退回 10 秒', () => {
    expect(parseRetryAfter('Wed, 21 Oct 2026 07:28:00 GMT')).toBe(10)
  })

  it('前後空白可容忍', () => {
    expect(parseRetryAfter(' 3 ')).toBe(3)
  })
})

describe('F1 classifyFailure', () => {
  const noHeaders = new Headers()

  it('503 JUDGE_BUSY 為 server_busy 並帶 Retry-After 秒數', () => {
    const headers = new Headers({ 'Retry-After': '2' })
    expect(classifyFailure(503, errorBody('JUDGE_BUSY'), headers)).toEqual({
      kind: 'server_busy',
      retryAfterSeconds: 2,
    })
  })

  it('503 JUDGE_BUSY 沒有 Retry-After 時倒數 10 秒', () => {
    expect(classifyFailure(503, errorBody('JUDGE_BUSY'), noHeaders)).toEqual({
      kind: 'server_busy',
      retryAfterSeconds: 10,
    })
  })

  it('503 JUDGE_UNAVAILABLE 為 judge_unavailable', () => {
    expect(classifyFailure(503, errorBody('JUDGE_UNAVAILABLE'), noHeaders)).toEqual({ kind: 'judge_unavailable' })
  })

  it('413 PAYLOAD_TOO_LARGE 為 payload_too_large', () => {
    expect(classifyFailure(413, errorBody('PAYLOAD_TOO_LARGE'), noHeaders)).toEqual({ kind: 'payload_too_large' })
  })

  it.each([
    [400, 'INVALID_REQUEST'],
    [404, 'LEVEL_NOT_FOUND'],
    [500, 'INTERNAL_ERROR'],
  ])('%i %s 歸為 network_error 並保留 code 與 message', (status, code) => {
    expect(classifyFailure(status, errorBody(code, '訊息'), noHeaders)).toEqual({
      kind: 'network_error',
      status,
      code,
      message: '訊息',
    })
  })

  it('413 沒有 JSON body 也歸 payload_too_large（只看狀態碼）', () => {
    expect(classifyFailure(413, '<html>Request Entity Too Large</html>', noHeaders)).toEqual({ kind: 'payload_too_large' })
  })

  it('503 但 code 不認得時歸為 network_error', () => {
    expect(classifyFailure(503, errorBody('SOMETHING_ELSE'), noHeaders)).toMatchObject({
      kind: 'network_error',
      status: 503,
    })
  })

  it('body 不是統一錯誤格式時歸為 network_error，code 與 message 為 null', () => {
    expect(classifyFailure(502, '<html>bad gateway</html>', noHeaders)).toEqual({
      kind: 'network_error',
      status: 502,
      code: null,
      message: null,
    })
  })
})

describe('F1 createApiClient', () => {
  it('listLevels 打 GET /api/levels 並回傳章節與關卡', async () => {
    const { client, fetchMock } = clientWith(jsonResponse(200, { chapters, levels }))
    const result = await client.listLevels()
    expect(result).toEqual({ ok: true, data: { chapters, levels } })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/levels')
  })

  it('getLevel 打 GET /api/levels/:id', async () => {
    const level = gridLevel()
    const { client, fetchMock } = clientWith(jsonResponse(200, level))
    const result = await client.getLevel('01-hello-ferris')
    expect(result).toEqual({ ok: true, data: level })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/levels/01-hello-ferris')
  })

  it('getLevel 回 404 時為 level_not_found', async () => {
    const { client } = clientWith(jsonResponse(404, errorBody('LEVEL_NOT_FOUND')))
    const result = await client.getLevel('99-unknown')
    expect(result).toEqual({ ok: false, failure: { kind: 'level_not_found' } })
  })

  it('run 以 POST JSON 送出 code', async () => {
    const response = runOk('MOVE DOWN\n')
    const { client, fetchMock } = clientWith(jsonResponse(200, response))
    const result = await client.run('01-hello-ferris', 'fn main() {}')
    expect(result).toEqual({ ok: true, data: response })
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toBe('/api/levels/01-hello-ferris/run')
    expect(init.method).toBe('POST')
    expect(new Headers(init.headers).get('Content-Type')).toBe('application/json')
    expect(JSON.parse(init.body as string)).toEqual({ code: 'fn main() {}' })
  })

  it('submit 打 POST /api/levels/:id/submit', async () => {
    const response = submitResult('ok')
    const { client, fetchMock } = clientWith(jsonResponse(200, response))
    const result = await client.submit('01-hello-ferris', 'fn main() {}')
    expect(result).toEqual({ ok: true, data: response })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/levels/01-hello-ferris/submit')
  })

  it('判題結果 compile_error 仍是 200 成功回應，不視為請求失敗', async () => {
    const response = { status: 'compile_error', compile: { success: false, timed_out: false, diagnostics: [] }, execution: null }
    const { client } = clientWith(jsonResponse(200, response))
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: true, data: response })
  })

  it('關卡 id 會做 URL 編碼', async () => {
    const { client, fetchMock } = clientWith(jsonResponse(200, runOk('')))
    await client.run('a b', 'x')
    expect(fetchMock.mock.calls[0][0]).toBe('/api/levels/a%20b/run')
  })

  it('run 回 503 JUDGE_BUSY 時讀 Retry-After header', async () => {
    const { client } = clientWith(jsonResponse(503, errorBody('JUDGE_BUSY'), { 'Retry-After': '2' }))
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: false, failure: { kind: 'server_busy', retryAfterSeconds: 2 } })
  })

  it('submit 回 503 JUDGE_UNAVAILABLE 為 judge_unavailable', async () => {
    const { client } = clientWith(jsonResponse(503, errorBody('JUDGE_UNAVAILABLE')))
    const result = await client.submit('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: false, failure: { kind: 'judge_unavailable' } })
  })

  it('run 回 413 為 payload_too_large', async () => {
    const { client } = clientWith(jsonResponse(413, errorBody('PAYLOAD_TOO_LARGE')))
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: false, failure: { kind: 'payload_too_large' } })
  })

  it('run 回 404 時為 network_error，不是 level_not_found', async () => {
    const { client } = clientWith(jsonResponse(404, errorBody('LEVEL_NOT_FOUND')))
    const result = await client.run('99-unknown', 'x')
    expect(result).toMatchObject({ ok: false, failure: { kind: 'network_error', status: 404, code: 'LEVEL_NOT_FOUND' } })
  })

  it('斷線（fetch reject）為 network_error，status 為 null', async () => {
    const { client } = clientWith(async () => {
      throw new TypeError('Failed to fetch')
    })
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({
      ok: false,
      failure: { kind: 'network_error', status: null, code: null, message: null },
    })
  })

  it('run 回 413 但 body 不是 JSON 也為 payload_too_large', async () => {
    const { client } = clientWith(new Response('Request Entity Too Large', { status: 413 }))
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: false, failure: { kind: 'payload_too_large' } })
  })

  it('請求被 abort 時歸為 network_error', async () => {
    const { client } = clientWith(async () => {
      throw new DOMException('aborted', 'AbortError')
    })
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toEqual({ ok: false, failure: { kind: 'network_error', status: null, code: null, message: null } })
  })

  it('200 但 body 不是 JSON 時為 network_error', async () => {
    const { client } = clientWith(new Response('not json', { status: 200 }))
    const result = await client.run('01-hello-ferris', 'x')
    expect(result).toMatchObject({ ok: false, failure: { kind: 'network_error' } })
  })

  it('開發模式下 4xx 以 console.error 印出 error.code 與 error.message', async () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const { client } = clientWith(jsonResponse(400, errorBody('INVALID_REQUEST', '缺少 code')))
    await client.run('01-hello-ferris', '')
    expect(spy).toHaveBeenCalled()
    const printed = spy.mock.calls.flat().join(' ')
    expect(printed).toContain('INVALID_REQUEST')
    expect(printed).toContain('缺少 code')
  })

  it('AbortSignal 會傳給 fetch', async () => {
    const { client, fetchMock } = clientWith(jsonResponse(200, runOk('')))
    const controller = new AbortController()
    await client.run('01-hello-ferris', 'x', controller.signal)
    const init = fetchMock.mock.calls[0][1] as RequestInit
    expect(init.signal).toBe(controller.signal)
  })

  it('health 回傳降級狀態', async () => {
    const body = { status: 'degraded', judge: { available: false, reason: 'runner_image_missing' } }
    const { client, fetchMock } = clientWith(jsonResponse(200, body))
    const result = await client.health()
    expect(result).toEqual({ ok: true, data: body })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/health')
  })
})

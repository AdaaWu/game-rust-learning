import type {
  HealthResponse,
  LevelDetail,
  LevelListResponse,
  RunResponse,
  SubmitResponse,
} from './types'

// ux.md §3.1 請求失敗四種
export type RequestFailure =
  | { kind: 'server_busy'; retryAfterSeconds: number }
  | { kind: 'judge_unavailable' }
  | { kind: 'payload_too_large' }
  | { kind: 'network_error'; status: number | null; code: string | null; message: string | null }

// GET /api/levels/:id 回 404 時另外區分，供「找不到這關」頁使用
export type LevelFailure = RequestFailure | { kind: 'level_not_found' }

export type ApiResult<T, F = RequestFailure> = { ok: true; data: T } | { ok: false; failure: F }

export const DEFAULT_RETRY_AFTER_SECONDS = 10

export interface ApiClientOptions {
  fetch?: typeof fetch
}

export interface ApiClient {
  listLevels(signal?: AbortSignal): Promise<ApiResult<LevelListResponse>>
  getLevel(id: string, signal?: AbortSignal): Promise<ApiResult<LevelDetail, LevelFailure>>
  run(id: string, code: string, signal?: AbortSignal): Promise<ApiResult<RunResponse>>
  submit(id: string, code: string, signal?: AbortSignal): Promise<ApiResult<SubmitResponse>>
  health(signal?: AbortSignal): Promise<ApiResult<HealthResponse>>
}

// Retry-After header 轉秒數，只接受正整數秒
export function parseRetryAfter(header: string | null): number {
  if (header === null) return DEFAULT_RETRY_AFTER_SECONDS
  const trimmed = header.trim()
  if (!/^\d+$/.test(trimmed)) return DEFAULT_RETRY_AFTER_SECONDS
  const seconds = Number(trimmed)
  return seconds > 0 ? seconds : DEFAULT_RETRY_AFTER_SECONDS
}

function readErrorBody(body: unknown): { code: string; message: string } | null {
  if (typeof body !== 'object' || body === null) return null
  const error = (body as { error?: unknown }).error
  if (typeof error !== 'object' || error === null) return null
  const { code, message } = error as { code?: unknown; message?: unknown }
  if (typeof code !== 'string' || typeof message !== 'string') return null
  return { code, message }
}

// 非 200 回應分類
export function classifyFailure(status: number, body: unknown, headers: Headers): RequestFailure {
  if (status === 413) return { kind: 'payload_too_large' }
  const error = readErrorBody(body)
  if (status === 503 && error?.code === 'JUDGE_BUSY') {
    return { kind: 'server_busy', retryAfterSeconds: parseRetryAfter(headers.get('Retry-After')) }
  }
  if (status === 503 && error?.code === 'JUDGE_UNAVAILABLE') return { kind: 'judge_unavailable' }
  return { kind: 'network_error', status, code: error?.code ?? null, message: error?.message ?? null }
}

const NETWORK_ERROR: RequestFailure = { kind: 'network_error', status: null, code: null, message: null }

async function readJson(response: Response): Promise<{ ok: true; value: unknown } | { ok: false }> {
  try {
    return { ok: true, value: await response.json() }
  } catch {
    return { ok: false }
  }
}

export function createApiClient(options: ApiClientOptions = {}): ApiClient {
  const doFetch: typeof fetch = (input, init) => (options.fetch ?? globalThis.fetch)(input, init)

  async function request<T>(url: string, init: RequestInit): Promise<{ result: ApiResult<T>; status: number | null }> {
    let response: Response
    try {
      response = await doFetch(url, init)
    } catch {
      return { result: { ok: false, failure: NETWORK_ERROR }, status: null }
    }
    if (response.status === 413) {
      return { result: { ok: false, failure: { kind: 'payload_too_large' } }, status: 413 }
    }
    const body = await readJson(response)
    if (response.status === 200) {
      if (!body.ok) return { result: { ok: false, failure: { kind: 'network_error', status: 200, code: null, message: null } }, status: 200 }
      return { result: { ok: true, data: body.value as T }, status: 200 }
    }
    const failure = classifyFailure(response.status, body.ok ? body.value : null, response.headers)
    if (import.meta.env.DEV && failure.kind === 'network_error' && response.status >= 400 && response.status < 500) {
      console.error(`[api] ${response.status} ${failure.code ?? ''} ${failure.message ?? ''}`)
    }
    return { result: { ok: false, failure }, status: response.status }
  }

  function get<T>(url: string, signal?: AbortSignal) {
    return request<T>(url, { method: 'GET', signal })
  }

  function post<T>(url: string, code: string, signal?: AbortSignal) {
    return request<T>(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code }),
      signal,
    })
  }

  const levelUrl = (id: string) => `/api/levels/${encodeURIComponent(id)}`

  return {
    listLevels: async (signal) => (await get<LevelListResponse>('/api/levels', signal)).result,
    getLevel: async (id, signal) => {
      const { result, status } = await get<LevelDetail>(levelUrl(id), signal)
      if (!result.ok && status === 404) return { ok: false, failure: { kind: 'level_not_found' } }
      return result
    },
    run: async (id, code, signal) => (await post<RunResponse>(`${levelUrl(id)}/run`, code, signal)).result,
    submit: async (id, code, signal) => (await post<SubmitResponse>(`${levelUrl(id)}/submit`, code, signal)).result,
    health: async (signal) => (await get<HealthResponse>('/api/health', signal)).result,
  }
}

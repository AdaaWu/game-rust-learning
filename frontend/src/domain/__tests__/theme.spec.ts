import { afterEach, describe, expect, it, vi } from 'vitest'
import { applyTheme, resolveTheme } from '../theme'

function mockPrefersDark(dark: boolean) {
  window.matchMedia = vi.fn((query: string) => ({
    matches: dark && query.includes('prefers-color-scheme: dark'),
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })) as unknown as typeof window.matchMedia
}

describe('F7 深淺色套用（ux §8.3）', () => {
  afterEach(() => {
    delete document.documentElement.dataset.theme
  })

  it('system 時依 prefers-color-scheme 決定', () => {
    expect(resolveTheme('system', true)).toBe('dark')
    expect(resolveTheme('system', false)).toBe('light')
  })

  it('指定 light／dark 時不看系統', () => {
    expect(resolveTheme('light', true)).toBe('light')
    expect(resolveTheme('dark', false)).toBe('dark')
  })

  it('applyTheme 在 <html> 設 data-theme', () => {
    mockPrefersDark(false)
    applyTheme('dark')
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    applyTheme('light')
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('applyTheme system 時依媒體查詢設實際值', () => {
    mockPrefersDark(true)
    applyTheme('system')
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    mockPrefersDark(false)
    applyTheme('system')
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('不設 data-style', () => {
    mockPrefersDark(false)
    applyTheme('light')
    expect(document.documentElement.hasAttribute('data-style')).toBe(false)
  })
})

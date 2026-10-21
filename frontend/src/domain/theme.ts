import type { Theme } from './progress'

export type AppliedTheme = 'light' | 'dark'

// system 時依 prefers-color-scheme 決定
export function resolveTheme(theme: Theme, prefersDark: boolean): AppliedTheme {
  if (theme === 'system') return prefersDark ? 'dark' : 'light'
  return theme
}

function systemPrefersDark(): boolean {
  return typeof window.matchMedia === 'function' && window.matchMedia('(prefers-color-scheme: dark)').matches
}

// 在 <html> 設 data-theme
export function applyTheme(theme: Theme): void {
  document.documentElement.setAttribute('data-theme', resolveTheme(theme, systemPrefersDark()))
}

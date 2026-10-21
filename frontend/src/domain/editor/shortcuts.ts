export type ShortcutAction = 'run' | 'submit'

export interface KeyLike {
  key: string
  ctrlKey: boolean
  metaKey: boolean
  shiftKey: boolean
  altKey: boolean
}

export function isMacPlatform(platform: string): boolean {
  return /mac|iphone|ipad|ipod/i.test(platform)
}

// ux.md §7.2：macOS 用 Cmd，其他用 Ctrl
export function matchShortcut(event: KeyLike, isMac: boolean): ShortcutAction | null {
  if (event.key !== 'Enter' || event.altKey) return null
  const primary = isMac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey
  if (!primary) return null
  return event.shiftKey ? 'submit' : 'run'
}

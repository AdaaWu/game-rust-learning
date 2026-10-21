import { describe, expect, it } from 'vitest'
import { isMacPlatform, matchShortcut } from '../shortcuts'
import type { KeyLike } from '../shortcuts'

function key(k: Partial<KeyLike>): KeyLike {
  return { key: 'Enter', ctrlKey: false, metaKey: false, shiftKey: false, altKey: false, ...k }
}

describe('F6 快捷鍵', () => {
  it.each(['MacIntel', 'macOS', 'iPhone'])('%s 視為 macOS', (p) => {
    expect(isMacPlatform(p)).toBe(true)
  })

  it.each(['Win32', 'Linux x86_64'])('%s 不是 macOS', (p) => {
    expect(isMacPlatform(p)).toBe(false)
  })

  it('Windows：Ctrl+Enter 為 Run', () => {
    expect(matchShortcut(key({ ctrlKey: true }), false)).toBe('run')
  })

  it('Windows：Ctrl+Shift+Enter 為 Submit', () => {
    expect(matchShortcut(key({ ctrlKey: true, shiftKey: true }), false)).toBe('submit')
  })

  it('macOS：Cmd+Enter 為 Run、Cmd+Shift+Enter 為 Submit', () => {
    expect(matchShortcut(key({ metaKey: true }), true)).toBe('run')
    expect(matchShortcut(key({ metaKey: true, shiftKey: true }), true)).toBe('submit')
  })

  it('macOS 上 Ctrl+Enter 不觸發', () => {
    expect(matchShortcut(key({ ctrlKey: true }), true)).toBeNull()
  })

  it('單獨 Enter 不觸發', () => {
    expect(matchShortcut(key({}), false)).toBeNull()
  })

  it('Ctrl+Shift+R 不綁定', () => {
    expect(matchShortcut(key({ key: 'R', ctrlKey: true, shiftKey: true }), false)).toBeNull()
  })

  it('加了 Alt 不觸發', () => {
    expect(matchShortcut(key({ ctrlKey: true, altKey: true }), false)).toBeNull()
  })
})

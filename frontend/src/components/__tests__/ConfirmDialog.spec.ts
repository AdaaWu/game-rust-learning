import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfirmDialog from '../ConfirmDialog.vue'

const MSG = '重設會把程式碼改回起始版本，目前的修改會消失，確定嗎？'

function mountDialog() {
  return mount(ConfirmDialog, { props: { message: MSG }, attachTo: document.body })
}

describe('F7 ConfirmDialog 確認框', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('是 modal 對話框並顯示訊息', () => {
    const w = mountDialog()
    const dialog = w.get('[data-testid="confirm-dialog"]')
    expect(dialog.attributes('role')).toBe('dialog')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(w.get('[data-testid="confirm-message"]').text()).toBe(MSG)
  })

  it('有確認與取消兩顆按鈕', () => {
    const w = mountDialog()
    expect(w.get('[data-testid="confirm-ok"]').text()).toBe('確定')
    expect(w.get('[data-testid="confirm-cancel"]').text()).toBe('取消')
  })

  it('按確認發出 confirm', async () => {
    const w = mountDialog()
    await w.get('[data-testid="confirm-ok"]').trigger('click')
    expect(w.emitted('confirm')).toHaveLength(1)
    expect(w.emitted('cancel')).toBeUndefined()
  })

  it('按取消發出 cancel', async () => {
    const w = mountDialog()
    await w.get('[data-testid="confirm-cancel"]').trigger('click')
    expect(w.emitted('cancel')).toHaveLength(1)
    expect(w.emitted('confirm')).toBeUndefined()
  })

  it('Esc 視為取消', async () => {
    const w = mountDialog()
    await w.get('[data-testid="confirm-dialog"]').trigger('keydown', { key: 'Escape' })
    expect(w.emitted('cancel')).toHaveLength(1)
  })

  it('焦點鎖在對話框內：Tab 與 Shift+Tab 循環', async () => {
    const w = mountDialog()
    const ok = w.get('[data-testid="confirm-ok"]')
    const cancel = w.get('[data-testid="confirm-cancel"]')
    const focusables = w.findAll('button')
    const first = focusables[0]
    const last = focusables[focusables.length - 1]
    expect([ok.element, cancel.element]).toContain(first.element)
    ;(last.element as HTMLButtonElement).focus()
    await last.trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(first.element)
    await first.trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(last.element)
  })

  it('開啟時焦點移到對話框內', async () => {
    const w = mountDialog()
    await w.vm.$nextTick()
    expect(w.get('[data-testid="confirm-dialog"]').element.contains(document.activeElement)).toBe(true)
  })
})

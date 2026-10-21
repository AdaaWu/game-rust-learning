import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import LevelCompleteDialog from '../LevelCompleteDialog.vue'
import type { StarSet } from '../../domain/progress'

function mountDialog(earned: StarSet) {
  return mount(LevelCompleteDialog, {
    props: { levelNumber: '1-2', title: '黏住的腳', earned },
    attachTo: document.body,
  })
}

describe('F7 LevelCompleteDialog 通關結算', () => {
  it('是 modal 對話框並顯示關卡編號與名稱', () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    const dialog = w.get('[role="dialog"]')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(dialog.text()).toContain('1-2')
    expect(dialog.text()).toContain('黏住的腳')
    w.unmount()
  })

  it('每顆星旁寫達成條件與是否達成', () => {
    const w = mountDialog({ pass: true, noHint: false, firstTry: true })
    const rows = w.findAll('[data-testid="star-row"]')
    expect(rows.map((r) => r.attributes('data-star'))).toEqual(['pass', 'noHint', 'firstTry'])
    expect(rows.map((r) => r.attributes('data-earned'))).toEqual(['true', 'false', 'true'])
    expect(rows[0].text()).toContain('通過')
    expect(rows[1].text()).toContain('沒看提示')
    expect(rows[2].text()).toContain('一次就過')
    w.unmount()
  })

  it('未達成的「沒看提示」寫怎麼拿到', () => {
    const w = mountDialog({ pass: true, noHint: false, firstTry: true })
    expect(w.get('[data-testid="star-row"][data-star="noHint"]').text()).toContain('不看提示再挑戰一次就能拿到')
    w.unmount()
  })

  it('未達成的「一次就過」也寫怎麼拿到', () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: false })
    expect(w.get('[data-testid="star-row"][data-star="firstTry"] [data-testid="star-tip"]').text()).not.toBe('')
    w.unmount()
  })

  it('已達成的星沒有拿法提示', () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    expect(w.findAll('[data-testid="star-row"]')).toHaveLength(3)
    expect(w.find('[data-testid="star-tip"]').exists()).toBe(false)
    w.unmount()
  })

  it('亮星實心、暗星空心（不只靠顏色）', () => {
    const w = mountDialog({ pass: true, noHint: false, firstTry: false })
    const icons = w.findAll('[data-testid="star-icon"]').map((s) => s.text())
    expect(icons).toEqual(['★', '☆', '☆'])
    w.unmount()
  })

  it('三顆按鈕：下一關（主要）、再玩一次、回地圖', async () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    const buttons = w.findAll('button')
    expect(buttons.map((b) => b.text())).toEqual(['下一關', '再玩一次', '回地圖'])
    expect(buttons[0].classes()).toContain('is-primary')
    await buttons[0].trigger('click')
    await buttons[1].trigger('click')
    await buttons[2].trigger('click')
    expect(w.emitted('next')).toHaveLength(1)
    expect(w.emitted('replay')).toHaveLength(1)
    expect(w.emitted('map')).toHaveLength(1)
    w.unmount()
  })

  it('Esc 關閉', async () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    await w.get('[role="dialog"]').trigger('keydown', { key: 'Escape' })
    expect(w.emitted('close')).toHaveLength(1)
    w.unmount()
  })

  it('焦點鎖在對話框內：最後一顆按 Tab 回到第一顆，第一顆按 Shift+Tab 到最後一顆', async () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    const buttons = w.findAll('button')
    ;(buttons[2].element as HTMLButtonElement).focus()
    await buttons[2].trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(buttons[0].element)
    await buttons[0].trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(buttons[2].element)
    w.unmount()
  })

  it('開啟時焦點移到對話框內', async () => {
    const w = mountDialog({ pass: true, noHint: true, firstTry: true })
    await w.vm.$nextTick()
    expect(w.get('[role="dialog"]').element.contains(document.activeElement)).toBe(true)
    w.unmount()
  })
})

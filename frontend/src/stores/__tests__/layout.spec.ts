import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLayoutStore } from '../layout'

describe('F7 版面記憶 store（平板抽屜、手機分頁）', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('平板：第一次進關時抽屜自動展開', () => {
    expect(useLayoutStore().claimDrawerAutoOpen('01')).toBe(true)
  })

  it('平板：同一關之後不再自動展開', () => {
    const s = useLayoutStore()
    s.claimDrawerAutoOpen('01')
    expect(s.claimDrawerAutoOpen('01')).toBe(false)
  })

  it('平板：不同關卡各自記錄', () => {
    const s = useLayoutStore()
    s.claimDrawerAutoOpen('01')
    expect(s.claimDrawerAutoOpen('02')).toBe(true)
  })

  it('手機：首次進關預設「故事」分頁，之後為「程式」', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    expect(s.mobileTab).toBe('story')
    s.setMobileTab('result')
    s.enterLevelMobile('01')
    expect(s.mobileTab).toBe('code')
  })

  it('手機：另一關首次進入仍是「故事」', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    s.enterLevelMobile('02')
    expect(s.mobileTab).toBe('story')
  })

  it('Run／Submit 送出後自動切到「結果」分頁', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    s.notifyRequestStarted()
    expect(s.mobileTab).toBe('result')
  })

  it('結果回來時玩家不在「結果」分頁：顯示小圓點', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    s.notifyRequestStarted()
    s.setMobileTab('code')
    s.notifyResult()
    expect(s.resultDot).toBe(true)
  })

  it('結果回來時玩家就在「結果」分頁：不顯示小圓點', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    s.notifyRequestStarted()
    s.notifyResult()
    expect(s.resultDot).toBe(false)
  })

  it('切到「結果」分頁時清除小圓點', () => {
    const s = useLayoutStore()
    s.enterLevelMobile('01')
    s.setMobileTab('code')
    s.notifyResult()
    s.setMobileTab('result')
    expect(s.resultDot).toBe(false)
  })

  it('只存記憶體：不寫 localStorage，新 store（重新整理）後重置', () => {
    const s = useLayoutStore()
    s.claimDrawerAutoOpen('01')
    s.enterLevelMobile('01')
    expect(localStorage.length).toBe(0)
    setActivePinia(createPinia())
    const fresh = useLayoutStore()
    expect(fresh.claimDrawerAutoOpen('01')).toBe(true)
    fresh.enterLevelMobile('01')
    expect(fresh.mobileTab).toBe('story')
  })
})

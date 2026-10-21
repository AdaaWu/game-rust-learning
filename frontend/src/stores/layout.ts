import { defineStore } from 'pinia'

export type MobileTab = 'story' | 'code' | 'result'

// 平板抽屜與手機分頁的記憶，只存記憶體
export const useLayoutStore = defineStore('layout', {
  state: () => ({
    mobileTab: 'code' as MobileTab,
    // 結果分頁有新內容且玩家不在該分頁
    resultDot: false,
    drawerOpenedLevels: new Set<string>(),
    mobileVisitedLevels: new Set<string>(),
  }),
  actions: {
    // 本次頁面載入後第一次進該關回 true，之後回 false
    claimDrawerAutoOpen(levelId: string): boolean {
      if (this.drawerOpenedLevels.has(levelId)) return false
      this.drawerOpenedLevels.add(levelId)
      return true
    },
    // 進關時設定手機預設分頁：首次為故事，之後為程式
    enterLevelMobile(levelId: string) {
      this.resultDot = false
      if (this.mobileVisitedLevels.has(levelId)) {
        this.mobileTab = 'code'
        return
      }
      this.mobileVisitedLevels.add(levelId)
      this.mobileTab = 'story'
    },
    setMobileTab(tab: MobileTab) {
      this.mobileTab = tab
      if (tab === 'result') this.resultDot = false
    },
    // Run／Submit 送出時自動切到結果
    notifyRequestStarted() {
      this.setMobileTab('result')
    },
    // 結果回來
    notifyResult() {
      this.resultDot = this.mobileTab !== 'result'
    },
  },
})

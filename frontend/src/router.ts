import { createRouter } from 'vue-router'
import type { RouterHistory } from 'vue-router'
import { isUnlocked } from './domain/progress'
import { useLevelStore } from './stores/level'
import { useProgressStore } from './stores/progress'
import { useToastStore } from './stores/toast'
import LevelMapView from './views/LevelMapView.vue'

export function createAppRouter(history: RouterHistory) {
  const router = createRouter({
    history,
    routes: [
      { path: '/', name: 'map', component: LevelMapView },
      // 關卡頁含 Monaco，延遲載入
      { path: '/level/:levelId', name: 'level', component: () => import('./views/LevelView.vue') },
    ],
  })

  let progressLoaded = false
  router.beforeEach(async (to) => {
    const progress = useProgressStore()
    if (!progressLoaded) {
      progress.load()
      progressLoaded = true
    }
    if (to.name !== 'level') return true
    const levelId = String(to.params.levelId)
    const levelStore = useLevelStore()
    const list = await levelStore.fetchList()
    // 列表拿不到或不在列表中：放行，由關卡頁處理
    if (!list.ok || !list.data.some((l) => l.id === levelId)) return true
    if (isUnlocked(levelId, list.data, progress.progress)) return true
    useToastStore().show('這關還沒解鎖')
    return { name: 'map' }
  })

  return router
}

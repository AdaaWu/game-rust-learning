import type { LevelDetail } from '../api/types'

// 時間軸 fixture：全部取自 rustc 1.99.1 --edition 2024 -A dead_code -F unsafe_code 實際執行輸出
// 小幫手為 architecture.md §3.11 參考實作；行號是接上小幫手後的原始碼行號

// 33 starter：Run 逾時（死鎖後被 kill），分身 1、分身 0 互等窄橋
export const tl33Starter = "@tl|recv_wait|58|main|回報\n@tl|start|36|分身 2|\n@tl|send|44|分身 2|回報\n@tl|start|36|分身 1|\n@tl|lock_wait|39|分身 1|窄橋 1\n@tl|lock_got|39|分身 1|窄橋 1\n分身 2 回報完畢，讓出 0 座窄橋\n@tl|end||分身 2|\n@tl|start|36|分身 0|\n分身 1 站上窄橋 1\n@tl|recv|58|main|回報\n@tl|lock_wait|39|分身 0|窄橋 0\n@tl|lock_got|39|分身 0|窄橋 0\n分身 0 站上窄橋 0\n@tl|recv_wait|58|main|回報\n@tl|lock_wait|39|分身 1|窄橋 0\n@tl|lock_wait|39|分身 0|窄橋 1\n"

// 33 solution：Run ok，3 則回報、channel 最後關閉
export const tl33Solution = "@tl|start|36|分身 2|\n@tl|recv_wait|63|main|回報\n@tl|send|47|分身 2|回報\n@tl|recv|63|main|回報\n@tl|recv_wait|63|main|回報\n分身 2 回報完畢，讓出 0 座窄橋\n@tl|end||分身 2|\n@tl|start|36|分身 0|\n@tl|lock_wait|42|分身 0|窄橋 0\n@tl|lock_got|42|分身 0|窄橋 0\n分身 0 站上窄橋 0\n@tl|start|36|分身 1|\n@tl|lock_wait|42|分身 1|窄橋 0\n@tl|lock_wait|42|分身 0|窄橋 1\n@tl|lock_got|42|分身 0|窄橋 1\n分身 0 站上窄橋 1\n@tl|send|47|分身 0|回報\n@tl|recv|63|main|回報\n@tl|recv_wait|63|main|回報\n分身 0 回報完畢，讓出 2 座窄橋\n@tl|lock_free||分身 0|窄橋 0\n@tl|lock_free||分身 0|窄橋 1\n@tl|end||分身 0|\n@tl|lock_got|42|分身 1|窄橋 0\n分身 1 站上窄橋 0\n@tl|lock_wait|42|分身 1|窄橋 1\n@tl|lock_got|42|分身 1|窄橋 1\n分身 1 站上窄橋 1\n@tl|send|47|分身 1|回報\n@tl|recv|63|main|回報\n@tl|recv_wait|63|main|回報\n分身 1 回報完畢，讓出 2 座窄橋\n@tl|lock_free||分身 1|窄橋 0\n@tl|lock_free||分身 1|窄橋 1\n@tl|end||分身 1|\n@tl|recv_closed|63|main|回報\n指揮官收到 17 道指令，大遷徙出發！\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nPICK\nMOVE RIGHT\nMOVE UP\nMOVE UP\nMOVE RIGHT\nPICK\nMOVE RIGHT\nOPEN DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nPICK\nMOVE DOWN\n"

// 33 修好鎖順序但沒 drop(tx)：Run 逾時，main 停在等回報、分身都已結束
export const tl33NoDrop = "@tl|start|36|分身 0|\n@tl|lock_wait|41|分身 0|窄橋 0\n@tl|lock_got|41|分身 0|窄橋 0\n分身 0 站上窄橋 0\n@tl|recv_wait|60|main|回報\n@tl|start|36|分身 1|\n@tl|lock_wait|41|分身 1|窄橋 0\n@tl|start|36|分身 2|\n@tl|send|46|分身 2|回報\n分身 2 回報完畢，讓出 0 座窄橋\n@tl|end||分身 2|\n@tl|recv|60|main|回報\n@tl|recv_wait|60|main|回報\n@tl|lock_wait|41|分身 0|窄橋 1\n@tl|lock_got|41|分身 0|窄橋 1\n分身 0 站上窄橋 1\n@tl|send|46|分身 0|回報\n分身 0 回報完畢，讓出 2 座窄橋\n@tl|lock_free||分身 0|窄橋 0\n@tl|lock_free||分身 0|窄橋 1\n@tl|end||分身 0|\n@tl|recv|60|main|回報\n@tl|recv_wait|60|main|回報\n@tl|lock_got|41|分身 1|窄橋 0\n分身 1 站上窄橋 0\n@tl|lock_wait|41|分身 1|窄橋 1\n@tl|lock_got|41|分身 1|窄橋 1\n分身 1 站上窄橋 1\n@tl|send|46|分身 1|回報\n分身 1 回報完畢，讓出 2 座窄橋\n@tl|lock_free||分身 1|窄橋 0\n@tl|lock_free||分身 1|窄橋 1\n@tl|end||分身 1|\n@tl|recv|60|main|回報\n@tl|recv_wait|60|main|回報\n"

// 31 starter：Run ok，三隻螃蟹都讀到 0，小旗子在放開計數器之後
export const tl31Starter = "@tl|start|18|螃蟹 0|\n@tl|lock_wait|6|螃蟹 0|計數器\n@tl|lock_got|6|螃蟹 0|計數器\n@tl|lock_free||螃蟹 0|計數器\n@tl|start|18|螃蟹 1|\n@tl|note|7|螃蟹 0|讀到 0\n@tl|end||螃蟹 0|\n@tl|start|18|螃蟹 2|\n@tl|lock_wait|6|螃蟹 2|計數器\n@tl|lock_got|6|螃蟹 2|計數器\n@tl|lock_free||螃蟹 2|計數器\n@tl|note|7|螃蟹 2|讀到 0\n@tl|end||螃蟹 2|\n@tl|lock_wait|6|螃蟹 1|計數器\n@tl|lock_got|6|螃蟹 1|計數器\n@tl|lock_free||螃蟹 1|計數器\n@tl|note|7|螃蟹 1|讀到 0\n@tl|end||螃蟹 1|\n@tl|lock_wait|26|main|計數器\n@tl|lock_got|26|main|計數器\n@tl|lock_free||main|計數器\n總共 0 顆寶石\n"

// 31 solution：Run ok，小旗子在持有區段內
export const tl31Solution = "@tl|start|18|螃蟹 0|\n@tl|lock_wait|6|螃蟹 0|計數器\n@tl|lock_got|6|螃蟹 0|計數器\n@tl|note|7|螃蟹 0|讀到 0\n@tl|lock_free||螃蟹 0|計數器\n@tl|end||螃蟹 0|\n@tl|start|18|螃蟹 2|\n@tl|lock_wait|6|螃蟹 2|計數器\n@tl|lock_got|6|螃蟹 2|計數器\n@tl|note|7|螃蟹 2|讀到 3\n@tl|lock_free||螃蟹 2|計數器\n@tl|end||螃蟹 2|\n@tl|start|18|螃蟹 1|\n@tl|lock_wait|6|螃蟹 1|計數器\n@tl|lock_got|6|螃蟹 1|計數器\n@tl|note|7|螃蟹 1|讀到 10\n@tl|lock_free||螃蟹 1|計數器\n@tl|end||螃蟹 1|\n@tl|lock_wait|26|main|計數器\n@tl|lock_got|26|main|計數器\n@tl|lock_free||main|計數器\n總共 15 顆寶石\n"

// 30 solution（closure 開頭加 lane）：Run ok
export const tl30Solution = "@tl|start|9|點名 0|\n@tl|end||點名 0|\n@tl|start|9|點名 1|\n@tl|end||點名 1|\n@tl|start|9|點名 2|\n@tl|end||點名 2|\n分身 0 到！\n分身 1 到！\n分身 2 到！\n@tl|start|33|分身 0|\n@tl|end||分身 0|\n@tl|start|33|分身 1|\n@tl|end||分身 1|\n@tl|start|33|分身 2|\n@tl|end||分身 2|\n分身 0 抵達珊瑚灘\n分身 1 抵達海龜灣\n分身 2 抵達珊瑚灘\n"

// 同一執行緒重複鎖同一把 Mutex：Run 逾時
export const tlSelfLock = "@tl|start|8|寄居蟹|\n@tl|lock_wait|9|寄居蟹|貝殼\n@tl|lock_got|9|寄居蟹|貝殼\n@tl|lock_wait|10|寄居蟹|貝殼\n"

// 三隻螃蟹各握一塊礁石等下一塊：Run 逾時，三條循環
export const tlThreeCycle = "@tl|start|11|螃蟹 0|\n@tl|start|11|螃蟹 2|\n@tl|start|11|螃蟹 1|\n@tl|lock_wait|12|螃蟹 0|礁石 0\n@tl|lock_wait|12|螃蟹 2|礁石 2\n@tl|lock_got|12|螃蟹 2|礁石 2\n@tl|lock_got|12|螃蟹 0|礁石 0\n@tl|lock_wait|12|螃蟹 1|礁石 1\n@tl|lock_got|12|螃蟹 1|礁石 1\n@tl|lock_wait|14|螃蟹 2|礁石 0\n@tl|lock_wait|14|螃蟹 1|礁石 2\n@tl|lock_wait|14|螃蟹 0|礁石 1\n"

// main 沒 join 就結束：Run ok，兩條具名泳道沒有 end
export const tlUnjoined = "@tl|start|6|快腳蟹|\n@tl|end||快腳蟹|\n@tl|start|11|慢吞蟹 0|\n@tl|start|11|慢吞蟹 1|\n指揮官先走了\n"

// main 握著寶箱時 panic：Run runtime_panic，守衛蟹沒有 end
export const tlPanic = "@tl|start|8|守衛蟹|\n@tl|lock_wait|12|main|寶箱\n@tl|lock_got|12|main|寶箱\n寶箱裡有 0 樣\n@tl|lock_free||main|寶箱\n"

// 上一個程式的 stderr
export const tlPanicStderr = "\nthread 'main' (11182987) panicked at full_p_panic.rs:14:22:\nindex out of bounds: the len is 0 but the index is 0\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"

// rustc --test 執行時 stack overflow abort（exit 134）的 stderr
export const stackOverflowStderr = "\nthread 'zz_deep_echo' (11184663) has overflowed its stack\nfatal runtime error: stack overflow, aborting\n"

// 等待鏈不成環：分身 A 握窄橋 0 等回報、分身 B 等窄橋 0（BDD 分析師實際執行產生，逾時）
export const tlWaitChain = "@tl|start|11|分身 A|\n@tl|lock_wait|12|分身 A|窄橋 0\n@tl|lock_got|12|分身 A|窄橋 0\n@tl|recv_wait|13|分身 A|回報\n@tl|start|18|分身 B|\n@tl|lock_wait|19|分身 B|窄橋 0\n"

// visual = timeline 的關卡（33 關縮減版；starter 只用來比對是否修改）
export function timelineLevel(overrides: Partial<LevelDetail> = {}): LevelDetail {
  return {
    id: '01-hello-ferris',
    chapter: 1,
    order: 1,
    title: '大遷徙',
    kind: 'debug',
    visual: 'timeline',
    story: '三隻分身各站上窄橋',
    concept: '## 死鎖',
    hints: [],
    starter: 'fn main() {}\n',
    grid: null,
    memory_snapshots: null,
    ...overrides,
  }
}

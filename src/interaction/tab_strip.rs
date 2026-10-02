//! 页签栏的纯逻辑(不含任何 iced 类型):溢出窗口计算与拖拽确认阈值。
//! 从 dozer-app 搬来,供多个项目的页签栏共用;视觉外壳与交互接线见
//! `interaction::tabs::tab_core`。

/// 给定各 tab 宽、tab 间距、可视宽、当前 first,算出实际渲染窗口:
/// (钳制后的 first, 可见区间的独占结束下标)。
/// - 全部 tab 能放下(总宽<=avail) → first=0, visible_end=n(全可见,无溢出)。
/// - 溢出 → 先按原算法算 max_first(从右往左累加,找最大窗口起点使尾部放得下),
///   钳 first 到 [0, max_first];再从钳后的 first 往右累加,算出这一屏实际能
///   放下几个(`visible_end`)——原算法只钳 first,不知道"从 first 起到底能看见
///   几个",全靠调用方外层 `.clip(true)` 视觉裁切,拿不到索引,这次要靠这个
///   新窗口的可见区间构建"隐藏了哪些 tab"的列表,必须补上这个正向累加。
pub struct TabOverflow {
    pub first: usize,
    pub visible_end: usize,
}

impl TabOverflow {
    pub fn hidden_before(&self) -> std::ops::Range<usize> {
        0..self.first
    }

    pub fn hidden_after(&self, len: usize) -> std::ops::Range<usize> {
        self.visible_end..len
    }

    pub fn has_overflow(&self, len: usize) -> bool {
        self.first > 0 || self.visible_end < len
    }
}

pub fn tab_window(widths: &[f32], gap: f32, avail: f32, first: usize) -> TabOverflow {
    let n = widths.len();
    if n == 0 {
        return TabOverflow {
            first: 0,
            visible_end: 0,
        };
    }
    let total: f32 = widths.iter().sum::<f32>() + gap * (n.saturating_sub(1)) as f32;
    if total <= avail {
        return TabOverflow {
            first: 0,
            visible_end: n,
        };
    }
    // 求 max_first：从右往左累加，找最大的窗口起点使 tails 放得下。
    let mut max_first = n - 1;
    let mut acc = 0.0;
    for i in (0..n).rev() {
        let w = widths[i] + if i < n - 1 { gap } else { 0.0 };
        if acc + w <= avail {
            acc += w;
            max_first = i;
        } else {
            break;
        }
    }
    let clamped = first.min(max_first);
    // 从钳后的 first 往右累加，算这一屏实际放得下几个。
    // 至少放 `first` 这一个:单个 tab 比可用宽度还宽(超长文件名)时,累加
    // 一个都放不下,`visible_end == first` 会让整条 tab 栏空白;保底放一个,
    // 由外层 `.clip(true)` 截出它的前半段(标题部分可见)。
    let mut visible_end = clamped + 1;
    let mut fwd = 0.0;
    for (i, w) in widths.iter().enumerate().skip(clamped) {
        let w = *w + if i > clamped { gap } else { 0.0 };
        if i == clamped || fwd + w <= avail {
            fwd += w;
            visible_end = i + 1;
        } else {
            break;
        }
    }
    TabOverflow {
        first: clamped,
        visible_end,
    }
}

/// 选中某个 tab(`target`)后:若它已经在当前窗口可见区间内,`first` 原样
/// 不变(避免"点已可见的 tab 也跟着跳一下"的抖动);若它当前隐藏(在窗口外),
/// 把候选 first 设为 `target` 本身,交给 `tab_window` 重新钳出一个包含它的
/// 窗口——这就是"自动滚动带入可见区"的全部逻辑,没有新算法,只是换个候选值
/// 重跑一次既有的钳制。
pub fn tab_window_reveal(
    widths: &[f32],
    gap: f32,
    avail: f32,
    first: usize,
    target: usize,
) -> usize {
    let current = tab_window(widths, gap, avail, first);
    if target >= current.first && target < current.visible_end {
        current.first
    } else {
        tab_window(widths, gap, avail, target).first
    }
}

/// 页签拖拽确认阈值：按下瞬间到当前光标的位移必须越过这个半径才算"确认是
/// 一次拖拽换位"，见 [`tab_drag_past_threshold`]。
pub const TAB_DRAG_CONFIRM_THRESHOLD_PX: f32 = 4.0;

/// `press_pos` 到 `cursor` 的位移是否已越过 [`TAB_DRAG_CONFIRM_THRESHOLD_PX`]。
/// 页签(4px 间距)排得很紧:触控板等高灵敏输入下,单击落点到抬起之间的
/// 亚像素抖动偶尔会越界到邻居页签的命中框,若不设阈值就会触发一次肉眼
/// 不可见的"拖拽",把正被按住那个页签的 hover 光效错挪到邻居页签上——
/// 观感上就是两个页签同时像被选中(2026-09-04 用户反馈)。真正的拖拽
/// (持续位移必然越界)不受影响。
pub fn tab_drag_past_threshold(press_pos: (f32, f32), cursor: (f32, f32)) -> bool {
    let dx = cursor.0 - press_pos.0;
    let dy = cursor.1 - press_pos.1;
    dx * dx + dy * dy > TAB_DRAG_CONFIRM_THRESHOLD_PX * TAB_DRAG_CONFIRM_THRESHOLD_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_window_no_overflow_all_visible() {
        let w = tab_window(&[50.0, 50.0, 50.0], 4.0, 500.0, 0);
        assert_eq!((w.first, w.visible_end), (0, 3));
        assert!(!w.has_overflow(3));
    }

    #[test]
    fn tab_window_empty_is_empty() {
        let w = tab_window(&[], 4.0, 500.0, 7);
        assert_eq!((w.first, w.visible_end), (0, 0));
        assert!(!w.has_overflow(0));
    }

    #[test]
    fn tab_window_overflow_clamps_and_computes_visible_end() {
        let widths = [100.0; 5];
        let w = tab_window(&widths, 0.0, 250.0, 0);
        assert_eq!((w.first, w.visible_end), (0, 2));
        assert!(w.has_overflow(5));
        assert_eq!(w.hidden_before(), 0..0);
        assert_eq!(w.hidden_after(5), 2..5);

        // 请求的 first 越界 → 钳到 max_first(=3),此时尾部 3 个恰好全可见。
        let w = tab_window(&widths, 0.0, 250.0, 99);
        assert_eq!((w.first, w.visible_end), (3, 5));
        assert_eq!(w.hidden_before(), 0..3);
        assert_eq!(w.hidden_after(5), 5..5);

        let w = tab_window(&widths, 0.0, 250.0, 1);
        assert_eq!((w.first, w.visible_end), (1, 3));
        assert_eq!(w.hidden_before(), 0..1);
        assert_eq!(w.hidden_after(5), 3..5);
    }

    /// 单个 tab 比可用宽度还宽(超长文件名)时,窗口仍要放出它(部分可见),
    /// 而不是 `visible_end == first` 导致整条 tab 栏空白。
    #[test]
    fn tab_window_oversized_single_tab_still_visible() {
        let w = tab_window(&[900.0], 4.0, 300.0, 0);
        assert_eq!((w.first, w.visible_end), (0, 1));
        // 多 tab 且首个超宽:仍至少放出 first 这一个。
        let w = tab_window(&[900.0, 80.0], 4.0, 300.0, 0);
        assert_eq!((w.first, w.visible_end), (0, 1));
    }

    #[test]
    fn tab_window_reveal_keeps_visible_tab_still_no_jump() {
        let widths = [100.0; 5];
        // first=1 时可见区间是 [1,3):选中已经可见的 tab 1,first 不应该变。
        assert_eq!(tab_window_reveal(&widths, 0.0, 250.0, 1, 1), 1);
    }

    #[test]
    fn tab_window_reveal_scrolls_hidden_tab_into_view() {
        let widths = [100.0; 5];
        // first=0 时可见区间是 [0,2):选中隐藏在右侧的 tab 4,应重新钳出
        // 一个包含它的窗口。
        let new_first = tab_window_reveal(&widths, 0.0, 250.0, 0, 4);
        let w = tab_window(&widths, 0.0, 250.0, new_first);
        assert!((w.first..w.visible_end).contains(&4));
    }

    /// 2026-09-04 用户反馈"agent tab 偶尔两个同时看起来被选中"的根因防回归:
    /// 单击 tab 时按下瞬间到抬起前的亚像素抖动不该被当成一次拖拽换位。
    #[test]
    fn tab_drag_past_threshold_false_when_cursor_has_not_moved() {
        assert!(!tab_drag_past_threshold((100.0, 100.0), (100.0, 100.0)));
        assert!(!tab_drag_past_threshold((100.0, 100.0), (101.0, 100.0)));
    }

    /// 恰好等于阈值(平方比较是 `>` 不是 `>=`)不算越过,严格大于才算。
    #[test]
    fn tab_drag_past_threshold_false_when_exactly_at_threshold() {
        assert!(!tab_drag_past_threshold(
            (0.0, 0.0),
            (TAB_DRAG_CONFIRM_THRESHOLD_PX, 0.0)
        ));
    }

    /// 光标越过阈值(任意方向,这里用纯 x 位移)判定为真的拖拽。
    #[test]
    fn tab_drag_past_threshold_true_once_cursor_moves_past_it() {
        assert!(tab_drag_past_threshold(
            (0.0, 0.0),
            (TAB_DRAG_CONFIRM_THRESHOLD_PX + 1.0, 0.0)
        ));
        // 纵向位移同样算。
        assert!(tab_drag_past_threshold(
            (0.0, 0.0),
            (0.0, TAB_DRAG_CONFIRM_THRESHOLD_PX + 1.0)
        ));
    }
}

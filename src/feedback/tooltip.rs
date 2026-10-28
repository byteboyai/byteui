//! 页签 / 顶栏的标题全称气泡(原 Dozer `app/view.rs::controlled_tooltip`,Digger 的
//! `agent_tabs.rs`/`tab_chrome.rs` 各抄过一份)。

use crate::interaction::icons;
use iced_widget::core::Element;
use iced_widget::tooltip::{Position, Tooltip};
use iced_widget::{container, text};

/// 悬停多久才弹标题全称。Dozer(`dozer_panel_browser::TAB_TOOLTIP_DELAY`)与 Digger
/// (`agent::TOOLTIP_DELAY`)原先各写一份,值同为 2 秒。
pub const TOOLTIP_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

/// 页签 / 顶栏的标题全称气泡:`content` **始终**包进 `Tooltip`,2 秒延迟交给 iced 自带的
/// `Tooltip::delay`(它自己计时并在到期时请求重绘)。
///
/// 原先的做法是"悬停满 2s 才把 `content` 包进 `Tooltip`、离开就返回裸 `content`",理由是
/// 当时认为 `Tooltip` 没有延迟开关——但 iced 0.14 有。那种做法会让控件树结构随 `show`
/// 翻转,里面 `MouseArea` 的状态被重置成"未悬停":翻转那一帧如果光标已经离开,`on_exit`
/// 永远不会触发,悬停底色与 × 就卡住不消失(用户反馈"tab 经常鼠标离开后 hover 色还留着")。
/// 所以结构恒定,`show` 只保留为参数以免改动所有调用方(见测试)。`position` 由调用方按
/// 页签位置定(顶栏页签用 `Bottom`、底部面板页签用 `Top`,免得气泡出屏)。`label` 收
/// `String`(拥有所有权),使气泡 `Element` 寿命不受调用方局部借用牵制。
pub fn controlled_tooltip<'a, M, R>(
    content: Element<'a, M, iced_widget::Theme, R>,
    label: String,
    position: Position,
    _show: bool,
) -> Element<'a, M, iced_widget::Theme, R>
where
    M: Clone + 'a,
    R: iced_widget::core::text::Renderer + 'a,
{
    let bubble = container(
        text(label)
            .size(12)
            .color(crate::theme::color::current().cream),
    )
    .padding([5, 9]);
    Tooltip::new(content, bubble, position)
        .gap(4)
        .delay(TOOLTIP_DELAY)
        .style(icons::tooltip_bubble_style())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_widget::text;

    #[test]
    fn the_tooltip_widget_tree_shape_does_not_depend_on_show() {
        // 到期才把内容包进 `Tooltip`,控件树结构随之翻转,里面 `MouseArea` 的状态被重置成
        // "未悬停":下一帧会再发一次 enter(重置提示计时,气泡闪一下就没),翻转那一帧
        // 若光标已离开则 exit 永远不触发(悬停底色卡住)。结构必须恒定,延迟交给
        // `Tooltip::delay`。
        let shape = |show: bool| {
            let content: iced_widget::core::Element<
                '_,
                (),
                iced_widget::Theme,
                iced_renderer::Renderer,
            > = text("tab").into();
            let el = controlled_tooltip(
                content,
                "full title".into(),
                iced_widget::tooltip::Position::Top,
                show,
            );
            (el.as_widget().tag(), el.as_widget().children().len())
        };
        assert_eq!(shape(true), shape(false));
    }

    #[test]
    fn the_tooltip_delay_is_two_seconds_like_dozer_and_digger() {
        assert_eq!(TOOLTIP_DELAY, std::time::Duration::from_secs(2));
    }
}

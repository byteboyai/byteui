//! amis `dialog`(对话框)外观原语:弹窗卡片样式、底部操作按钮行、操作按钮样式,
//! 以及"标题 + 说明 + 取消/确认"的确认框骨架。从 dozer-app 搬来,供多个项目共用。
//!
//! 只管弹窗**内容**的外观;弹窗本身如何显示(独立原生窗口、遮罩等)由应用决定,
//! 不在这里。

use iced_widget::core::{Border, Color, Element, Length, alignment::Horizontal};
use iced_widget::{Row, button, column, container, row, text};

use crate::interaction::icons::IconKind;

/// 弹窗卡片容器样式:PANEL 底 + 金色描边 + 圆角。描边宽/圆角是组件内常量
/// (不随全局缩放变化,与搬迁前 dozer 的 `region::dialog()` 行为一致)。
pub fn card_style(_t: &iced_widget::Theme) -> container::Style {
    let colors = crate::theme::color::current();
    container::Style {
        background: Some(colors.panel.into()),
        border: Border {
            color: colors.gold,
            width: 1.5,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

/// 弹窗底部操作按钮行:靠右下角对齐(取消在左、确认在右的相对顺序不变,
/// 只是整行不再贴左/居中)。
pub fn actions<'a, Msg: 'a>(
    row: Row<'a, Msg, iced_widget::Theme, iced_renderer::Renderer>,
) -> Element<'a, Msg, iced_widget::Theme, iced_renderer::Renderer> {
    container(row)
        .width(Length::Fill)
        .align_x(Horizontal::Right)
        .into()
}

/// 弹窗/footer-bar 按钮共用的描边规则:静止态固定 `border`,
/// 悬浮/按下态一律变 `gold`。
pub fn action_button_border_color(status: button::Status) -> Color {
    match status {
        button::Status::Hovered | button::Status::Pressed => crate::theme::color::current().gold,
        _ => crate::theme::color::current().border,
    }
}

/// 弹窗操作按钮(取消/确认/删除)完整样式:PANEL 底 + 统一描边规则 + 调用方
/// 指定的文字色。文字色按语义传:`dim` 给取消/次要,`red` 给危险删除,
/// `gold` 给非破坏性的主要确认。底色与弹窗卡片同走 `panel` 主题色,保持一致。
pub fn action_button_style(
    text_color: Color,
) -> impl Fn(&iced_widget::Theme, button::Status) -> button::Style {
    move |_t, s| {
        let colors = crate::theme::color::current();
        button::Style {
            // 静止态底走 `panel`，悬浮/按下态提亮到 `card`——配合描边变金的
            // 既有规则，给弹窗操作按钮一个明确的 hover 反馈。
            background: Some(
                match s {
                    button::Status::Hovered | button::Status::Pressed => colors.card,
                    _ => colors.panel,
                }
                .into(),
            ),
            text_color,
            border: Border {
                color: action_button_border_color(s),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..button::Style::default()
        }
    }
}

/// `confirm()` 的入参——字段数≥7 且 `title`/`description` 两个相邻同类型
/// `String` 传错顺序编译器发现不了，用具名字段结构体代替位置参数。
#[derive(Clone)]
pub struct ConfirmDialog<Msg> {
    /// 标题前的可选图标（无图标传 `None`）。
    pub icon: Option<IconKind>,
    pub title: String,
    pub description: String,
    pub cancel_label: String,
    pub cancel_msg: Msg,
    pub confirm_label: String,
    pub confirm_msg: Msg,
    /// 右上角关闭按钮：传 `Some(msg)` 才在标题行右侧渲染 × 图标按钮，
    /// `None` 不渲染。× 与「取消」语义等价——都关掉弹窗、不执行确认动作。
    pub close_msg: Option<Msg>,
    /// 确认按钮文字色：`red` 给危险删除，`gold` 给非破坏性主要确认。
    pub confirm_color: Color,
    /// 标题/说明/按钮行之间的纵向间距,由调用方指定(各处原弹窗并不统一)。
    pub content_spacing: f32,
}

/// 标题 + 说明 + 取消/确认两按钮的确认弹窗骨架。
/// 只适用于"纯文字+两按钮"的简单确认框；带输入框/单选组等额外控件的弹窗
/// 不适用，继续各自实现。
///
/// 卡片宽度用 `Length::Fill`:调用方应把它放进自己的宿主窗口(如独立原生
/// 子窗口),卡片填满宿主,而不是按主窗口宽度取比例。
pub fn confirm<'a, Msg: 'a + Clone>(
    spec: ConfirmDialog<Msg>,
) -> Element<'a, Msg, iced_widget::Theme, iced_renderer::Renderer> {
    let colors = crate::theme::color::current();
    let title_content: Element<'a, Msg, iced_widget::Theme, iced_renderer::Renderer> = match spec
        .icon
    {
        Some(icon) => row![
            crate::interaction::icons::view(icon, crate::theme::icon_size::row(), colors.cream,),
            text(spec.title)
                .size(crate::theme::font::subtitle())
                .color(colors.cream),
        ]
        .spacing(6)
        .align_y(iced_widget::core::Alignment::Center)
        .into(),
        None => text(spec.title)
            .size(crate::theme::font::subtitle())
            .color(colors.cream)
            .into(),
    };
    // 标题行右侧的可选 × 关闭按钮：仅 `close_msg` 为 `Some` 时渲染，把按钮
    // 推到最右；无关闭按钮时整行就是标题本身。
    let header: Element<'a, Msg, iced_widget::Theme, iced_renderer::Renderer> = match spec.close_msg
    {
        Some(close_msg) => {
            let close_btn = button(crate::interaction::icons::view(
                IconKind::X,
                crate::theme::icon_size::row(),
                colors.dim,
            ))
            .on_press(close_msg)
            .padding(4)
            .style(
                move |_t: &iced_widget::Theme, s: button::Status| button::Style {
                    background: match s {
                        button::Status::Hovered | button::Status::Pressed => Some(
                            Color {
                                a: 0.15,
                                ..colors.gold
                            }
                            .into(),
                        ),
                        _ => None,
                    },
                    border: Border {
                        width: 0.0,
                        ..Border::default()
                    },
                    ..button::Style::default()
                },
            );
            row![title_content, iced_widget::space::horizontal(), close_btn]
                .align_y(iced_widget::core::Alignment::Center)
                .into()
        }
        None => title_content,
    };
    let cancel = button(
        text(spec.cancel_label)
            .size(crate::theme::font::label())
            .color(colors.dim),
    )
    .on_press(spec.cancel_msg)
    .padding([6, 12])
    .style(action_button_style(colors.dim));
    let confirm = button(
        text(spec.confirm_label)
            .size(crate::theme::font::label())
            .color(spec.confirm_color),
    )
    .on_press(spec.confirm_msg)
    .padding([6, 12])
    .style(action_button_style(spec.confirm_color));

    let dialog = container(
        column![
            header,
            text(spec.description)
                .size(crate::theme::font::label())
                .color(colors.dim),
            actions(row![cancel, confirm].spacing(8)),
        ]
        .spacing(spec.content_spacing),
    )
    .width(Length::Fill)
    .padding(16)
    .style(card_style);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(iced_widget::core::alignment::Vertical::Center)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::color::ColorTokens;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Cancel,
        Confirm,
        Close,
    }

    fn spec(close: Option<TestMsg>) -> ConfirmDialog<TestMsg> {
        ConfirmDialog {
            icon: None,
            title: "删除文件 \"a.txt\"?".to_string(),
            description: "会移入系统回收站。".to_string(),
            cancel_label: "取消".to_string(),
            cancel_msg: TestMsg::Cancel,
            confirm_label: "删除".to_string(),
            confirm_msg: TestMsg::Confirm,
            close_msg: close,
            confirm_color: ColorTokens::byteboy2077().red,
            content_spacing: 8.0,
        }
    }

    #[test]
    fn card_style_is_panel_with_gold_border() {
        let c = crate::theme::color::current();
        let s = card_style(&iced_widget::Theme::Dark);
        assert_eq!(s.background, Some(c.panel.into()));
        assert_eq!(s.border.color, c.gold);
        assert_eq!(s.border.width, 1.5);
        assert_eq!(s.border.radius, 8.0.into());
    }

    #[test]
    fn action_border_is_gold_on_hover_and_press_else_border() {
        let c = crate::theme::color::current();
        assert_eq!(action_button_border_color(button::Status::Active), c.border);
        assert_eq!(
            action_button_border_color(button::Status::Disabled),
            c.border
        );
        assert_eq!(action_button_border_color(button::Status::Hovered), c.gold);
        assert_eq!(action_button_border_color(button::Status::Pressed), c.gold);
    }

    #[test]
    fn action_button_background_lifts_to_card_on_hover() {
        let c = crate::theme::color::current();
        let style = action_button_style(c.dim);
        let theme = iced_widget::Theme::Dark;
        assert_eq!(
            style(&theme, button::Status::Active).background,
            Some(c.panel.into())
        );
        assert_eq!(
            style(&theme, button::Status::Hovered).background,
            Some(c.card.into())
        );
        assert_eq!(style(&theme, button::Status::Active).text_color, c.dim);
    }

    #[test]
    fn confirm_dialog_struct_carries_all_fields() {
        let s = spec(None);
        assert_eq!(s.title, "删除文件 \"a.txt\"?");
        assert_eq!(s.confirm_msg, TestMsg::Confirm);
        assert!(s.close_msg.is_none());
    }

    #[test]
    fn confirm_dialog_is_cloneable() {
        let s = spec(Some(TestMsg::Close));
        let cloned = s.clone();
        assert_eq!(cloned.title, s.title);
        assert_eq!(cloned.close_msg, Some(TestMsg::Close));
    }

    #[test]
    fn confirm_constructs_with_and_without_icon_and_close() {
        let _: Element<TestMsg, iced_widget::Theme, iced_renderer::Renderer> = confirm(spec(None));
        let mut with_icon = spec(Some(TestMsg::Close));
        with_icon.icon = Some(IconKind::X);
        let _: Element<TestMsg, iced_widget::Theme, iced_renderer::Renderer> = confirm(with_icon);
    }

    #[test]
    fn actions_constructs() {
        let r: Row<TestMsg, iced_widget::Theme, iced_renderer::Renderer> = row![];
        let _: Element<TestMsg, iced_widget::Theme, iced_renderer::Renderer> = actions(r);
    }
}

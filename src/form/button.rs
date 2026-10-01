//! amis `button`(按钮):<https://baidu.github.io/amis/zh-CN/components/button>
//! 金色主按钮(甲方的关键动作,如"批准并提交""打开项目")与次要描边按钮
//! (如"拒绝")。`on_press` 为 `None` 即禁用:不产生消息,样式变暗。

use iced_widget::button::{self, Status};
use iced_widget::core::{Border, Color, Element, Padding};
use iced_widget::{Row, text};

use crate::theme::color::{ColorTokens, current, mix};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Primary,
    Secondary,
}

/// 纯函数:状态 → 样式,便于不起窗口地测试。
pub fn style_for(kind: Kind, status: Status, colors: &ColorTokens) -> button::Style {
    let radius = 6.0;
    match (kind, status) {
        (Kind::Primary, Status::Disabled) => button::Style {
            background: Some(mix(colors.panel, colors.gold, 0.25).into()),
            text_color: colors.dim,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            ..button::Style::default()
        },
        (Kind::Primary, status) => {
            let bg = match status {
                Status::Hovered | Status::Pressed => mix(colors.gold, Color::WHITE, 0.15),
                _ => colors.gold,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: colors.panel,
                border: Border {
                    radius: radius.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }
        }
        (Kind::Secondary, Status::Disabled) => button::Style {
            background: None,
            text_color: colors.dim,
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: radius.into(),
            },
            ..button::Style::default()
        },
        (Kind::Secondary, status) => {
            let border = match status {
                Status::Hovered | Status::Pressed => colors.gold,
                _ => colors.border,
            };
            button::Style {
                background: None,
                text_color: colors.cream,
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: radius.into(),
                },
                ..button::Style::default()
            }
        }
    }
}

pub fn view<'a, Message: Clone + 'a>(
    label: &'a str,
    kind: Kind,
    leading: Option<Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer>>,
    on_press: Option<Message>,
) -> Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer> {
    let mut content = Row::new()
        .spacing(6)
        .align_y(iced_widget::core::alignment::Vertical::Center);
    if let Some(icon) = leading {
        content = content.push(icon);
    }
    content = content.push(text(label).size(crate::theme::font::label()));
    iced_widget::button(content)
        .padding(Padding::from([6.0, 14.0]))
        .on_press_maybe(on_press)
        .style(move |_theme: &iced_widget::Theme, status: Status| {
            style_for(kind, status, &current())
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum Msg {
        Pressed,
    }

    #[test]
    fn primary_uses_gold_background_and_dark_text() {
        let c = ColorTokens::byteboy2077();
        let s = style_for(Kind::Primary, Status::Active, &c);
        assert_eq!(s.background, Some(c.gold.into()));
        assert_eq!(s.text_color, c.panel);
    }

    #[test]
    fn secondary_is_outlined_with_transparent_background() {
        let c = ColorTokens::byteboy2077();
        let s = style_for(Kind::Secondary, Status::Active, &c);
        assert_eq!(s.background, None);
        assert_eq!(s.border.color, c.border);
        assert_eq!(s.text_color, c.cream);
    }

    #[test]
    fn disabled_is_dimmed_for_both_kinds() {
        let c = ColorTokens::byteboy2077();
        for kind in [Kind::Primary, Kind::Secondary] {
            let s = style_for(kind, Status::Disabled, &c);
            assert_eq!(s.text_color, c.dim, "{kind:?}");
        }
    }

    #[test]
    fn constructs_enabled_disabled_and_with_leading() {
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> =
            view("批准并提交", Kind::Primary, None, Some(Msg::Pressed));
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> =
            view("拒绝", Kind::Secondary, None, None);
        let icon: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> =
            iced_widget::text("+").into();
        let _ = view("新建项目", Kind::Primary, Some(icon), Some(Msg::Pressed));
    }
}

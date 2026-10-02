//! amis `badge`(角标):<https://baidu.github.io/amis/zh-CN/components/badge>
//! 数字角标(如"待审变更 3")与红点角标。超过上限显示 `99+` 这类写法。
//! 是否显示(例如数量为 0 时隐藏)由调用方决定,组件只负责渲染。

use iced_widget::core::{Border, Element};
use iced_widget::{container, text};

use crate::theme::color::current;

/// 数字显示文本:超过 `max` 显示 `{max}+`。
pub fn count_label(count: u32, max: u32) -> String {
    if count > max {
        format!("{max}+")
    } else {
        count.to_string()
    }
}

/// 数字角标:金色底、深色字,用于"待审变更 3"这类需要引起甲方注意的计数。
pub fn count<'a, Message: 'a>(
    count: u32,
    max: u32,
) -> Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer> {
    let colors = current();
    container(
        text(count_label(count, max))
            .size(crate::theme::font::caption_sm())
            .color(colors.panel),
    )
    .padding([1.0, 6.0])
    .style(move |_theme: &iced_widget::Theme| container::Style {
        background: Some(colors.gold.into()),
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

/// 红点角标:只表示"有新内容",不带数字。
pub fn dot<'a, Message: 'a>() -> Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer> {
    let colors = current();
    let size = crate::theme::font::dot_sm() as f32 * 0.8;
    container(iced_widget::Space::new())
        .width(size)
        .height(size)
        .style(move |_theme: &iced_widget::Theme| container::Style {
            background: Some(colors.red.into()),
            border: Border {
                radius: (size / 2.0).into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum Msg {}

    #[test]
    fn count_label_caps_at_max() {
        assert_eq!(count_label(0, 99), "0");
        assert_eq!(count_label(99, 99), "99");
        assert_eq!(count_label(100, 99), "99+");
        assert_eq!(count_label(5, 9), "5");
        assert_eq!(count_label(10, 9), "9+");
    }

    #[test]
    fn badge_components_construct_without_panic() {
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> = count(3, 99);
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> = count(0, 99);
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> = dot();
    }
}

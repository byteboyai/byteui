//! amis `tag`(标签):<https://baidu.github.io/amis/zh-CN/components/tag>
//! 带文字的小标签,如"可审查""冲突""仅本地"。颜色用 `Tone` 选语义色,
//! 底色是语义色与面板色按固定比例混合,不写死色值。

use iced_widget::core::{Border, Color, Element};
use iced_widget::{container, text};

use crate::theme::color::{ColorTokens, current, mix};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tone {
    Neutral,
    Gold,
    Cyan,
    Green,
    Red,
}

pub fn tone_color(tone: Tone, colors: &ColorTokens) -> Color {
    match tone {
        Tone::Neutral => colors.dim,
        Tone::Gold => colors.gold,
        Tone::Cyan => colors.cyan,
        Tone::Green => colors.green,
        Tone::Red => colors.red,
    }
}

pub fn view<'a, Message: 'a>(
    text_content: &'a str,
    tone: Tone,
) -> Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer> {
    let colors = current();
    let accent = tone_color(tone, &colors);
    let bg = mix(colors.panel, accent, 0.18);
    let border = mix(colors.panel, accent, 0.45);
    container(
        text(text_content)
            .size(crate::theme::font::caption_sm())
            .color(accent),
    )
    .padding([2.0, 8.0])
    .style(move |_theme: &iced_widget::Theme| container::Style {
        background: Some(bg.into()),
        border: Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
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
    fn tone_color_maps_to_semantic_tokens() {
        let c = ColorTokens::byteboy2077();
        assert_eq!(tone_color(Tone::Gold, &c), c.gold);
        assert_eq!(tone_color(Tone::Cyan, &c), c.cyan);
        assert_eq!(tone_color(Tone::Green, &c), c.green);
        assert_eq!(tone_color(Tone::Red, &c), c.red);
        assert_eq!(tone_color(Tone::Neutral, &c), c.dim);
    }

    #[test]
    fn empty_text_constructs_without_panic() {
        let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> = view("", Tone::Neutral);
    }

    #[test]
    fn every_tone_constructs_without_panic() {
        for tone in [
            Tone::Neutral,
            Tone::Gold,
            Tone::Cyan,
            Tone::Green,
            Tone::Red,
        ] {
            let _: Element<Msg, iced_widget::Theme, iced_renderer::Renderer> = view("可审查", tone);
        }
    }
}

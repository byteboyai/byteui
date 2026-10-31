//! 面板标题栏:图标 + 标题 + 可选的尾部操作 + 底部 1px 分割线。
//!
//! Dozer 的 `home_panel_head` 与 Digger 的 conversations/group-chat 适配层
//! 原先各自手写同一套行布局(固定高度、图标/标题间距、右侧 Fill 推操作、
//! 标题下 1px 分割线)。这里抽出**最小共享接口**;标题字号、颜色、间距与
//! 分割线间距仍可逐项覆盖，布局确实不同的产品面板不必强行套用。

use iced_widget::core::{Alignment, Color, Element, Length};
use iced_widget::{column, container, row, text};

use crate::interaction::icons::{self, IconKind};

/// 面板标题栏。默认值对齐 ByteUI 的设计基准:
/// - 图标尺寸 `icon_size::row()`
/// - 标题字号 `font::title()`
/// - 强调色 `tab_active_border`
/// - 图标/标题间距 8、行高 `geometry::tab_button_size()`
/// - 底部 1px `border` 色分割线，标题行到分割线间距 4
pub struct PanelHeading<'a, Message> {
    icon: IconKind,
    title: std::borrow::Cow<'a, str>,
    actions: Option<Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer>>,
    icon_size: f32,
    title_size: u32,
    color: Color,
    spacing: f32,
    height: f32,
    divider: bool,
    divider_spacing: f32,
    divider_color: Option<Color>,
}

impl<'a, Message: 'a> PanelHeading<'a, Message> {
    pub fn new(icon: IconKind, title: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        Self {
            icon,
            title: title.into(),
            actions: None,
            icon_size: crate::theme::icon_size::row(),
            title_size: crate::theme::font::title(),
            color: crate::theme::color::current().tab_active_border,
            spacing: 8.0,
            height: crate::theme::geometry::tab_button_size(),
            divider: true,
            divider_spacing: 4.0,
            divider_color: None,
        }
    }

    /// 右侧尾部操作：用 `Length::Fill` 间隔推到行尾。
    pub fn actions(
        mut self,
        actions: impl Into<Option<Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer>>>,
    ) -> Self {
        self.actions = actions.into();
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn title_size(mut self, size: u32) -> Self {
        self.title_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// 关掉底部 1px 分割线(由产品自己画)。
    pub fn without_divider(mut self) -> Self {
        self.divider = false;
        self
    }

    /// 标题行到分割线的间距。
    pub fn divider_spacing(mut self, spacing: f32) -> Self {
        self.divider_spacing = spacing;
        self
    }

    /// 覆盖分割线颜色；默认取当前主题的 `border`。
    pub fn divider_color(mut self, color: Color) -> Self {
        self.divider_color = Some(color);
        self
    }

    pub fn into_element(self) -> Element<'a, Message, iced_widget::Theme, iced_renderer::Renderer> {
        let PanelHeading {
            icon,
            title,
            actions,
            icon_size,
            title_size,
            color,
            spacing,
            height,
            divider,
            divider_spacing,
            divider_color,
        } = self;

        let mut head = row![
            icons::view(icon, icon_size, color),
            text(title).size(title_size).color(color),
        ]
        .spacing(spacing)
        .height(Length::Fixed(height))
        .align_y(Alignment::Center);

        if let Some(actions) = actions {
            head = head
                .push(iced_widget::space::horizontal())
                .push(actions)
                .width(Length::Fill);
        }

        if !divider {
            return head.into();
        }

        let line_color = divider_color.unwrap_or_else(|| crate::theme::color::current().border);
        column![
            head,
            container(iced_widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(move |_t: &iced_widget::Theme| container::Style {
                    background: Some(line_color.into()),
                    ..container::Style::default()
                }),
        ]
        .spacing(divider_spacing)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    enum Msg {}

    fn action() -> Element<'static, Msg, iced_widget::Theme, iced_renderer::Renderer> {
        container(iced_widget::Space::new()).into()
    }

    #[test]
    fn constructs_with_and_without_actions_and_divider() {
        let _: Element<'_, Msg, _, _> =
            PanelHeading::<Msg>::new(IconKind::ListTodo, "Todo").into_element();
        let _: Element<'_, Msg, _, _> = PanelHeading::new(IconKind::ListTodo, "Todo".to_string())
            .actions(action())
            .into_element();
        let _: Element<'_, Msg, _, _> = PanelHeading::new(IconKind::ListTodo, "Todo")
            .without_divider()
            .title_size(15)
            .divider_color(Color::WHITE)
            .into_element();
    }
}

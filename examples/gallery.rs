//! 组件展示:把各组件的各种状态摆在一个窗口里,改组件后在这里目测。
//! 运行:`cargo run --example gallery`;浅色:`GALLERY_SCHEME=light cargo run --example gallery`

use byteui::data::{badge, card, tag};
use byteui::feedback::status;
use byteui::form::{button, switch};
use byteui::interaction::icons::{self, IconKind};
use byteui::theme::color;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

#[derive(Default)]
struct Gallery {
    switch_on: bool,
}

#[derive(Clone)]
enum Msg {
    Toggled(bool),
    Noop,
}

impl Gallery {
    fn update(&mut self, msg: Msg) {
        if let Msg::Toggled(v) = msg {
            self.switch_on = v;
        }
    }

    fn theme(&self) -> iced::Theme {
        match color::current_scheme() {
            color::ColorScheme::Light => iced::Theme::Light,
            _ => iced::Theme::Dark,
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        let c = color::current();
        let title = |s: &'static str| text(s).size(14).color(c.cream);
        let body = column![
            title("Button"),
            row![
                button::view("批准并提交", button::Kind::Primary, None, Some(Msg::Noop)),
                button::view("拒绝", button::Kind::Secondary, None, Some(Msg::Noop)),
                button::view("已禁用", button::Kind::Primary, None, None),
                button::view("已禁用", button::Kind::Secondary, None, None),
                button::view(
                    "新建项目",
                    button::Kind::Primary,
                    Some(icons::view(IconKind::Plus, 14.0, c.panel)),
                    Some(Msg::Noop),
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            title("Tag"),
            row![
                tag::view("可审查", tag::Tone::Green),
                tag::view("冲突", tag::Tone::Red),
                tag::view("仅本地", tag::Tone::Cyan),
                tag::view("r128", tag::Tone::Gold),
                tag::view("v7", tag::Tone::Neutral),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            title("Badge"),
            row![
                badge::count(0, 99),
                badge::count(3, 99),
                badge::count(120, 99),
                badge::dot(),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            title("Status / Switch / Card"),
            row![
                status::dot(c.green),
                status::dot(c.gold),
                status::dot(c.red),
                switch::view("开关", self.switch_on, Msg::Toggled),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            card::view("Project Memory", Some("r128 · 健康"), false, false),
            title("Glyph badge(普通 / 选中 / 拖拽 dim;不同底色、含中文)"),
            row![
                glyph(&c, 'P', c.cyan, false, false),
                glyph(&c, '双', c.magenta, false, false),
                glyph(&c, 'N', c.lime, true, false),
                glyph(&c, 'E', c.purple, false, true),
                glyph(&c, '?', c.orange, false, false),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            title("Glyph badge:全部 accent 色(检查深/浅色下字符是否可读)"),
            row(accents(&c)
                .into_iter()
                .zip("ABCDEFGH".chars())
                .map(|(fill, ch)| glyph(&c, ch, fill, false, false)))
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(14);
        container(body)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(c.panel.into()),
                ..container::Style::default()
            })
            .into()
    }
}

/// 图标栏里的一个字符徽标按钮(`active` = 选中金框,`dim` = 拖拽源变淡)。
fn glyph(
    c: &color::ColorTokens,
    ch: char,
    fill: iced::Color,
    active: bool,
    dim: bool,
) -> Element<'static, Msg> {
    icons::glyph_button_entry(
        ch,
        fill,
        icons::readable_ink(fill, c.cream, c.bg),
        28.0,
        active,
        dim,
        0.0,
        true,
        40.0,
        true,
        Msg::Noop,
        |_| Msg::Noop,
        "示例应用",
    )
}

fn accents(c: &color::ColorTokens) -> [iced::Color; 8] {
    [c.cyan, c.green, c.purple, c.orange, c.magenta, c.blue, c.lime, c.teal]
}

fn main() -> iced::Result {
    if std::env::var("GALLERY_SCHEME").is_ok_and(|v| v == "light") {
        color::set_scheme(color::ColorScheme::Light);
    }
    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .theme(Gallery::theme)
        .run()
}

//! 组件展示:把各组件的各种状态摆在一个窗口里,改组件后在这里目测。
//! 运行:`cargo run --example gallery`

use byteui::data::{badge, card, tag};
use byteui::feedback::status;
use byteui::form::{button, switch};
use byteui::theme::color;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

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
        iced::Theme::Dark
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
            ]
            .spacing(8),
            title("Tag"),
            row![
                tag::view("可审查", tag::Tone::Green),
                tag::view("冲突", tag::Tone::Red),
                tag::view("仅本地", tag::Tone::Cyan),
                tag::view("r128", tag::Tone::Gold),
                tag::view("v7", tag::Tone::Neutral),
            ]
            .spacing(8),
            title("Badge"),
            row![badge::count(3, 99), badge::count(120, 99), badge::dot()].spacing(8),
            title("Status / Switch / Card"),
            row![
                status::dot(c.green),
                status::dot(c.gold),
                status::dot(c.red),
                switch::view("开关", self.switch_on, Msg::Toggled),
            ]
            .spacing(8),
            card::view("Project Memory", Some("r128 · 健康"), false, false),
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

fn main() -> iced::Result {
    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .theme(Gallery::theme)
        .run()
}

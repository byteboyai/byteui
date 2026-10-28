//! 统一弹出菜单(popup menu)样式原语:菜单项、分隔线、外框,以及 iced 自绘浮层(锚点钳制 + 遮罩定位)。
//!
//! 来源:Dozer `chrome/menu.rs`(项/分隔线/外框,以文件树右键菜单为唯一视觉基准)与 Digger
//! `adapters/menu_overlay.rs`(`MenuAnchor`、`OverlayRow`、`view_rows`、`rows_size`,已泛型化)。
//! 外框的度量来自 [`crate::theme::region::context_menu`](不透明背景)。
//!
//! - 外框 [`shell`]:`context_menu` 区域(底色/描边/圆角/内边距/项间距,已含全局 scale)。
//! - 单项 [`item`]:恒定宽 `menu_item_width` 的"图标+文字";[`item_row`] 接受任意前置元素 + 自定义文字色 +
//!   可选点击/锁定;[`item_row_fill`] 为整行撑满(`Fill`)的版本。hover/pressed → `TAB_HOVER` 底 + 圆角 6,
//!   高亮矩形左右各再内缩 `menu_hover_inset`。
//! - 锁定:`msg` 为 `None` 时置灰且不可点、hover 不高亮。
//! - 分组 [`separator`]:1px `BORDER` 分隔线,左右各内缩 `menu_pad_h`。
//! - 浮层:[`view_rows`] 按已规整的行渲染整窗遮罩 + 钳制后的菜单;[`overlay_shell`] 给自带行的调用方
//!   (溢出菜单)复用同一外框与遮罩。**注意 [`shell`] 与 [`overlay_shell`] 不是一回事**:前者只是外框,
//!   后者还带遮罩和定位。
//!
//! 所有原语对消息类型 `Msg` 泛型化。Dozer 的 `shell_frosted`(叠噪点贴图)依赖 Dozer 本地的 `frosted`,留在 Dozer。

use crate::interaction::icons;
use iced_widget::core::{Border, Color, Element, Length, Padding, Shadow, Vector};
use iced_widget::{button, column, container, row, text};

type El<'a, Msg> = Element<'a, Msg, iced_widget::Theme, iced_renderer::Renderer>;

/// macOS 原生右键菜单的高亮/命中区是较大的圆角矩形(而非直角),`item`/`item_row` 的 hover/pressed 态、
/// `item_row_fill` 一并复用这个半径。
const MENU_HOVER_RADIUS: f32 = 6.0;

/// 单个菜单项:可选图标 + 文字。常宽固定(菜单不随 label 长短伸缩),hover/pressed 切到 `TAB_HOVER` 底。
/// 文字与图标取默认 `BODY`。`msg` 为点击下发消息。
pub fn item<'a, Msg: 'a + Clone>(
    icon: Option<icons::IconKind>,
    label: impl Into<String>,
    msg: Msg,
) -> El<'a, Msg> {
    item_row(
        icon_leading(icon, crate::theme::color::current().body),
        label,
        crate::theme::color::current().body,
        Some(msg),
    )
}

/// 置灰且不可点的菜单项:无 `on_press`,hover 也不高亮,始终透出容器底。文字/图标取 `color`(调用方置 `DIM`)。
pub fn item_locked<'a, Msg: 'a + Clone>(
    icon: Option<icons::IconKind>,
    label: impl Into<String>,
    color: Color,
) -> El<'a, Msg> {
    item_row(icon_leading(icon, color), label, color, None)
}

/// 最通用的菜单项:接受任意**前置元素**(图标、状态指示点、复选框…)+ 文字。行为同其它项(常宽、hover 高亮、
/// 可锁定)。`leading` 为 `None` 时只渲染文字。`msg` 为 `Some` 挂点击并 hover 高亮,`None` 表示锁定/置灰。
pub fn item_row<'a, Msg: 'a + Clone>(
    leading: Option<El<'a, Msg>>,
    label: impl Into<String>,
    color: Color,
    msg: Option<Msg>,
) -> El<'a, Msg> {
    menu_button(
        item_content(leading, label.into(), color),
        color,
        msg,
        Length::Fixed(crate::theme::geometry::menu_item_width()),
    )
}

/// 满宽菜单项:同 [`item_row`],但整行撑满可用宽度(`Fill`),供需要整行可点区域的窄面板内选择器使用。
pub fn item_row_fill<'a, Msg: 'a + Clone>(
    leading: Option<El<'a, Msg>>,
    label: impl Into<String>,
    color: Color,
    msg: Option<Msg>,
) -> El<'a, Msg> {
    menu_button(
        item_content(leading, label.into(), color),
        color,
        msg,
        Length::Fill,
    )
}

fn item_content<'a, Msg: 'a>(
    leading: Option<El<'a, Msg>>,
    label: String,
    color: Color,
) -> iced_widget::Row<'a, Msg, iced_widget::Theme, iced_renderer::Renderer> {
    let label = text(label).size(crate::theme::font::label()).color(color);
    match leading {
        Some(leading) => row![leading, label],
        None => row![label],
    }
}

/// 把可选的图标转成前置元素(`None` = 无图标,只渲染文字)。
pub fn icon_leading<'a, Msg: 'a>(
    icon: Option<icons::IconKind>,
    color: Color,
) -> Option<El<'a, Msg>> {
    icon.map(|icon| icons::view(icon, crate::theme::icon_size::row(), color))
}

/// 菜单项之间的细分隔线:1px `BORDER` 高度,宽同菜单常宽。左右各让开 `menu_pad_h`——比 hover 高亮的
/// `menu_hover_inset` 多一截,分隔线因此比高亮矩形略短(2026-09 用户口径),与原生菜单 `separator_view` 的留白量同一数值。
pub fn separator<'a, Msg: 'a>() -> El<'a, Msg> {
    let divider: El<'a, Msg> = container(crate::layout::divider::horizontal())
        .width(Length::Fill)
        .into();
    container(divider)
        .width(Length::Fixed(crate::theme::geometry::menu_item_width()))
        .padding(Padding {
            left: crate::theme::geometry::menu_pad_h(),
            right: crate::theme::geometry::menu_pad_h(),
            ..Padding::default()
        })
        .into()
}

/// 菜单外框的容器样式:`context_menu` 区域的底色/描边 + 软阴影。macOS 原生右键菜单靠系统合成器的实时高斯
/// 模糊把浮层和背景拉开层次;iced 没有实时模糊可用,这里用一圈软阴影模拟"浮起"观感。
fn frame_style() -> impl Fn(&iced_widget::Theme) -> container::Style {
    let region = crate::theme::region::context_menu();
    move |_t: &iced_widget::Theme| container::Style {
        background: region.background.map(Into::into),
        border: region.border.unwrap_or_default(),
        shadow: Shadow {
            color: Color {
                a: 0.45,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 10.0),
            blur_radius: 28.0,
        },
        ..container::Style::default()
    }
}

/// 菜单外壳:把一列菜单项(或分隔线)包进 `context_menu` 区域样式。返回可再套一层定位容器来摆放。`width`
/// 通常传 `Length::Fixed(menu_item_width())` 得到常宽菜单;内容自适应宽的条目可传 `Shrink`。
pub fn shell<'a, Msg: 'a>(items: Vec<El<'a, Msg>>, width: Length) -> El<'a, Msg> {
    let region = crate::theme::region::context_menu();
    container(column(items).spacing(region.gap))
        .width(width)
        .padding(region.padding)
        .style(frame_style())
        .into()
}

/// 菜单项/行的形状:一行图标 + 文字,或一条分隔线。
pub enum OverlayRow<Msg> {
    Entry {
        icon: Option<icons::IconKind>,
        label: String,
        enabled: bool,
        msg: Box<Msg>,
        /// 图标自己的颜色(Agent 选择器用);`None` 跟随文字色。
        tint: Option<Color>,
    },
    Separator,
}

/// 弹出位置:左上角锚在 `(x, y)`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuAnchor {
    pub x: f32,
    pub y: f32,
}

impl MenuAnchor {
    /// 按菜单实际尺寸把锚点钳进窗口:右侧/下侧放不下时向左/上翻,结果不小于 0。
    pub fn clamped(self, size: (f32, f32), window: (f32, f32)) -> Self {
        let (menu_w, menu_h) = size;
        let x = self.x.min((window.0 - menu_w).max(0.0)).max(0.0);
        let y = self.y.min((window.1 - menu_h).max(0.0)).max(0.0);
        Self { x, y }
    }
}

/// 单个菜单行的高度估算(上下内边距 + 一行文字);只用于锚点钳制,与真实渲染高度可能有细微出入。
fn row_height() -> f32 {
    crate::theme::geometry::menu_pad_v() * 2.0 + crate::theme::font::label() as f32 * 1.2
}

/// 已规整行的外形尺寸:宽恒为 `menu_item_width` + 内边距,高按行数估算(分隔线只占一条)。
pub fn rows_size<Msg>(rows: &[OverlayRow<Msg>]) -> (f32, f32) {
    let region = crate::theme::region::context_menu();
    let row_h = row_height();
    let gap = crate::theme::geometry::menu_gap();
    let width =
        crate::theme::geometry::menu_item_width() + region.padding.left + region.padding.right;
    let mut height = region.padding.top + region.padding.bottom;
    for row in rows {
        height += match row {
            OverlayRow::Entry { .. } => row_h + gap,
            OverlayRow::Separator => 1.0 + gap,
        };
    }
    (width, height)
}

/// 整窗透明遮罩(点空白处关闭)+ 把菜单摆到锚点:菜单自己叠在遮罩上吞掉自己的点击。
fn scrim_and_position<'a, Msg: Clone + 'a>(
    menu: El<'a, Msg>,
    anchor: MenuAnchor,
    on_dismiss: Msg,
) -> El<'a, Msg> {
    let scrim: El<'a, Msg> = iced_widget::mouse_area(
        container(iced_widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(on_dismiss)
    .into();
    let positioned: El<'a, Msg> = container(menu)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: anchor.y,
            left: anchor.x,
            right: 0.0,
            bottom: 0.0,
        })
        .into();
    iced_widget::stack![scrim, positioned].into()
}

fn empty_layer<'a, Msg: 'a>() -> El<'a, Msg> {
    container(iced_widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// 按已规整的行渲染菜单浮层:整窗铺满,右下对齐到钳制后的锚点。空行表不弹(返回不拦事件的空层)。
pub fn view_rows<'a, Msg: Clone + 'a>(
    rows: Vec<OverlayRow<Msg>>,
    anchor: MenuAnchor,
    window: (f32, f32),
    on_dismiss: Msg,
) -> El<'a, Msg> {
    if rows.is_empty() {
        return empty_layer();
    }
    let (w, h) = rows_size(&rows);
    let anchor = anchor.clamped((w, h), window);
    let body: Vec<El<'a, Msg>> = rows
        .into_iter()
        .map(|row| match row {
            OverlayRow::Entry {
                icon,
                label,
                enabled,
                msg,
                tint,
            } => {
                let colors = crate::theme::color::current();
                let color = if enabled { colors.body } else { colors.dim };
                item_row(
                    icon_leading(icon, tint.unwrap_or(color)),
                    label,
                    color,
                    enabled.then_some(*msg),
                )
            }
            OverlayRow::Separator => separator(),
        })
        .collect();
    scrim_and_position(shell(body, Length::Fixed(w)), anchor, on_dismiss)
}

/// 菜单外框 + 整窗透明遮罩(溢出菜单等自带行的调用方复用同一视觉)。`content` 是已经拼好的菜单主体;
/// `width` 用于右/下边缘钳制之外的固定宽。
pub fn overlay_shell<'a, Msg: Clone + 'a>(
    content: El<'a, Msg>,
    anchor: MenuAnchor,
    width: f32,
    on_dismiss: Msg,
) -> El<'a, Msg> {
    let region = crate::theme::region::context_menu();
    let menu: El<'a, Msg> = container(content)
        .width(Length::Fixed(width))
        .padding(region.padding)
        .style(frame_style())
        .into();
    scrim_and_position(menu, anchor, on_dismiss)
}

/// 菜单单项按钮的通用样式接线:透明底(透出容器底色)、文字 `base_color`、hover/pressed 切 `TAB_HOVER` 底 +
/// `MENU_HOVER_RADIUS` 圆角,高亮矩形再左右各内缩 `menu_hover_inset`。`Some(msg)` 挂 `on_press`,`None` 表示
/// 锁定/置灰项(不可点、不高亮)。
fn menu_button<'a, Msg: 'a + Clone>(
    content: iced_widget::Row<'a, Msg, iced_widget::Theme, iced_renderer::Renderer>,
    base_color: Color,
    msg: Option<Msg>,
    width: Length,
) -> El<'a, Msg> {
    let content = content
        .spacing(crate::theme::geometry::menu_gap())
        .align_y(iced_widget::core::Alignment::Center);
    let enabled = msg.is_some();
    // 按钮自身左右 padding 扣掉外侧 `menu_hover_inset`,文字位置仍落在 `menu_pad_h`(与原生菜单 `MenuItemView`
    // 的文字起点同一口径);省下的 inset 由外层容器承担,成为高亮矩形到菜单内容边缘的左右留白。
    let inset = crate::theme::geometry::menu_hover_inset();
    let pad_h = (crate::theme::geometry::menu_pad_h() - inset).max(0.0);
    let btn = button(content)
        .width(Length::Fill)
        .padding([crate::theme::geometry::menu_pad_v(), pad_h])
        .style(move |_t: &iced_widget::Theme, s: button::Status| {
            let base = button::Style {
                background: None,
                text_color: base_color,
                ..button::Style::default()
            };
            let hovered =
                matches!(s, button::Status::Hovered) || matches!(s, button::Status::Pressed);
            if enabled && hovered {
                button::Style {
                    background: Some(crate::theme::color::current().tab_hover.into()),
                    text_color: base_color,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: MENU_HOVER_RADIUS.into(),
                    },
                    ..base
                }
            } else {
                base
            }
        });
    let btn: El<'_, Msg> = match msg {
        Some(msg) => btn.on_press(msg).into(),
        None => btn.into(),
    };
    // 左右内缩:外层容器透明底、左右各 padding `menu_hover_inset` 推开内层按钮,hover 高亮矩形只画在按钮上。
    container(btn)
        .width(width)
        .padding(Padding {
            left: inset,
            right: inset,
            ..Padding::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_anchor_clamps_into_the_window() {
        let a = MenuAnchor { x: 900.0, y: 700.0 }.clamped((220.0, 300.0), (1000.0, 800.0));
        assert_eq!(a, MenuAnchor { x: 780.0, y: 500.0 });
        let b = MenuAnchor { x: -5.0, y: -5.0 }.clamped((220.0, 300.0), (1000.0, 800.0));
        assert_eq!(b, MenuAnchor { x: 0.0, y: 0.0 });
    }

    #[test]
    fn menu_anchor_never_goes_negative_when_the_window_is_smaller_than_the_menu() {
        let a = MenuAnchor { x: 50.0, y: 50.0 }.clamped((220.0, 300.0), (100.0, 100.0));
        assert_eq!(a, MenuAnchor { x: 0.0, y: 0.0 });
        assert!(a.x.is_finite() && a.y.is_finite());
    }

    fn entry(label: &str) -> OverlayRow<u8> {
        OverlayRow::Entry {
            icon: None,
            label: label.into(),
            enabled: true,
            msg: Box::new(1),
            tint: None,
        }
    }

    #[test]
    fn rows_size_counts_entries_and_separators() {
        let region = crate::theme::region::context_menu();
        let gap = crate::theme::geometry::menu_gap();
        let (w, h) = rows_size(&[entry("a"), entry("b"), OverlayRow::Separator]);
        assert_eq!(
            w,
            crate::theme::geometry::menu_item_width() + region.padding.left + region.padding.right
        );
        let row_h =
            crate::theme::geometry::menu_pad_v() * 2.0 + crate::theme::font::label() as f32 * 1.2;
        let expected =
            region.padding.top + region.padding.bottom + 2.0 * (row_h + gap) + (1.0 + gap);
        assert!((h - expected).abs() < 0.001, "{h} vs {expected}");
    }

    #[test]
    fn view_rows_with_no_rows_builds_an_empty_layer() {
        let _el: iced_widget::core::Element<'_, u8, iced_widget::Theme, iced_renderer::Renderer> =
            view_rows(
                Vec::<OverlayRow<u8>>::new(),
                MenuAnchor { x: 0.0, y: 0.0 },
                (800.0, 600.0),
                0,
            );
    }

    #[test]
    fn the_shell_background_comes_from_the_opaque_context_menu_region() {
        // `shell` 读 `context_menu()`,不透明是这个区域的约定(见 region 的测试);这里只钉住外壳确实
        // 走它而不是自带一份。
        let bg = crate::theme::region::context_menu().background.unwrap();
        assert_eq!(bg.a, 1.0);
        let _el: iced_widget::core::Element<'_, u8, iced_widget::Theme, iced_renderer::Renderer> =
            shell(Vec::new(), iced_widget::core::Length::Shrink);
    }
}

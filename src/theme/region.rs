//! 外壳区域样式:顶栏、左右图标栏、左右面板区、放大浮层、底栏。
//!
//! 数值与 Dozer `crates/dozer-app/assets/theme/workspace.json` 的同名区域**逐项相同**(那份 JSON 仍是 Dozer 自己的
//! 数据源,Dozer 迁移到本模块之前两处并存;`examples/dump_shell_regions.rs` 导出同结构的规格,供跨仓库一致性检查比对)。
//! 颜色令牌每次现读 `color::current()`(运行时可切配色),`padding` / `margin` / `gap` / 放大浮层尺寸乘全局
//! `icon_size::scale()`。
//!
//! 解析函数显式接收配色和缩放,公开接口只是用当前全局值去调它们——测试因此不碰进程级全局,不会与别的模块的
//! 主题测试互相干扰。
use iced_widget::core::{Border, Color, Padding};
use serde_json::{Value, json};

use super::{color, icon_size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionStyle {
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub padding: Padding,
    pub gap: f32,
    /// 区域整体相对外围容器的外边距(悬浮留白)。渲染时套一层外容器做 inset,几何侧(原生子视图的矩形)要同步扣减。
    pub margin: Padding,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaximizeOverlayStyle {
    pub scrim_background: Color,
    pub scrim_padding: f32,
    pub border: Border,
}

#[derive(Clone, Copy)]
enum Token {
    Bg,
    Panel,
    Border,
    Gold,
    Scrim,
    TabActiveBorder,
}

impl Token {
    fn color(self, c: &color::ColorTokens) -> Color {
        match self {
            Token::Bg => c.bg,
            Token::Panel => c.panel,
            Token::Border => c.border,
            Token::Gold => c.gold,
            Token::Scrim => c.scrim,
            Token::TabActiveBorder => c.tab_active_border,
        }
    }

    /// Dozer `workspace.json` 里的令牌名。
    fn name(self) -> &'static str {
        match self {
            Token::Bg => "BG",
            Token::Panel => "PANEL",
            Token::Border => "BORDER",
            Token::Gold => "GOLD",
            Token::Scrim => "SCRIM",
            Token::TabActiveBorder => "TAB_ACTIVE_BORDER",
        }
    }
}

struct Spec {
    background: Option<Token>,
    /// (颜色, 线宽, 圆角)
    border: Option<(Token, f32, f32)>,
    /// [top, right, bottom, left]
    padding: [f32; 4],
    gap: f32,
    margin: [f32; 4],
}

const ZERO: [f32; 4] = [0.0; 4];

const TOP_BAR: Spec = Spec {
    background: Some(Token::Bg),
    border: Some((Token::Border, 1.0, 0.0)),
    padding: [0.0, 10.0, 0.0, 78.0],
    gap: 8.0,
    margin: ZERO,
};
const LEFT_ICON_RAIL: Spec = Spec {
    background: Some(Token::Panel),
    border: None,
    padding: [16.0, 6.0, 0.0, 6.0],
    gap: 12.0,
    margin: ZERO,
};
const RIGHT_ICON_RAIL: Spec = Spec {
    background: Some(Token::Panel),
    border: None,
    padding: [16.0, 6.0, 0.0, 6.0],
    gap: 12.0,
    margin: ZERO,
};
const STATUS_BAR: Spec = Spec {
    background: Some(Token::Panel),
    border: Some((Token::Border, 1.0, 0.0)),
    padding: [0.0, 8.0, 0.0, 8.0],
    gap: 0.0,
    margin: ZERO,
};
const LEFT_ZONE: Spec = Spec {
    background: Some(Token::Bg),
    border: Some((Token::Border, 0.0, 8.0)),
    padding: [1.0; 4],
    gap: 0.0,
    margin: [6.0, 0.0, 0.0, 6.0],
};
const RIGHT_ZONE: Spec = Spec {
    background: Some(Token::Bg),
    border: Some((Token::Border, 0.0, 8.0)),
    padding: [1.0; 4],
    gap: 0.0,
    margin: [6.0, 6.0, 0.0, 0.0],
};

/// 放大浮层:遮罩背景令牌、遮罩内边距、描边(颜色令牌, 线宽, 圆角)。
const MAXIMIZE_SCRIM: Token = Token::Scrim;
const MAXIMIZE_SCRIM_PADDING: f32 = 40.0;
const MAXIMIZE_BORDER: (Token, f32, f32) = (Token::Gold, 1.5, 10.0);

fn scaled(p: [f32; 4], s: f32) -> Padding {
    Padding {
        top: p[0] * s,
        right: p[1] * s,
        bottom: p[2] * s,
        left: p[3] * s,
    }
}

fn resolve(spec: &Spec, c: &color::ColorTokens, s: f32) -> RegionStyle {
    RegionStyle {
        background: spec.background.map(|t| t.color(c)),
        border: spec.border.map(|(t, width, radius)| Border {
            color: t.color(c),
            width,
            radius: radius.into(),
        }),
        padding: scaled(spec.padding, s),
        gap: spec.gap * s,
        margin: scaled(spec.margin, s),
    }
}

fn resolve_overlay(c: &color::ColorTokens, s: f32) -> MaximizeOverlayStyle {
    let (t, width, radius) = MAXIMIZE_BORDER;
    MaximizeOverlayStyle {
        scrim_background: MAXIMIZE_SCRIM.color(c),
        scrim_padding: MAXIMIZE_SCRIM_PADDING * s,
        border: Border {
            color: t.color(c),
            width: width * s,
            radius: (radius * s).into(),
        },
    }
}

pub fn top_bar() -> RegionStyle {
    resolve(&TOP_BAR, &color::current(), icon_size::scale())
}
pub fn left_icon_rail() -> RegionStyle {
    resolve(&LEFT_ICON_RAIL, &color::current(), icon_size::scale())
}
pub fn right_icon_rail() -> RegionStyle {
    resolve(&RIGHT_ICON_RAIL, &color::current(), icon_size::scale())
}
pub fn status_bar() -> RegionStyle {
    resolve(&STATUS_BAR, &color::current(), icon_size::scale())
}
/// 左面板区整体外框:把"左1左2两栏"框成一个视觉整体,不是某一栏自己的边框。
pub fn left_zone() -> RegionStyle {
    resolve(&LEFT_ZONE, &color::current(), icon_size::scale())
}
/// 右面板区整体外框,同 `left_zone`。
pub fn right_zone() -> RegionStyle {
    resolve(&RIGHT_ZONE, &color::current(), icon_size::scale())
}
pub fn maximize_overlay() -> MaximizeOverlayStyle {
    resolve_overlay(&color::current(), icon_size::scale())
}
/// 窗口根容器底色 / 渲染清屏色(Dozer `workspace.json` 的顶层 `background`,令牌 `TAB_ACTIVE_BORDER`)。
pub fn window_background() -> Color {
    Token::TabActiveBorder.color(&color::current())
}

fn spec_json(spec: &Spec) -> Value {
    json!({
        "background": spec.background.map(Token::name),
        "border": spec.border.map(|(t, width, radius)| json!({ "color": t.name(), "width": width, "radius": radius })),
        "padding": spec.padding,
        "gap": spec.gap,
        "margin": spec.margin,
    })
}

/// 7 个外壳区域的原始规格(令牌名 + 未缩放的数值),结构与 Dozer `workspace.json` 的 `regions` 对应项一致
/// (`padding` / `margin` 一律展开为 `[top, right, bottom, left]`)。仅供跨仓库一致性检查使用。
#[doc(hidden)]
pub fn spec_dump() -> Value {
    let (t, width, radius) = MAXIMIZE_BORDER;
    json!({
        "top_bar": spec_json(&TOP_BAR),
        "left_icon_rail": spec_json(&LEFT_ICON_RAIL),
        "right_icon_rail": spec_json(&RIGHT_ICON_RAIL),
        "status_bar": spec_json(&STATUS_BAR),
        "left_zone": spec_json(&LEFT_ZONE),
        "right_zone": spec_json(&RIGHT_ZONE),
        "maximize_overlay": {
            "scrim": { "background": MAXIMIZE_SCRIM.name(), "padding": MAXIMIZE_SCRIM_PADDING },
            "border": { "color": t.name(), "width": width, "radius": radius },
        },
        "background": Token::TabActiveBorder.name(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> color::ColorTokens {
        color::ColorTokens::byteboy2077()
    }

    fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
        Padding {
            top: t,
            right: r,
            bottom: b,
            left: l,
        }
    }

    /// 防漂移锚:数值必须与 Dozer `workspace.json` 的同名区域一致(跨仓库一致性检查是第二道闸)。
    #[test]
    fn the_regions_match_dozers_workspace_json_literals() {
        let c = tokens();
        let top = resolve(&TOP_BAR, &c, 1.0);
        assert_eq!(top.background, Some(c.bg));
        assert_eq!(
            top.border,
            Some(Border {
                color: c.border,
                width: 1.0,
                radius: 0.0.into()
            })
        );
        assert_eq!(
            top.padding,
            pad(0.0, 10.0, 0.0, 78.0),
            "left 78 leaves room for the traffic lights"
        );
        assert_eq!(top.gap, 8.0);

        for rail in [&LEFT_ICON_RAIL, &RIGHT_ICON_RAIL] {
            let r = resolve(rail, &c, 1.0);
            assert_eq!(r.background, Some(c.panel));
            assert_eq!(r.border, None);
            assert_eq!(r.padding, pad(16.0, 6.0, 0.0, 6.0));
            assert_eq!(r.gap, 12.0);
        }

        let status = resolve(&STATUS_BAR, &c, 1.0);
        assert_eq!(status.background, Some(c.panel));
        assert_eq!(status.padding, pad(0.0, 8.0, 0.0, 8.0));

        let left = resolve(&LEFT_ZONE, &c, 1.0);
        assert_eq!(left.margin, pad(6.0, 0.0, 0.0, 6.0));
        assert_eq!(left.padding, Padding::from(1.0));
        assert_eq!(
            left.border,
            Some(Border {
                color: c.border,
                width: 0.0,
                radius: 8.0.into()
            })
        );
        let right = resolve(&RIGHT_ZONE, &c, 1.0);
        assert_eq!(right.margin, pad(6.0, 6.0, 0.0, 0.0));
        assert_eq!(left.background, Some(c.bg));

        let overlay = resolve_overlay(&c, 1.0);
        assert_eq!(overlay.scrim_background, c.scrim);
        assert_eq!(overlay.scrim_padding, 40.0);
        assert_eq!(
            overlay.border,
            Border {
                color: c.gold,
                width: 1.5,
                radius: 10.0.into()
            }
        );
    }

    #[test]
    fn padding_margin_gap_and_the_overlay_follow_the_global_scale_but_borders_stay_put() {
        let c = tokens();
        let z = resolve(&LEFT_ZONE, &c, 2.0);
        assert_eq!(z.margin, pad(12.0, 0.0, 0.0, 12.0));
        assert_eq!(z.padding, Padding::from(2.0));
        assert_eq!(z.border.unwrap().width, 0.0);
        assert_eq!(
            z.border.unwrap().radius,
            8.0.into(),
            "region borders are not scaled"
        );
        assert_eq!(resolve(&LEFT_ICON_RAIL, &c, 2.0).gap, 24.0);
        let o = resolve_overlay(&c, 2.0);
        assert_eq!(o.scrim_padding, 80.0);
        assert_eq!(o.border.width, 3.0);
        assert_eq!(
            o.border.radius,
            20.0.into(),
            "the overlay frame scales with the UI"
        );
    }

    #[test]
    fn colors_follow_whatever_tokens_are_passed_in() {
        let mut c = tokens();
        c.bg = Color::from_rgb(0.1, 0.2, 0.3);
        assert_eq!(resolve(&TOP_BAR, &c, 1.0).background, Some(c.bg));
        assert_eq!(resolve(&LEFT_ZONE, &c, 1.0).background, Some(c.bg));
    }

    #[test]
    fn the_exported_spec_has_the_dozer_json_shape() {
        let d = spec_dump();
        assert_eq!(d["top_bar"]["padding"], json!([0.0, 10.0, 0.0, 78.0]));
        assert_eq!(d["left_zone"]["margin"], json!([6.0, 0.0, 0.0, 6.0]));
        assert_eq!(d["left_icon_rail"]["border"], Value::Null);
        assert_eq!(
            d["maximize_overlay"]["border"],
            json!({ "color": "GOLD", "width": 1.5, "radius": 10.0 })
        );
        assert_eq!(d["background"], "TAB_ACTIVE_BORDER");
        for region in [
            "top_bar",
            "left_icon_rail",
            "right_icon_rail",
            "status_bar",
            "left_zone",
            "right_zone",
        ] {
            for key in ["background", "border", "padding", "gap", "margin"] {
                assert!(d[region].get(key).is_some(), "{region}.{key}");
            }
        }
    }
}

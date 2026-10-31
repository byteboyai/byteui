//! ByteBoy2077 是编译期默认值,`set_theme` 可在运行时整体替换成另一份
//! 产品的取值——组件内部一律读 `current()`,不再有硬编码色值常量。

use iced_widget::core::Color;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::RwLock;

const fn c(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    pub bg: Color,
    pub panel: Color,
    pub term_bg: Color,
    pub card: Color,
    pub border: Color,
    pub cream: Color,
    pub body: Color,
    pub dim: Color,
    pub gold: Color,
    pub cyan: Color,
    pub green: Color,
    pub purple: Color,
    pub red: Color,
    pub ignored: Color,
    pub orange: Color,
    pub magenta: Color,
    pub blue: Color,
    pub lime: Color,
    pub teal: Color,
    pub scrim: Color,
    pub tab_active_border: Color,
    pub tab_active_bg: Color,
    pub tab_hover: Color,
    pub desc_bg: Color,
}

impl ColorTokens {
    /// `bg`/`panel`(`term_bg` 跟 `panel` 同值)2026-09-16 起拆成两个不同
    /// 数值——此前三者恒等,面板背景语义上分"可收缩的 list 侧用 bg、不可
    /// 收缩的 content 侧用 panel"(见 Dozer 应用侧的
    /// 区域样式 JSON),浅色主题本就两值不同,深色缺这个区分度、用户要求补上。
    /// `panel` 保留原有数值(终端/内容区视觉不变),`bg` 提亮一档给 list 侧
    /// 一点分离感,同色系不出戏。颜色 token 的唯一来源就是这里,
    /// 锁值测试见本文件 `tests`。
    pub const fn byteboy2077() -> Self {
        Self {
            bg: c(0x0d, 0x13, 0x1c),
            panel: c(0x0a, 0x0e, 0x16),
            term_bg: c(0x0a, 0x0e, 0x16),
            card: c(0x12, 0x20, 0x2a),
            border: c(0x1c, 0x34, 0x40),
            cream: c(0xFF, 0xE5, 0xB4),
            body: c(0x9A, 0xB4, 0xC4),
            dim: c(0x6B, 0x7F, 0x8F),
            gold: c(0xF2, 0xD9, 0x4E),
            cyan: c(0x47, 0xDE, 0xF0),
            green: c(0x1A, 0xD5, 0x85),
            purple: c(0x95, 0x80, 0xFF),
            red: c(0xFF, 0x6E, 0x6E),
            ignored: c(0x6B, 0x7F, 0x8F),
            orange: c(0xFF, 0x9B, 0x4D),
            magenta: c(0xFF, 0x6E, 0xC7),
            blue: c(0x4D, 0x8C, 0xFF),
            lime: c(0xA3, 0xE6, 0x35),
            teal: c(0x2D, 0xD4, 0xBF),
            scrim: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.55,
            },
            tab_active_border: c(0xDC, 0xC9, 0xA3),
            tab_active_bg: c(0x15, 0x26, 0x30),
            tab_hover: c(0x27, 0x7a, 0x85),
            desc_bg: c(0x15, 0x26, 0x30),
        }
    }

    /// 逐一对应 `design/浅色配色表.html` 的语义色板表(23 项),与
    /// `byteboy2077()` 同源中性色相(navy `bg` #0a0e16)提亮而来，不用
    /// 暖白/暖灰，避免与 `gold`(甲方动作专属)"暖调即品牌"的印象冲突。
    pub const fn byteboy2077_light() -> Self {
        Self {
            bg: c(0xff, 0xfd, 0xf6),
            panel: c(0xfe, 0xf2, 0xe4),
            term_bg: c(0xfe, 0xf2, 0xe4),
            card: c(0xfe, 0xfd, 0xfb),
            border: c(0xd7, 0xdf, 0xe5),
            cream: c(0x16, 0x23, 0x2e),
            body: c(0x36, 0x42, 0x4e),
            dim: c(0x4c, 0x5c, 0x68),
            gold: c(0x11, 0x8b, 0x96),
            cyan: c(0x0e, 0x8a, 0x9e),
            green: c(0x12, 0x8f, 0x5a),
            purple: c(0x6a, 0x4f, 0xdb),
            red: c(0xd1, 0x48, 0x3f),
            ignored: c(0x8b, 0x98, 0xa2),
            orange: c(0xc9, 0x7a, 0x1b),
            magenta: c(0xc9, 0x3b, 0x93),
            blue: c(0x2f, 0x6f, 0xe0),
            lime: c(0x6f, 0xa8, 0x13),
            teal: c(0x0D, 0x94, 0x88),
            scrim: Color {
                r: 0x0a as f32 / 255.0,
                g: 0x0e as f32 / 255.0,
                b: 0x14 as f32 / 255.0,
                a: 0.4,
            },
            tab_active_border: c(0x65, 0x88, 0x9e),
            tab_active_bg: c(0x98, 0xb6, 0xc1),
            tab_hover: c(0xa3, 0xd2, 0xe2),
            desc_bg: c(0xea, 0xef, 0xf2),
        }
    }
}

/// 调色板来源：命名方案(Dark/Light)或任意自定义 palette。
///
/// 任意 `set_theme` 换上来的颜色不能被说成是某个内置方案；网页/终端只消费
/// 明暗提示(`scheme`)，来源由本枚举单独表达。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteSource {
    Named(ColorScheme),
    Custom,
}

/// 一次主题变更的完整快照：颜色、明暗提示、来源与单调递增的 revision。
///
/// 颜色与提示来自同一次写入，因此不会出现"新颜色 + 旧方案名"的中间态。
/// revision 供异步投递(如 WebView 注入)丢弃过期更新。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeSnapshot {
    pub colors: ColorTokens,
    /// 供网页 `color-scheme` 与终端 ANSI 主题使用的明暗提示。
    /// 命名方案直接取方案名；自定义 palette 由背景亮度推导。
    pub scheme: ColorScheme,
    pub source: PaletteSource,
    pub revision: u64,
}

impl ThemeSnapshot {
    pub fn is_light(&self) -> bool {
        self.scheme == ColorScheme::Light
    }
}

/// 单锁保存颜色与方案元数据：每次有效变更原子替换快照并递增 revision。
static SNAPSHOT: RwLock<ThemeSnapshot> = RwLock::new(ThemeSnapshot {
    colors: ColorTokens::byteboy2077(),
    scheme: ColorScheme::Dark,
    source: PaletteSource::Named(ColorScheme::Dark),
    revision: 0,
});

/// 自定义 palette 的明暗提示：按 `bg` 的相对亮度判断(阈值 0.5)。
/// 只用于兼容旧的 `current_scheme()` 与网页 `color-scheme`，不代表这是
/// 某个命名方案——来源看 [`ThemeSnapshot::source`]。
fn scheme_hint(tokens: &ColorTokens) -> ColorScheme {
    let bg = tokens.bg;
    let luminance = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
    if luminance >= 0.5 {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    }
}

fn write_snapshot(colors: ColorTokens, scheme: ColorScheme, source: PaletteSource) {
    let mut snapshot = SNAPSHOT.write().expect("byteui color RwLock poisoned");
    snapshot.colors = colors;
    snapshot.scheme = scheme;
    snapshot.source = source;
    snapshot.revision = snapshot.revision.wrapping_add(1);
}

/// 当前生效的完整主题快照。调用方应在一次视图构建里取一次，避免同一帧
/// 读到两次不同 revision 的颜色。
pub fn current_snapshot() -> ThemeSnapshot {
    *SNAPSHOT.read().expect("byteui color RwLock poisoned")
}

/// 当前快照的 revision；每次有效主题变更递增。
pub fn revision() -> u64 {
    SNAPSHOT
        .read()
        .expect("byteui color RwLock poisoned")
        .revision
}

/// 当前生效的颜色 token(默认 ByteBoy2077)。
pub fn current() -> ColorTokens {
    SNAPSHOT
        .read()
        .expect("byteui color RwLock poisoned")
        .colors
}

/// 整体替换当前颜色 token——供产品换任意自定义 palette。明暗提示按背景
/// 亮度推导，来源标记为 [`PaletteSource::Custom`]，不会冒充内置方案。
pub fn set_theme(tokens: ColorTokens) {
    write_snapshot(tokens, scheme_hint(&tokens), PaletteSource::Custom);
}

/// 当前生效的命名配色方案。`set_theme` 能换任意 `ColorTokens`，但不知道
/// 换上的是哪份"名字"；`term_model.rs` 的 ANSI16 终端色板和设置弹窗都需要
/// 一个"当前是深色还是浅色"的单一真相源，所以单独用这个枚举记名字。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

/// 当前生效的明暗提示(默认 `Dark`)。自定义 palette 返回按背景亮度推导的
/// 提示，来源请读 [`current_snapshot`] 的 `source`。
pub fn current_scheme() -> ColorScheme {
    SNAPSHOT
        .read()
        .expect("byteui color RwLock poisoned")
        .scheme
}

/// 按命名方案整体切换 token(同时更新明暗提示与来源)。设置弹窗选中
/// 即调这个,立即全局生效,不需要重启。颜色与方案名在同一次写锁里替换。
pub fn set_scheme(scheme: ColorScheme) {
    let tokens = match scheme {
        ColorScheme::Dark => ColorTokens::byteboy2077(),
        ColorScheme::Light => ColorTokens::byteboy2077_light(),
    };
    write_snapshot(tokens, scheme, PaletteSource::Named(scheme));
}

#[derive(Serialize, Deserialize)]
struct PersistedScheme {
    scheme: ColorScheme,
}

/// 把当前方案落盘到调用方指定的路径,供下次启动 `init_scheme` 读回。
/// (`byteui` 不内置任何 Dozer 专属路径约定,同 `icon_size::persist_scale`。)
pub fn persist_scheme(path: &Path) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let s = serde_json::to_string_pretty(&PersistedScheme {
        scheme: current_scheme(),
    })
    .unwrap_or_default();
    let _ = std::fs::write(path, s);
}

/// 启动时把上次退出前落盘的方案读回并应用。文件缺失/损坏都静默保持当前
/// 默认值(同 `icon_size::init_scale` 的"任何错误都不阻断主流程"定位)。
pub fn init_scheme(path: &Path) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(p) = serde_json::from_str::<PersistedScheme>(&raw) else {
        return;
    };
    set_scheme(p.scheme);
}

/// 两色按 `t`(0..=1)线性插值。`t` 超出 [0,1] 不外夹,调用方保证区间。
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

#[cfg(test)]
pub(super) fn restore_test_snapshot(snapshot: ThemeSnapshot) {
    *SNAPSHOT.write().unwrap_or_else(|e| e.into_inner()) = snapshot;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::test_support::lock_global_state;

    fn lock() -> crate::theme::test_support::GlobalStateGuard {
        lock_global_state()
    }

    #[test]
    fn byteboy2077_bg_matches_hex() {
        let t = ColorTokens::byteboy2077();
        assert_eq!(t.bg.r, 0x0d as f32 / 255.0);
        assert_eq!(t.bg.g, 0x13 as f32 / 255.0);
        assert_eq!(t.bg.b, 0x1c as f32 / 255.0);
    }

    #[test]
    fn current_defaults_to_byteboy2077() {
        let _guard = lock();
        set_theme(ColorTokens::byteboy2077());
        let c = current();
        assert_eq!(c.gold, ColorTokens::byteboy2077().gold);
    }

    #[test]
    fn set_theme_replaces_current_and_is_visible_globally() {
        let _guard = lock();
        let mut custom = ColorTokens::byteboy2077();
        custom.gold = Color::from_rgb8(0x00, 0x00, 0x00);
        set_theme(custom);
        assert_eq!(current().gold, Color::from_rgb8(0x00, 0x00, 0x00));
        // 复原,避免污染同进程里跑在本测试之后的其它测试。
        set_theme(ColorTokens::byteboy2077());
    }

    #[test]
    fn mix_at_zero_and_one_returns_endpoints() {
        let a = Color::from_rgb8(0, 0, 0);
        let b = Color::from_rgb8(255, 255, 255);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
    }

    /// 防漂移锚：`byteboy2077_light()` 的取值必须和 `design/浅色配色表.html`
    /// 的语义色板表逐项一致。
    #[test]
    fn byteboy2077_light_bg_matches_hex() {
        let t = ColorTokens::byteboy2077_light();
        assert_eq!(t.bg, Color::from_rgb8(0xff, 0xfd, 0xf6));
        assert_eq!(t.panel, Color::from_rgb8(0xfe, 0xf2, 0xe4));
        assert_eq!(t.gold, Color::from_rgb8(0x11, 0x8b, 0x96));
        assert_eq!(t.cream, Color::from_rgb8(0x16, 0x23, 0x2e));
    }

    #[test]
    fn current_scheme_defaults_to_dark() {
        let _guard = lock();
        set_scheme(ColorScheme::Dark);
        assert_eq!(current_scheme(), ColorScheme::Dark);
    }

    #[test]
    fn set_scheme_switches_tokens_and_reports_current_scheme() {
        let _guard = lock();
        set_scheme(ColorScheme::Light);
        assert_eq!(current_scheme(), ColorScheme::Light);
        assert_eq!(current().bg, ColorTokens::byteboy2077_light().bg);
        // 复原，避免污染同进程里跑在本测试之后的其它测试。
        set_scheme(ColorScheme::Dark);
        assert_eq!(current().bg, ColorTokens::byteboy2077().bg);
    }

    #[test]
    fn persist_scheme_and_init_scheme_roundtrip_via_explicit_path() {
        let _guard = lock();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("color_theme.json");

        set_scheme(ColorScheme::Light);
        persist_scheme(&path);
        set_scheme(ColorScheme::Dark);

        init_scheme(&path);
        assert_eq!(current_scheme(), ColorScheme::Light);

        // 复原，避免污染同进程里跑在本测试之后的其它测试。
        set_scheme(ColorScheme::Dark);
    }

    #[test]
    fn init_scheme_with_missing_or_corrupted_file_keeps_current_scheme() {
        let _guard = lock();
        set_scheme(ColorScheme::Dark);
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.json");
        init_scheme(&missing);
        assert_eq!(current_scheme(), ColorScheme::Dark);

        let bad = dir.path().join("color_theme.json");
        std::fs::write(&bad, "not json").unwrap();
        init_scheme(&bad);
        assert_eq!(current_scheme(), ColorScheme::Dark);
    }

    /// 快照一致性:颜色、明暗提示、来源与 revision 来自同一次写入。
    #[test]
    fn snapshot_is_consistent_and_revision_monotonic() {
        let _guard = lock();
        set_scheme(ColorScheme::Dark);
        let before = current_snapshot();

        set_scheme(ColorScheme::Light);
        let after = current_snapshot();
        assert_eq!(after.colors, ColorTokens::byteboy2077_light());
        assert_eq!(after.scheme, ColorScheme::Light);
        assert_eq!(after.source, PaletteSource::Named(ColorScheme::Light));
        assert_eq!(after.revision, before.revision + 1);
        // 同一个快照里的颜色与提示必须一致,不存在"新颜色 + 旧方案名"。
        assert_eq!(current(), after.colors);
        assert_eq!(current_scheme(), after.scheme);
        assert_eq!(revision(), after.revision);

        set_scheme(ColorScheme::Dark);
    }

    /// 任意自定义 palette 不能被说成命名方案;明暗提示按背景亮度给出。
    #[test]
    fn custom_palette_is_marked_custom_and_hints_by_luminance() {
        let _guard = lock();
        set_scheme(ColorScheme::Dark);

        let mut dark_custom = ColorTokens::byteboy2077();
        dark_custom.bg = Color::from_rgb8(0x10, 0x10, 0x10);
        set_theme(dark_custom);
        let snap = current_snapshot();
        assert_eq!(snap.source, PaletteSource::Custom);
        assert_eq!(snap.scheme, ColorScheme::Dark);
        assert_eq!(snap.colors.bg, dark_custom.bg);

        let mut light_custom = ColorTokens::byteboy2077();
        light_custom.bg = Color::from_rgb8(0xf5, 0xf5, 0xf5);
        set_theme(light_custom);
        let snap = current_snapshot();
        assert_eq!(snap.source, PaletteSource::Custom);
        assert_eq!(snap.scheme, ColorScheme::Light);
        assert!(snap.is_light());
        assert_eq!(snap.colors.bg, light_custom.bg);

        set_scheme(ColorScheme::Dark);
    }
}

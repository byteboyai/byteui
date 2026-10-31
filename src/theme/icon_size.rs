//! 图标尺寸 token 化：`ByteBoy2077` 是编译期默认值，`set_theme` 可在
//! 运行时整体替换成另一份产品的取值（同 `theme::color`/`theme::font`/
//! `theme::geometry` 的模式）——组件内部一律读 `current()`，不直接引用
//! `byteboy2077()`。这里只管控件内部图标的"设计基准尺寸"，不越界。
//!
//! 本模块所有尺寸 accessor（`rail`/`row`/`chevron`/`tree_row_gap`）返回的值
//! 都已乘过 `scale()`，因此改 `scale` 即整体缩放全部图标与图标相关间距
//! （一个旋钮控制全局）；`icons::view` 是纯渲染入口，不再二次乘 scale。
//! 改 `rail/row/chevron/tree_row_gap` 则只调某类位置的相对大小。
//!
//! `scale()` 是**运行时可变**的：启动默认值取 `IconSizeTokens.scale`
//! （见下方 `current().scale`），可被环境变量 `DOZER_ICON_SCALE` 覆盖；
//! 运行时由 `set_scale` / `zoom_by`（Ctrl + / Ctrl - 快捷键入口）改写，
//! 下一帧布局即按新值重排——所有 accessor 每帧都实时读 `scale()`，不
//! 缓存缩放结果。**这套运行时缩放机制和 `IconSizeTokens`（静态设计基准
//! 尺寸）是两回事，不要混淆**：`set_theme` 换的是"设计基准值"，
//! `set_scale`/`zoom_by` 改的是"运行时倍数"，两者独立正交。
//!
//! 改过的 scale 需要**跨重启保留**：用户在会话里放大/缩小后退出，下次重开
//! 应回到退出时的 scale。`init_scale`/`persist_scale`/`reset_scale` 都改吃
//! 调用方传入的落盘路径（`byteui` 不内置任何 Dozer 专属路径约定）；环境
//! 变量 `DOZER_ICON_SCALE` 是显式覆盖，优先级高于落盘值（见 `init_scale`）。
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct IconSizeTokens {
    /// 主导航图标栏按钮(左/右 rail)与顶栏设置齿轮：16x16。
    pub rail: f32,
    /// 文件树行 / 右键菜单项 / tab 箭头 / 最大化按钮等：14x14。
    pub row: f32,
    /// 文件树展开/收起箭头：12x12（比同行文件图标略小）。
    pub chevron: f32,
    /// 面板 tab 栏翻页箭头（`<` / `>`）：10x10，比文件树 chevron 略小。
    pub tab_arrow: f32,
    /// 顶栏 "Dozer Home" tab 的品牌图标(house)：12x12。
    pub home: f32,
    /// 文件树行内"箭头↔图标"之间的间距（设计基准 2px）。
    pub tree_row_gap: f32,
    /// 设计基准缩放因子：1.0 = 设计基准；作为 `base_scale()` 的兜底值，
    /// 和运行时可变的 `CURRENT_SCALE` 是两回事。
    pub scale: f32,
}

impl IconSizeTokens {
    /// ByteBoy2077 的取值（与应用侧 JSON 的 `icon_sizes` 节点同名字段一致），
    /// 仅作未显式 `set_theme()` 时的兜底默认值。
    pub const fn byteboy2077() -> Self {
        Self {
            rail: 16.0,
            row: 14.0,
            chevron: 12.0,
            tab_arrow: 9.0,
            home: 12.0,
            tree_row_gap: 2.0,
            scale: 1.0,
        }
    }
}

static CURRENT: RwLock<IconSizeTokens> = RwLock::new(IconSizeTokens::byteboy2077());

/// 当前生效的图标尺寸 token（默认 ByteBoy2077）。
pub fn current() -> IconSizeTokens {
    *CURRENT.read().expect("byteui icon_size RwLock poisoned")
}

/// 整体替换当前图标尺寸 token——供调用方（如应用启动时的 `theme::init()`）
/// 在启动时用自己的 `workspace.json` 覆盖默认值。**不影响**运行时缩放
/// 倍数（`CURRENT_SCALE`），那是独立机制，见模块文档。
pub fn set_theme(tokens: IconSizeTokens) {
    *CURRENT.write().expect("byteui icon_size RwLock poisoned") = tokens;
}

pub fn rail() -> f32 {
    current().rail * scale()
}
pub fn row() -> f32 {
    current().row * scale()
}
pub fn chevron() -> f32 {
    current().chevron * scale()
}
/// 面板 tab 栏翻页箭头（`<` / `>`）尺寸，已含全局 scale。
pub fn tab_arrow() -> f32 {
    current().tab_arrow * scale()
}
/// 顶栏 "Dozer Home" tab 品牌图标尺寸，已含全局 scale。
pub fn home() -> f32 {
    current().home * scale()
}
/// 文件树行内"箭头↔图标"间距，已含全局 scale。
pub fn tree_row_gap() -> f32 {
    current().tree_row_gap * scale()
}

/// 全局缩放因子的运行时当前值（逻辑像素倍数）。`u32::MAX` 是哨兵，表示
/// "尚未被运行时改写"，此时回落到启动基准（环境覆盖或 token 基准值）。
static CURRENT_SCALE: AtomicU32 = AtomicU32::new(u32::MAX);

/// `init_scale` 时解析出的有效环境覆盖；`u32::MAX` 表示没有覆盖。
/// 单独保存它，`reset_scale` 才能回到"本次启动的环境覆盖"而不是一律回到
/// token 基准。值在初始化时已规范化，之后不再读进程环境。
static STARTUP_ENV_SCALE: AtomicU32 = AtomicU32::new(u32::MAX);

/// 是否已经执行过 `init_scale`。初始化后 `scale()` 不再逐次读环境变量。
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 环境变量覆盖：`BYTEUI_ICON_SCALE` 优先；`DOZER_ICON_SCALE` 是 Dozer 早期
/// 使用的名字，仍然兼容（已有用户的设置不失效）。
const ENV_SCALE: &str = "BYTEUI_ICON_SCALE";
const ENV_SCALE_LEGACY: &str = "DOZER_ICON_SCALE";

/// 所有缩放入口共用的纯校验函数。
///
/// 非有限数（NaN/±∞）、零和负数都无效，返回 `None`；有限正数钳制到
/// `[SCALE_MIN, SCALE_MAX]`。调用方拿到 `None` 时必须保留原有效值，不能
/// 写入状态。
pub fn normalize_scale(value: f32) -> Option<f32> {
    if value.is_finite() && value > 0.0 {
        Some(value.clamp(SCALE_MIN, SCALE_MAX))
    } else {
        None
    }
}

/// 纯函数，便于不碰进程环境地测试：非数字、0、负数都当作"没设"；
/// 有效正数统一钳制到 `[SCALE_MIN, SCALE_MAX]`（环境变量 `99` 不再直接生效）。
fn pick_env_scale(primary: Option<String>, legacy: Option<String>) -> Option<f32> {
    let parse = |v: Option<String>| {
        v.and_then(|s| s.parse::<f32>().ok())
            .and_then(normalize_scale)
    };
    parse(primary).or_else(|| parse(legacy))
}

fn env_scale_override() -> Option<f32> {
    pick_env_scale(
        std::env::var(ENV_SCALE).ok(),
        std::env::var(ENV_SCALE_LEGACY).ok(),
    )
}

/// token 基准值也要经过同一套校验；手改 JSON 写入 NaN/0/负数时回落到 1.0，
/// 而不是把非法值传播到所有图标和几何尺寸。
fn token_base_scale() -> f32 {
    normalize_scale(current().scale).unwrap_or(1.0)
}

/// 启动基准 scale：初始化后只用已物化的环境覆盖或 token 基准；
/// 未初始化（旧调用路径）才临时读取环境变量，保持兼容。
fn base_scale() -> f32 {
    if INITIALIZED.load(Ordering::Relaxed) {
        let bits = STARTUP_ENV_SCALE.load(Ordering::Relaxed);
        if bits == u32::MAX {
            token_base_scale()
        } else {
            f32::from_bits(bits)
        }
    } else {
        env_scale_override().unwrap_or_else(token_base_scale)
    }
}

/// 全局缩放因子：所有 token accessor 都会乘它，因此改这一个值即整体缩放
/// 全部图标与图标相关间距（一个旋钮控制全局）。运行时可变——见
/// `set_scale` / `zoom_by`。
pub fn scale() -> f32 {
    let bits = CURRENT_SCALE.load(Ordering::Relaxed);
    if bits == u32::MAX {
        base_scale()
    } else {
        f32::from_bits(bits)
    }
}

/// 把全局 scale 直接设为目标值。有效值钳制到 `[SCALE_MIN, SCALE_MAX]`；
/// NaN/±∞/0/负数被忽略，保留原有效值。下一帧布局自动按新值重排。
pub fn set_scale(target: f32) {
    if let Some(value) = normalize_scale(target) {
        CURRENT_SCALE.store(value.to_bits(), Ordering::Relaxed);
    }
}

/// 相对缩放（Ctrl + / Ctrl - 的入口）：在当前值基础上乘 `factor`。
/// 非有限/非正乘数以及乘积溢出都不写入状态。
pub fn zoom_by(factor: f32) {
    if !factor.is_finite() || factor <= 0.0 {
        return;
    }
    // 先确认乘积有限，再交给 `set_scale` 钳制；0 * inf 之类也不会写状态。
    let target = scale() * factor;
    if target.is_finite() {
        set_scale(target);
    }
}

/// 还原到本次启动的基准 scale（Ctrl+1 入口）：若启动时有有效环境覆盖，
/// 回到该覆盖；否则清空运行时改写，回落到 token 基准（`set_theme` 之后
/// 仍跟随新的 token `scale`，不会被启动时的旧值冻结）。
/// 同时把落盘值复位成出厂默认，使"还原"在下次重启后依然生效。
pub fn reset_scale(path: &Path) {
    let startup_bits = STARTUP_ENV_SCALE.load(Ordering::Relaxed);
    CURRENT_SCALE.store(startup_bits, Ordering::Relaxed);
    save_persisted_scale(path, token_base_scale());
}

/// 启动时把上次退出前落盘的 scale 读回并应用为当前值,调用方传入落盘路径
/// (`byteui` 不内置任何 Dozer 专属路径约定)。环境变量 `BYTEUI_ICON_SCALE`
/// (旧名 `DOZER_ICON_SCALE`)是显式覆盖,优先级高于落盘值。
pub fn init_scale(path: &Path) {
    init_scale_with(
        path,
        std::env::var(ENV_SCALE).ok(),
        std::env::var(ENV_SCALE_LEGACY).ok(),
    );
}

/// `init_scale` 的可测版本：环境变量值由参数传入，不碰进程环境。
/// 有效环境值优先并**物化**到运行时状态（初始化后 accessor 不再读环境）；
/// 非法值等同没设，继续读落盘值；两者都没有时保留 token 基准哨兵。
fn init_scale_with(path: &Path, primary: Option<String>, legacy: Option<String>) {
    STARTUP_ENV_SCALE.store(u32::MAX, Ordering::Relaxed);
    if let Some(v) = pick_env_scale(primary, legacy) {
        STARTUP_ENV_SCALE.store(v.to_bits(), Ordering::Relaxed);
        CURRENT_SCALE.store(v.to_bits(), Ordering::Relaxed);
    } else if let Some(v) = load_persisted_scale(path) {
        CURRENT_SCALE.store(v.to_bits(), Ordering::Relaxed);
    } else {
        CURRENT_SCALE.store(u32::MAX, Ordering::Relaxed);
    }
    INITIALIZED.store(true, Ordering::Relaxed);
}

/// 把当前 scale 落盘到调用方指定的路径,供下次启动 `init_scale` 读回。
pub fn persist_scale(path: &Path) {
    save_persisted_scale(path, scale());
}

#[derive(Serialize, Deserialize)]
struct PersistedScale {
    scale: f32,
}

/// 读存盘 scale：文件缺失/损坏/越界/非有限数都回落 `None`（调用方保持默认）。
fn load_persisted_scale(path: &Path) -> Option<f32> {
    load_from(path)
}

/// 写存盘 scale：目录不存在先建；任何 IO 失败静默（缩放是体验增强，不阻断
/// 主流程，任何错误都静默）。
fn save_persisted_scale(path: &Path, v: f32) {
    let _ = save_to(path, v);
}

/// 显式路径读取，供单测指向临时文件，不碰用户真实配置目录。
/// 越界值夹回 `[SCALE_MIN, SCALE_MAX]`（手改过的大数不至于让 UI 失控），
/// 非有限数/非正数视为无效，回落 `None`。
pub(crate) fn load_from(path: &Path) -> Option<f32> {
    let raw = std::fs::read_to_string(path).ok()?;
    let p: PersistedScale = serde_json::from_str(&raw).ok()?;
    normalize_scale(p.scale)
}

/// 显式路径写入，供单测指向临时文件，不碰用户真实配置目录。
pub(crate) fn save_to(path: &Path, v: f32) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let s = serde_json::to_string_pretty(&PersistedScale { scale: v }).unwrap_or_default();
    std::fs::write(path, s)
}

/// UI 缩放允许的下限/上限（逻辑像素倍数），防止缩到不可读或放到失控。
pub const SCALE_MIN: f32 = 0.5;
pub const SCALE_MAX: f32 = 3.0;

#[cfg(test)]
pub(super) struct TestSnapshot {
    tokens: IconSizeTokens,
    scale: u32,
    startup: u32,
    initialized: bool,
}

#[cfg(test)]
pub(super) fn test_snapshot() -> TestSnapshot {
    TestSnapshot {
        tokens: current(),
        scale: CURRENT_SCALE.load(Ordering::Relaxed),
        startup: STARTUP_ENV_SCALE.load(Ordering::Relaxed),
        initialized: INITIALIZED.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
pub(super) fn restore_test_snapshot(snapshot: TestSnapshot) {
    set_theme(snapshot.tokens);
    CURRENT_SCALE.store(snapshot.scale, Ordering::Relaxed);
    STARTUP_ENV_SCALE.store(snapshot.startup, Ordering::Relaxed);
    INITIALIZED.store(snapshot.initialized, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::test_support::lock_global_state;

    /// 与 color/region 的测试共用同一把进程级锁。
    fn lock() -> crate::theme::test_support::GlobalStateGuard {
        lock_global_state()
    }

    /// 把共享状态复位到"未初始化、无运行时改写、默认 token"，跑完也必须再调
    /// 一次，避免污染同进程里的其它测试（panic 场景由 `lock` 的 poison 恢复兜底）。
    fn reset_global_state() {
        CURRENT_SCALE.store(u32::MAX, Ordering::Relaxed);
        STARTUP_ENV_SCALE.store(u32::MAX, Ordering::Relaxed);
        INITIALIZED.store(false, Ordering::Relaxed);
        set_theme(IconSizeTokens::byteboy2077());
    }

    /// 锁值快照：`byteboy2077()` 的每个字段值不得随意改动；
    /// 改动必须同步确认所有消费方。
    #[test]
    fn byteboy2077_locked_values() {
        let t = IconSizeTokens::byteboy2077();
        assert_eq!(t.rail, 16.0);
        assert_eq!(t.row, 14.0);
        assert_eq!(t.chevron, 12.0);
        assert_eq!(t.tab_arrow, 9.0);
        assert_eq!(t.home, 12.0);
        assert_eq!(t.tree_row_gap, 2.0);
        assert_eq!(t.scale, 1.0);
    }

    #[test]
    fn current_defaults_to_byteboy2077() {
        assert_eq!(current().rail, IconSizeTokens::byteboy2077().rail);
    }

    #[test]
    fn set_theme_replaces_current_and_is_visible_globally() {
        let _guard = lock();
        reset_global_state();
        let mut custom = IconSizeTokens::byteboy2077();
        custom.rail = 999.0;
        set_theme(custom);
        assert_eq!(current().rail, 999.0);
        reset_global_state();
    }

    #[test]
    fn accessors_reflect_current_at_default_scale() {
        let _guard = lock();
        reset_global_state();
        assert_eq!(rail(), 16.0);
        assert_eq!(row(), 14.0);
        assert_eq!(chevron(), 12.0);
        assert_eq!(tab_arrow(), 9.0);
        reset_global_state();
    }

    // ---- normalize_scale：所有入口共用的一套校验 ----

    #[test]
    fn normalize_scale_rejects_non_finite_and_non_positive() {
        assert_eq!(normalize_scale(f32::NAN), None);
        assert_eq!(normalize_scale(f32::INFINITY), None);
        assert_eq!(normalize_scale(f32::NEG_INFINITY), None);
        assert_eq!(normalize_scale(0.0), None);
        assert_eq!(normalize_scale(-0.0), None);
        assert_eq!(normalize_scale(-1.0), None);
    }

    #[test]
    fn normalize_scale_clamps_finite_positive_values() {
        assert_eq!(normalize_scale(1.0), Some(1.0));
        assert_eq!(normalize_scale(0.01), Some(SCALE_MIN));
        assert_eq!(normalize_scale(99.0), Some(SCALE_MAX));
        assert_eq!(normalize_scale(f32::MIN_POSITIVE), Some(SCALE_MIN));
        assert_eq!(normalize_scale(f32::MAX), Some(SCALE_MAX));
    }

    #[test]
    fn set_scale_clamps_and_ignores_invalid_input() {
        let _guard = lock();
        reset_global_state();

        set_scale(2.0);
        assert_eq!(scale(), 2.0);

        // 非法输入保留原值，不写状态。
        set_scale(f32::NAN);
        assert_eq!(scale(), 2.0);
        set_scale(f32::INFINITY);
        assert_eq!(scale(), 2.0);
        set_scale(0.0);
        assert_eq!(scale(), 2.0);
        set_scale(-3.0);
        assert_eq!(scale(), 2.0);

        // 越界正数钳制。
        set_scale(99.0);
        assert_eq!(scale(), SCALE_MAX);
        set_scale(0.1);
        assert_eq!(scale(), SCALE_MIN);

        reset_global_state();
    }

    #[test]
    fn zoom_by_rejects_invalid_factors_and_overflow() {
        let _guard = lock();
        reset_global_state();
        set_scale(2.0);

        zoom_by(f32::NAN);
        assert_eq!(scale(), 2.0);
        zoom_by(f32::INFINITY);
        assert_eq!(scale(), 2.0);
        zoom_by(0.0);
        assert_eq!(scale(), 2.0);
        zoom_by(-1.0);
        assert_eq!(scale(), 2.0);

        // 乘法溢出（inf）不写入状态。
        zoom_by(f32::MAX);
        assert_eq!(scale(), 2.0);

        // 正常乘数仍然生效，并在边界钳制。
        zoom_by(1.5);
        assert_eq!(scale(), 3.0);
        zoom_by(10.0);
        assert_eq!(scale(), SCALE_MAX);

        reset_global_state();
    }

    #[test]
    fn invalid_token_base_scale_falls_back_to_one() {
        let _guard = lock();
        reset_global_state();
        for bad in [f32::NAN, f32::INFINITY, 0.0, -2.0] {
            let mut t = IconSizeTokens::byteboy2077();
            t.scale = bad;
            set_theme(t);
            assert_eq!(scale(), 1.0, "token scale {bad} 应回落到 1.0");
        }
        reset_global_state();
    }

    /// 落盘 round-trip：写出去的值读回来和写的一致，且会夹进合法范围。
    #[test]
    fn persisted_scale_round_trips_and_clamps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ui_scale.json");

        save_to(&path, 1.5).unwrap();
        assert_eq!(load_from(&path), Some(1.5));

        // 越界值落盘后被夹回合法范围（写入时不夹，读取时夹）。
        save_to(&path, 99.0).unwrap();
        assert_eq!(load_from(&path), Some(SCALE_MAX));
        save_to(&path, 0.01).unwrap();
        assert_eq!(load_from(&path), Some(SCALE_MIN));
    }

    /// 损坏/缺失/非有限数文件都回落 `None`，不污染默认 scale。
    #[test]
    fn corrupted_or_missing_scale_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.json");
        assert_eq!(load_from(&missing), None);

        let bad = dir.path().join("ui_scale.json");
        std::fs::write(&bad, "not json").unwrap();
        assert_eq!(load_from(&bad), None);

        std::fs::write(&bad, r#"{"scale": "x"}"#).unwrap();
        assert_eq!(load_from(&bad), None);

        std::fs::write(&bad, r#"{"scale": NaN}"#).unwrap();
        assert_eq!(load_from(&bad), None);
    }

    /// 用临时目录里的 `ui_scale.json` 当落盘路径，隔离用户真实配置目录；
    /// 跑完复位全局状态，避免污染同进程里之后的测试。
    fn with_temp_scale_file<F: FnOnce(&Path)>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ui_scale.json");
        f(&path);
        reset_global_state();
    }

    /// `init_scale` 把落盘值应用成当前 scale；`reset_scale` 清除运行时改写
    /// 并落盘复位成出厂默认。两者都吃显式临时路径，不再碰用户真实配置目录。
    #[test]
    fn init_applies_persisted_and_reset_clears_it() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            save_to(path, 2.0).unwrap();
            init_scale_with(path, None, None);
            assert_eq!(scale(), 2.0);

            reset_scale(path);
            assert_eq!(scale(), current().scale);
            assert_eq!(load_from(path), Some(current().scale));
        });
    }

    #[test]
    fn pick_env_scale_prefers_byteui_name() {
        assert_eq!(
            pick_env_scale(Some("1.5".into()), Some("2.0".into())),
            Some(1.5)
        );
    }

    #[test]
    fn pick_env_scale_falls_back_to_legacy_name() {
        assert_eq!(pick_env_scale(None, Some("2.0".into())), Some(2.0));
    }

    #[test]
    fn pick_env_scale_ignores_invalid_values() {
        // 非数字、0、负数、NaN、Infinity 都当作"没设"。
        assert_eq!(pick_env_scale(Some("abc".into()), None), None);
        assert_eq!(pick_env_scale(Some("0".into()), None), None);
        assert_eq!(pick_env_scale(Some("-1".into()), None), None);
        assert_eq!(pick_env_scale(Some("NaN".into()), None), None);
        assert_eq!(pick_env_scale(Some("inf".into()), None), None);
        // 主名非法时仍可回落到旧名。
        assert_eq!(
            pick_env_scale(Some("abc".into()), Some("2.0".into())),
            Some(2.0)
        );
    }

    #[test]
    fn pick_env_scale_clamps_out_of_range_values() {
        assert_eq!(pick_env_scale(Some("99".into()), None), Some(SCALE_MAX));
        assert_eq!(pick_env_scale(Some("0.01".into()), None), Some(SCALE_MIN));
        // 主名越界但有效时优先于旧名。
        assert_eq!(
            pick_env_scale(Some("99".into()), Some("1.5".into())),
            Some(SCALE_MAX)
        );
    }

    /// 环境变量是非法值（等同没设）时，仍然要读回用户保存的缩放。
    #[test]
    fn init_with_invalid_env_still_loads_persisted() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            save_to(path, 2.0).unwrap();
            init_scale_with(path, Some("abc".into()), Some("0".into()));
            assert_eq!(scale(), 2.0);
        });
    }

    /// 环境变量是有效值时，覆盖落盘值，并**物化**到运行时状态；
    /// 初始化后 accessor 不再读进程环境。
    #[test]
    fn init_with_valid_env_overrides_and_materializes() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            save_to(path, 2.0).unwrap();
            init_scale_with(path, Some("1.5".into()), None);
            assert_eq!(
                CURRENT_SCALE.load(Ordering::Relaxed),
                1.5f32.to_bits(),
                "有效环境值必须物化，不能留哨兵让 accessor 反复读环境"
            );
            assert_eq!(scale(), 1.5);
            assert_eq!(STARTUP_ENV_SCALE.load(Ordering::Relaxed), 1.5f32.to_bits());
        });
    }

    /// 环境值越界时按统一规则钳制后物化。
    #[test]
    fn init_with_out_of_range_env_clamps() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            init_scale_with(path, Some("99".into()), None);
            assert_eq!(scale(), SCALE_MAX);
        });
    }

    /// `reset_scale` 回到本次启动的环境覆盖，而不是一律回到 token 基准。
    #[test]
    fn reset_restores_startup_env_override() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            init_scale_with(path, Some("1.5".into()), None);
            set_scale(2.5);
            reset_scale(path);
            assert_eq!(scale(), 1.5);
            // 落盘仍是"出厂基准"，下次启动没有环境覆盖时回到 token 值。
            assert_eq!(load_from(path), Some(current().scale));
        });
    }

    /// 没有环境覆盖时，`reset_scale` 跟随运行时替换后的 token 基准，
    /// 不会被启动时的旧 token 值冻结。
    #[test]
    fn reset_follows_runtime_theme_token_base() {
        let _guard = lock();
        reset_global_state();
        with_temp_scale_file(|path| {
            init_scale_with(path, None, None);
            set_scale(2.0);

            let mut t = IconSizeTokens::byteboy2077();
            t.scale = 1.25;
            set_theme(t);

            reset_scale(path);
            assert_eq!(scale(), 1.25);
            assert_eq!(load_from(path), Some(1.25));
        });
    }

    /// 未调用 `init_scale` 的旧路径仍然可用：环境覆盖优先于 token 基准。
    #[test]
    fn uninitialized_scale_keeps_legacy_env_fallback() {
        let _guard = lock();
        reset_global_state();
        // 未初始化时 `base_scale()` 读进程环境；这里没有设置环境变量，
        // 因此等于 token 基准。兼容性由 `env_scale_override` 的既有测试覆盖。
        assert_eq!(scale(), 1.0);
        let mut t = IconSizeTokens::byteboy2077();
        t.scale = 1.5;
        set_theme(t);
        assert_eq!(scale(), 1.5);
        reset_global_state();
    }
}

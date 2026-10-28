//! 面板内 tab 栏的共享部件(原 Dozer `chrome/tab_widget.rs`;Digger 的 `agent_tabs.rs`/`tab_chrome.rs`
//! 各抄过一份,X1 起三处共用这一份):`panel_tab`(单个 tab 渲染器)、`tab_label`(横向 tab 与下拉行共用的
//! "前缀 + 标题")、`tab_container_style`、`tab_overflow_button`/`tab_overflow_menu`(V 溢出下拉入口与悬浮菜单,
//! 列出某 tab 组内全部 tab)、`tab_overflow_items`(macOS 原生菜单版本)。
//!
//! 悬停进度 `hover_t` 由调用方给:Dozer 传动画插值(0..=1),Digger 传阶跃值(0/1),部件只吃数值。
//! 窗口化滚动算法 `tab_window`/`tab_window_reveal` 在 [`crate::interaction::tab_strip`]。
//!
//! **不在这里的**:预览/代码、网格/原文、JSON 视图三个模式切换按钮是 Dozer 预览专属(两个依赖 Dozer 的
//! 国际化),留在 Dozer `chrome/tab_widget.rs`。

use crate::feedback::tooltip::controlled_tooltip;
use crate::interaction::{icons, tabs};
use crate::theme::font::top_bar_font;
use iced_widget::core::{Border, Color, Element, Length, Padding};
use iced_widget::tooltip::Position;
use iced_widget::{MouseArea, column, container, row, scrollable, stack, text};

/// 悬停组合值:悬停标题或 × 任一,胶囊背景都浮现、× 显形。夹在 0..=1。
pub fn combined_hover(hover: f32, close_hover: f32) -> f32 {
    hover.max(close_hover).clamp(0.0, 1.0)
}

/// 标题颜色:选中恒 `cream`;未选中在 `dim`→`gold` 之间按 `hover` 插值。
pub fn title_color(active: bool, hover: f32) -> Color {
    let colors = crate::theme::color::current();
    if active {
        colors.cream
    } else {
        crate::theme::color::mix(colors.dim, colors.gold, hover.clamp(0.0, 1.0))
    }
}

/// 关闭按钮颜色:静止时透明(不可见),悬停时 alpha = `hover`,颜色按 `close_hover` 从 `dim` 到 `gold`。
pub fn close_color(hover: f32, close_hover: f32) -> Color {
    let colors = crate::theme::color::current();
    Color {
        a: hover.clamp(0.0, 1.0),
        ..crate::theme::color::mix(colors.dim, colors.gold, close_hover.clamp(0.0, 1.0))
    }
}

/// 溢出菜单宽度:可用宽度 = 窗口宽 − 锚点 x − 24,夹在 [220, 560]。**不随 UI 缩放**(Dozer 现状);
/// Digger 原先的 `overflow_geometry` 把三个常量都乘了 scale,scale ≠ 1 时与此不同,迁移时以这里为准。
pub fn tab_overflow_menu_width(anchor_x: f32, window_w: f32) -> f32 {
    (window_w - anchor_x - 24.0).clamp(TAB_OVERFLOW_MENU_MIN_WIDTH, TAB_OVERFLOW_MENU_MAX_WIDTH)
}

/// 面板 tab 内边距:横向留白给 hover 胶囊,纵向收紧以缩小高度。左侧单独
/// 放大(原先与右侧同为 4,标题贴左缘太紧),右侧维持贴近关闭按钮的窄距。
/// 上下 `PANEL_TAB_PAD_Y` 从 1 收到 0(验收反馈:tab 栏分割线要跟左栏
/// header 分割线对齐,tab 栏整体降 2px 更贴近 header 那侧的基线)。
pub const PANEL_TAB_PAD_LEFT: f32 = 10.0;
pub const PANEL_TAB_PAD_X: f32 = 4.0;
pub const PANEL_TAB_PAD_Y: f32 = 0.0;

/// 传给 `tab_label::max_width` 表示"不设上限"的名目值——面板 tab 随标题实际
/// 内容伸缩,标题区用不上限宽参数 (`tab_label` 仅在下拉行需要按行宽裁列)。
pub const NO_TAB_W_LIMIT: f32 = f32::MAX;

/// 面板内 tab（终端 / 预览 / 浏览器三处共用）的渲染器，样式对齐顶栏未选中
/// 页签：标题 `body()`(13px) + `top_bar_font()`，静止 `DIM`、hover 动画
/// `DIM→金`；关闭 `×` 静止 `DIM`、hover `DIM→金`、24×24 命中框；未选中
/// hover 显 `TAB_HOVER` 胶囊背景（radius 8）。tab 宽度随标题实际内容伸缩
/// (`Length::Shrink`)，不设上限——标题超宽即在所在行多余空间放不下时被外层
/// `.clip` 截断（不换行、不补省略号），整行放得下时完整展示全称。激活态外观
/// (CREAM 标题 + CARD 实底 + 1px 边框)由本函数统一绘制，未选中态额外画
/// hover 细节。
///
/// 泛型 over 消息类型 `M`：终端/预览传 `app::Message`，浏览器传
/// `browser::Message`，保证三处渲染完全一致。`hover_t`/`close_hover_t` 是
/// 调用方动画源给的插值进度(0..=1)；`prefix` 承载终端状态点(标题左侧)，
/// `suffix` 承载预览编辑图标(标题右侧、关闭按钮前，仍是独立可点元素)。
/// `show_tooltip` 由调用方按"悬停满 2s"算好(`App::hover_tooltip_ready` /
/// `browser::State::hover_tooltip_ready`)，满则包一层 tooltip 显示标题全称。
// 共享的 panel tab 渲染器,被终端/预览/browser/database 多处跨模块复用。
// 曾因"拆结构体要引入 `Box<dyn Fn>`,得不偿失"没做 Builder 化——实际上
// 把两个闭包做成结构体自己的泛型参数(而非 trait object)就不用装箱,
// `PanelTabArgs` 就是这样做的:`on_select`/`on_close` 同为 `M`、
// `title_hover`/`close_hover`/`prefix`/`suffix` 各自同类型相邻,原先 11
// 个位置参数顺序传错编译器发现不了(Rust Design Patterns:Builder,用
// 具名字段替代同类型位置参数)。
/// tab 的"图标+文字"渲染,横向 tab(`panel_tab`)与下拉行(`tab_overflow_menu`)
/// 共用。`active`/`hover_t` 决定标题颜色(选中恒 CREAM,未选中 DIM→GOLD
/// 按 `hover_t` 插值,与原 `panel_tab` 配色公式一致)。
pub fn tab_label<'a, M: 'a>(
    prefix: Option<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>>,
    title: String,
    active: bool,
    hover_t: f32,
    max_width: f32,
) -> Element<'a, M, iced_widget::Theme, iced_renderer::Renderer> {
    let title_color = title_color(active, hover_t);
    let mut title_row = row![]
        .spacing(4)
        .align_y(iced_widget::core::Alignment::Center);
    if let Some(p) = prefix {
        title_row = title_row.push(p);
    }
    title_row = title_row.push(
        container(
            text(title)
                .font(top_bar_font())
                .size(crate::theme::font::body())
                // iced `Text` 默认 `Wrapping::Word`,不是曾经以为的
                // `None`——不显式关掉,标题超宽时会真的折成两行,而不是
                // 靠下面 `.clip(true)` 单行截断(验收反馈:较长标题换行)。
                .wrapping(iced_widget::core::text::Wrapping::None)
                .color(title_color),
        )
        // 超宽不补省略号、不换行:`max_width` 只对下拉行(有行宽预算)起到
        // 截列作用;面板横向 tab 传 `NO_TAB_W_LIMIT`(≈无限)则不触发截列,
        // 标题完整展示,由外层行的 `.clip` 决定哪里裁掉。
        .width(Length::Shrink)
        .max_width(max_width)
        .clip(true),
    );
    title_row.into()
}

/// tab 容器的背景/边框:选中态 CARD 实底+1px 边框,未选中 hover 时
/// TAB_HOVER 胶囊背景,都不是则透明。从 `panel_tab` 抽取,`tab_overflow_menu`
/// 的行高亮复用同一份判断,避免两处各写一份容易分叉的样式逻辑。
pub fn tab_container_style(
    active: bool,
    hover: f32,
) -> impl Fn(&iced_widget::Theme) -> container::Style {
    move |_theme: &iced_widget::Theme| {
        if active {
            container::Style {
                background: Some(crate::theme::color::current().card.into()),
                border: Border {
                    color: crate::theme::color::current().border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..container::Style::default()
            }
        } else if hover > 0.0 {
            // 悬停胶囊铺满整片 tab(含 × 区),× 落在其内部。
            container::Style {
                background: Some(
                    Color {
                        a: hover,
                        ..crate::theme::color::current().tab_hover
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        } else {
            container::Style::default()
        }
    }
}

pub struct PanelTabArgs<'a, M, F1, F2>
where
    M: Clone + 'a,
    F1: Fn(bool) -> M + 'a,
    F2: Fn(bool) -> M + 'a,
{
    pub title: String,
    pub active: bool,
    pub hover_t: f32,
    pub close_hover_t: f32,
    pub prefix: Option<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>>,
    pub suffix: Option<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>>,
    pub on_select: M,
    pub on_close: M,
    pub show_tooltip: bool,
    pub title_hover: F1,
    pub close_hover: F2,
}

pub fn panel_tab<'a, M, F1, F2>(
    args: PanelTabArgs<'a, M, F1, F2>,
) -> Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>
where
    M: Clone + 'a,
    F1: Fn(bool) -> M + 'a,
    F2: Fn(bool) -> M + 'a,
{
    let PanelTabArgs {
        title,
        active,
        hover_t,
        close_hover_t,
        prefix,
        suffix,
        on_select,
        on_close,
        show_tooltip,
        title_hover,
        close_hover,
    } = args;
    let close_sz = crate::theme::geometry::tab_button_size();
    // 组合 hover:悬停标题或 × 任一,胶囊背景都浮现、× 显形。
    let hover = combined_hover(hover_t, close_hover_t);
    let hovered = hover > 0.001;
    // 标题宽度不设上限:tab 随标题实际内容伸缩。给 `tab_label` 一个足够大的
    // 名义上限即可对其不产生截断效果(真实上限由外层行 `.clip` 决定)。
    let title_row = tab_label(prefix, title.clone(), active, hover_t, NO_TAB_W_LIMIT);

    // 选中/关闭的接线逻辑收在 `tabs::tab_core`(2026-08-12 抽取,试点已
    // 验证过——项目页签早已用它;这里是把面板 tab 自己那份原版实现换
    // 成同一个共享内核)。四个现有参数 title_hover/close_hover/on_select/
    // on_close 与 tab_core 的 on_select_hover/on_close_hover/on_select/
    // on_close 逐个对应,直接透传。
    let close_color = close_color(hover, close_hover_t);
    let (select, close) = tabs::tab_core(tabs::TabCoreArgs {
        content: title_row,
        close_sz,
        close_color,
        close_interactive: hovered,
        on_select,
        on_close,
        on_select_hover: title_hover,
        on_close_hover: close_hover,
    });

    let mut tab_row = row![select]
        .spacing(2)
        .align_y(iced_widget::core::Alignment::Center);
    if let Some(s) = suffix {
        tab_row = tab_row.push(s);
    }
    tab_row = tab_row.push(close);
    let el: Element<'a, M, iced_widget::Theme, iced_renderer::Renderer> = container(tab_row)
        .padding(Padding {
            top: PANEL_TAB_PAD_Y,
            right: PANEL_TAB_PAD_X,
            bottom: PANEL_TAB_PAD_Y,
            left: PANEL_TAB_PAD_LEFT,
        })
        .width(Length::Shrink)
        .style(tab_container_style(active, hover))
        .into();
    // 面板页签在屏幕底部,tooltip 用 `Top` 弹在页签上方,免出屏。仅当悬停
    // 满 2s(`show_tooltip`)才显示标题全称(见 `App::hover_tooltip_ready`)。
    controlled_tooltip(el, title, Position::Top, show_tooltip)
}

/// 溢出下拉入口。V 按钮在 tab 组非空时始终显示(即使当前没有横向溢出,
/// 下拉也要列出**该组全部 tab**,供随时跳转)。仅当 tab 组为空(编号 0)时
/// 返回 `None` 让调用方跳过——不走"禁用态灰按钮"(见 spec 语义确认)。
/// 图标 `ChevronsDown`(Lucide chevrons-down,双重箭头,与文件树分支切换
/// 等处共用的单箭头 `ChevronDown`区分开),套 `icons::icon_button_entry`
/// 标准图标按钮(静止 DIM、hover 平滑过渡到 GOLD),与面板内其余图标按钮
/// 同一套视觉,不再自绘 hover 背景/边框;
/// `hover_t`/`on_hover` 由调用方接自己那组专属 `HoverId`(如
/// `HoverId::TermTabOverflow`)——固定单按钮,不随 tab 增减/拖拽换位漂移,
/// 不需要 `rekey_hover_range`。命中区尺寸沿用原先的 `tab_arrow_button_size`,
/// 保证换皮前后不跳动。
pub fn tab_overflow_button<'a, M: Clone + 'a>(
    tab_count: usize,
    hover_t: f32,
    on_press: M,
    on_hover: impl Fn(bool) -> M + 'a,
) -> Option<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>> {
    if tab_count == 0 {
        return None;
    }
    Some(icons::icon_button_entry(
        icons::IconKind::ChevronsDown,
        crate::theme::icon_size::chevron(),
        false,
        false,
        hover_t,
        false,
        crate::theme::geometry::tab_arrow_button_size(),
        true,
        on_press,
        on_hover,
        // 空串:不带提示气泡(验收反馈去掉)——`icon_button_entry` 把空串
        // 当作"这个按钮不需要提示"的约定。
        "",
    ))
}

/// 悬浮下拉里的一行,对应该 tab 组里的**某个 tab**(V 菜单列的是组内全部
/// tab,不局限于当前横向被裁掉的)。`prefix` 与横向 tab 用同一个已经建好的
/// `Element`(状态点/图标/无),`active` 只决定标题文字颜色(CREAM,同
/// `tab_label` 的既有配色公式)——菜单里可能同时出现已经横向可见的当前选中
/// 项,文字颜色让用户看得出"这其实是当前那个",不再额外叠一个常驻背景框
/// (2026-09 用户反馈:那个框看着像卡死的 hover,跟其它行"只有真悬停才短暂
/// 显形"的规则不统一,见 `tab_overflow_menu` 文档)。
/// `closable=false` 用于 SSH/Database 的固定"空白"占位 tab(本来就不可关闭)。
/// `hover_t` 是这一行当前的行悬停动画进度(0..=1),由调用方从
/// `App::hover_progress(HoverId::TabOverflowRow(index))` 取出——菜单行本身
/// 不持有状态,和 `PanelTabArgs::hover_t` 同一套路(见 `tab_overflow_menu`)。
pub struct TabOverflowEntry<'a, M> {
    pub index: usize,
    pub prefix: Option<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>>,
    pub title: String,
    pub active: bool,
    pub closable: bool,
    pub hover_t: f32,
}

/// `tab_overflow_menu` 的参数:仿 `tabs::TabCoreArgs`/`tab_widget::PanelTabArgs`
/// 的具名字段结构体风格(闭包走结构体自身泛型参数,不用 `Box<dyn Fn>`)。
/// `anchor` 是点 V 按钮那一刻的 `App::last_cursor` 快照(逻辑坐标),
/// `window_size` 是当前窗口尺寸,两者一起用于把下拉钉在按钮附近同时不越出
/// 窗口边界。`on_row_hover` 把 `tab_core` 强制的 `on_select_hover`/
/// `on_close_hover` 两个回调接到该行自己的 `HoverId::TabOverflowRow(index)`
/// (`(idx, hovered) -> Message::Hover(...)`)——**绝不能**拿 `on_dismiss`
/// 顶替当 no-op:曾经图省事直接把 `on_dismiss.clone()` 传给 hover,酿成
/// "鼠标一移到菜单行上就把菜单自己关掉"的 bug(验收发现)。
pub struct TabOverflowMenuArgs<'a, M, FSel, FClose, FRH>
where
    M: Clone + 'a,
    FSel: Fn(usize) -> M,
    FClose: Fn(usize) -> M,
    FRH: Fn(usize, bool) -> M,
{
    pub entries: Vec<TabOverflowEntry<'a, M>>,
    pub anchor: (f32, f32),
    pub window_size: (f32, f32),
    pub on_select: FSel,
    pub on_close: FClose,
    pub on_dismiss: M,
    pub on_row_hover: FRH,
}

/// `tab_overflow_menu` 的原生菜单版本——纯选择列表。`entries` 由各调用方把
/// 各自的 tab 列表映射成 `(index, title, active)` 三元组;当前 tab(选中行)
/// 用 CREAM 文字 + 标题前固定图标列里的 `>`(`chevron-right`,CREAM)标识,
/// 其余行 BODY 且不画 `>`(2026-09 用户反馈:没选中的项不要显示箭头)。文字
/// 对齐靠渲染端 `MenuItemView` 为本菜单(走 `show` 默认口径)**无条件**预留
/// 固定图标列,无论该行是否真有图标,文字起点都钉在同一 x——不画箭头的行
/// 也不会错位(文件树/文件预览右键菜单的"无图标项顶到图标列左缘"是另一条
/// 口径,走 `show_align_no_icon_left`,与本菜单无关)。不含关闭按钮/
/// 状态点/hover(原生 NSMenu 是整行单击模型,见"迁移但去掉关闭按钮"的裁决),
/// 仅 macOS 编译。
#[cfg(target_os = "macos")]
pub fn tab_overflow_items<Msg: Clone>(
    entries: &[(usize, String, bool)],
    on_select: impl Fn(usize) -> Msg,
) -> Vec<crate::native_menu::Item<Msg>> {
    let cream = crate::theme::color::current().cream;
    let body = crate::theme::color::current().body;
    entries
        .iter()
        .map(|(idx, title, active)| {
            let color = if *active { cream } else { body };
            // 只有选中行带 `>` 图标(CREAM),未选中行 `icon: None` 不画箭头
            // (2026-09 用户反馈:没选中的项不要显示 `>`)。文字对齐靠原生渲染
            // 端 `MenuItemView` 为 `show` 默认口径**无条件**预留固定图标列
            // (宽度恒为 `icon_px + gap`),无论该行是否真有图标,文字起点都钉
            // 在同一 x——因此不画箭头的行不会把文字右推错位;iced 端
            // `tab_overflow_menu` 走同一口径(固定 `chevron_w` 列,未选中行塞
            // `Space`)。
            let (icon, icon_color) = if *active {
                (Some(icons::IconKind::ChevronRight), Some(cream))
            } else {
                (None, None)
            };
            crate::native_menu::Item::Entry {
                icon,
                icon_color,
                label: title.clone(),
                color,
                enabled: true,
                msg: on_select(*idx),
            }
        })
        .collect()
}

const TAB_OVERFLOW_MENU_MIN_WIDTH: f32 = 220.0;
/// 下拉宽度上限:即便可用屏幕空间(见下方 `avail_w`)比这个还宽,也不再
/// 继续撑——防止在超宽显示器上把菜单拉得离谱。跟 `TAB_OVERFLOW_MENU_MAX_HEIGHT`
/// 一样是个保守封顶(2026-09 用户反馈两轮:先是固定 220 宽把长文件名硬裁掉,
/// 换成按标题字符数估算宽度后估不准依然会裁——真正的修法是不再猜标题该有
/// 多宽,直接把"真实可用屏幕空间"当边界,标题短就窄、标题长就宽,只有真
/// 超出这个边界才被行内 `.clip(true)` 遮住,见下方 `menu_width` 计算)。
const TAB_OVERFLOW_MENU_MAX_WIDTH: f32 = 560.0;
const TAB_OVERFLOW_MENU_MAX_HEIGHT: f32 = 320.0;

/// 悬浮下拉列表:每行用既有的 `tabs::tab_core` 包装选中(mousedown)/关闭(×)
/// 交互(CLAUDE.md 裁决:tab 类 UI 优先复用 `tab_core`,不手写
/// `MouseArea`+`on_enter`/`on_exit`)。关闭按钮**始终可点**(不像横向 tab 那样
/// hover 才显形)——这是刻意的自定义(见截图:下拉里的 x 是常显的,不是
/// hover-only),因为悬浮列表本就是"已经主动点开来看"的场景,hover-only 反而
/// 多一次交互成本。行**有**动画 hover(验收反馈"tab 组下拉菜单应该有 hover
/// 效果"):`tab_core` 的 `on_select_hover`/`on_close_hover` 都接到该行
/// `HoverId::TabOverflowRow(index)` 上,行底色用 `entry.hover_t` 走横向 tab
/// 同一套 `tab_container_style` 插值(`hover = title_hover.max(close_hover)`,
/// 两者共用同一个 id,因此只取 `entry.hover_t` 一个值即可;同一帧内进/出会
/// 落到同一个 `HoverAnim`,不会来回闪)。`on_row_hover` **绝不能**拿
/// `on_dismiss` 顶替当 no-op:曾经图省事直接传 `on_dismiss.clone()`,酿成
/// "鼠标一移到菜单行上,`on_select_hover(true)` 一 fire 就把菜单自己关掉,
/// 根本放不上去"的 bug(验收反馈"鼠标无法放到展开的菜单上")。× 按钮本身仍有
/// "标准 hover 效果"——`tabs::tab_core` 的关闭按钮原生 `button::Status`
/// 悬停即变金,不依赖这里任何动画状态(见 `tab_core` 文档)。
///
/// 行底色恒传 `tab_container_style(false, entry.hover_t)`,不像横向 tab 那样
/// 给 `active` 行额外画一个常驻的选中方框(2026-09 用户反馈:下拉里当前
/// tab 那行永远描边、其它行只有真悬停时才短暂显形,两种视觉规则混在一起,
/// 看着像"hover 行为没统一"——一个格子的高亮到底是不是跟着鼠标走,行与行
/// 之间应该一致)。`entry.active` 仍然传给 `tab_label` 控制标题文字颜色
/// (CREAM vs 悬停插值的 DIM→GOLD),不叠背景框;当前 tab 改用标题前固定
/// 图标列里的 `>`(`chevron-right`,CREAM)来认(2026-09 用户要求)——所有行
/// 都给 `>` 留一个固定宽度的列,选中行画出来、未选中行留空,文字因此始终
/// 左对齐到同一 x,选中行不会被图标右推错位(原生版 `tab_overflow_items`
/// 只给选中行 `icon: Some(ChevronRight)`,未选中行 `icon: None`,靠渲染端
/// 无条件预留图标列走同一对齐口径)。
///
/// 悬浮定位:向下弹,手法同 `project_add_menu_popup`/`todo_calendar_overlay`——
/// `padding.top/left` 直接钉到锚点,钳一次 x/y 防止超出窗口右/下边缘(高度
/// 未知,用 `TAB_OVERFLOW_MENU_MAX_HEIGHT` 保守估算,同 `project_add_menu_popup`
/// "钳制会稍微保守但不会算少导致真的超出窗口"的容忍度)。**调用方必须**把
/// 这个函数的返回值提到 `App::view` 顶层 `stack![base, dismiss, popup]` 里拼
/// (同 `project_add_menu_popup` 的既有套路,见 `terminal::term_tab_overflow_popup`
/// 等四个姊妹函数)——`anchor`(`App::last_cursor` 快照)/`window_size` 都是
/// 全窗口坐标系,若嵌在某个面板自己的局部布局里,`Length::Fill` 只会撑满
/// 那个面板分到的一格(远小于整窗、左上角也非窗口原点),换算出来的位置会
/// 跟真实点击位置对不上(验收反馈"菜单错位了"——此前的教训)。
///
/// 文件/项目预览的 tab 栏下方是 wry webview 子视图,webview 恒在 iced 内容
/// 之上,向下弹的下拉本会被盖住——按 CLAUDE.md 裁决"需要盖住 webview 的
/// 原生浮层必须显式隐藏 webview,不能指望层级遮挡",在
/// `App::preview_desired` 里,`preview_tab_overflow_anchor`/
/// `project_preview_tab_overflow_anchor` 展开时强制该侧 webview
/// `visible = false`。终端/SSH/Database 三处没有 webview,不受影响。
pub fn tab_overflow_menu<'a, M, FSel, FClose, FRH>(
    args: TabOverflowMenuArgs<'a, M, FSel, FClose, FRH>,
) -> Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>
where
    M: Clone + 'a,
    FSel: Fn(usize) -> M + 'a,
    FClose: Fn(usize) -> M + 'a,
    FRH: Fn(usize, bool) -> M + 'a,
{
    let TabOverflowMenuArgs {
        entries,
        anchor,
        window_size,
        on_select,
        on_close,
        on_dismiss,
        on_row_hover,
    } = args;
    let close_sz = crate::theme::geometry::tab_button_size();
    // `menu_width` 是"真实可用屏幕空间"(锚点到窗口右边缘),不是靠标题字符数
    // 估出来的目标宽度——估算法则总有算不准的场景,标题实际渲染宽度只要比
    // 估算值宽一点,照样被裁。`row_max_w` 传给 `tab_label` 后只在标题**真的**
    // 比这块可用空间还宽时才触发它自带的 `.clip(true)`,短标题原样按自身宽度
    // 显示(`tab_label` 内部 `Length::Shrink`,不会被拉伸/裁短)。
    let menu_width = tab_overflow_menu_width(anchor.0, window_size.0);
    let row_max_w = menu_width - close_sz - 16.0;
    // 每行都要向视图层塞一份 hover 回调,闭包按行捕获 `idx`,无法共享同一个
    // 借用;`Rc` 让所有行共用同一份 `FRH`(只读 `Fn`,无内部可变性)。
    let on_row_hover = std::rc::Rc::new(on_row_hover);

    let mut rows: Vec<Element<'a, M, iced_widget::Theme, iced_renderer::Renderer>> = Vec::new();
    for entry in entries {
        let idx = entry.index;
        // 固定宽度的 `>` 图标列:选中行画 CREAM 的 chevron、未选中行留空
        // (`Space`),但两行都占同一列宽,文字由此左对齐到同一 x(与原生
        // `tab_overflow_items` 只给选中行塞 `icon: Some(ChevronRight)`、
        // 未选中行 `icon: None`、渲染端无条件预留图标列的口径一致);列宽
        // 统一从标题预算里扣掉,避免长标题被裁。
        let chevron_w = crate::theme::icon_size::row() + crate::theme::geometry::menu_gap();
        let label_max_w = (row_max_w - chevron_w).max(0.0);
        let label = tab_label(entry.prefix, entry.title, entry.active, 0.0, label_max_w);
        let marker: Element<'a, M, iced_widget::Theme, iced_renderer::Renderer> = if entry.active {
            row![icons::view(
                icons::IconKind::ChevronRight,
                crate::theme::icon_size::row(),
                crate::theme::color::current().cream,
            )]
            .into()
        } else {
            iced_widget::Space::new().into()
        };
        let label: Element<'a, M, iced_widget::Theme, iced_renderer::Renderer> =
            row![container(marker).width(Length::Fixed(chevron_w)), label,]
                .spacing(0)
                .align_y(iced_widget::core::Alignment::Center)
                .into();
        let close_color = crate::theme::color::current().dim;
        let (select, close) = tabs::tab_core(tabs::TabCoreArgs {
            content: label,
            close_sz,
            close_color,
            close_interactive: entry.closable,
            on_select: (on_select)(idx),
            on_close: (on_close)(idx),
            on_select_hover: {
                let f = on_row_hover.clone();
                move |h| f(idx, h)
            },
            on_close_hover: {
                let f = on_row_hover.clone();
                move |h| f(idx, h)
            },
        });
        let row_content = row![select, close]
            .spacing(4)
            .align_y(iced_widget::core::Alignment::Center);
        rows.push(
            container(row_content)
                .width(Length::Fill)
                .padding(Padding::new(4.0))
                .style(tab_container_style(false, entry.hover_t))
                .into(),
        );
    }

    let list = crate::interaction::menu::shell(
        vec![
            scrollable(column(rows).spacing(2))
                .height(Length::Shrink)
                .into(),
        ],
        Length::Fixed(menu_width),
    );
    let list = container(list).max_height(TAB_OVERFLOW_MENU_MAX_HEIGHT);

    // 全屏透明遮罩,接住"点外部关闭"(同已移除的预览 tab 右键菜单与
    // `project_add_menu_popup` 的既有套路)。内容本身不画任何东西(空白
    // Space),让容器撑满全窗。
    let dismiss = MouseArea::new(
        container(iced_widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(on_dismiss.clone());

    let (ax, ay) = anchor;
    let (window_w, window_h) = window_size;
    let x = ax.min((window_w - menu_width).max(0.0)).max(0.0);
    let y = ay
        .min((window_h - TAB_OVERFLOW_MENU_MAX_HEIGHT).max(0.0))
        .max(0.0);
    let positioned = container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: y,
            left: x,
            right: 0.0,
            bottom: 0.0,
        });

    stack![dismiss, positioned].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_widget::core::Color;

    type El = iced_widget::core::Element<'static, u8, iced_widget::Theme, iced_renderer::Renderer>;

    // —— 搬家前在 Dozer 上量到的黄金值(X1 Task 4 第 0 步) ——

    #[test]
    fn tab_padding_constants_match_dozer() {
        assert_eq!(PANEL_TAB_PAD_LEFT, 10.0);
        assert_eq!(PANEL_TAB_PAD_X, 4.0);
        assert_eq!(PANEL_TAB_PAD_Y, 0.0);
        assert_eq!(NO_TAB_W_LIMIT, f32::MAX);
    }

    #[test]
    fn tab_container_style_matches_dozers_three_states() {
        let theme = iced_widget::Theme::Dark;
        let colors = crate::theme::color::current();
        // 选中:CARD 实底 + 1px BORDER 边框 + 圆角 6。
        let active = tab_container_style(true, 0.0)(&theme);
        assert_eq!(active.background, Some(colors.card.into()));
        assert_eq!(active.border.color, colors.border);
        assert_eq!(active.border.width, 1.0);
        assert_eq!(active.border.radius, 6.0.into());
        // 未选中悬停:TAB_HOVER 胶囊,alpha = hover,无边框,圆角 6。
        let hovered = tab_container_style(false, 0.5)(&theme);
        assert_eq!(
            hovered.background,
            Some(
                Color {
                    a: 0.5,
                    ..colors.tab_hover
                }
                .into()
            )
        );
        assert_eq!(hovered.border.width, 0.0);
        assert_eq!(hovered.border.radius, 6.0.into());
        // 静止:完全透明。
        let rest = tab_container_style(false, 0.0)(&theme);
        assert_eq!(rest, iced_widget::container::Style::default());
    }

    // —— 溢出菜单宽度:Dozer 的公式,不随 UI 缩放 ——

    #[test]
    fn overflow_menu_width_follows_dozers_formula() {
        // 可用宽度 = 窗口宽 − 锚点 x − 24,夹在 [220, 560]。
        assert_eq!(tab_overflow_menu_width(0.0, 2000.0), 560.0);
        assert_eq!(tab_overflow_menu_width(1980.0, 2000.0), 220.0);
        assert_eq!(tab_overflow_menu_width(1000.0, 2000.0), 560.0);
        assert_eq!(tab_overflow_menu_width(500.0, 800.0), 276.0);
        assert_eq!(tab_overflow_menu_width(1900.0, 2000.0), 220.0);
    }

    #[test]
    fn overflow_menu_width_equals_diggers_geometry_at_scale_one() {
        // Digger `agent_tabs::overflow_geometry` 的宽度公式(逐字复制,scale = 1):
        // `(window.0 - anchor.0 - 24 * scale).clamp(220 * scale, 560 * scale)`。
        // 注意:Digger 的版本把三个常量都乘 scale,Dozer 的不乘——scale ≠ 1 时两者不同,
        // 迁移(Task 6)时以 Dozer 为准。
        for anchor_x in [0.0f32, 300.0, 1000.0, 1900.0, 1980.0] {
            for window_w in [800.0f32, 2000.0] {
                let digger = (window_w - anchor_x - 24.0).clamp(220.0, 560.0);
                assert_eq!(
                    tab_overflow_menu_width(anchor_x, window_w),
                    digger,
                    "anchor {anchor_x}, window {window_w}"
                );
            }
        }
    }

    // —— 纯函数(取自 Digger,公式与 Dozer 内联写法相同) ——

    #[test]
    fn hover_helpers_clamp_and_colour_like_dozer() {
        // `color::mix` 在端点附近有 1ulp 级的浮点误差,所以比较颜色用近似。
        let near = |a: Color, b: Color| {
            (a.r - b.r).abs() < 1e-5
                && (a.g - b.g).abs() < 1e-5
                && (a.b - b.b).abs() < 1e-5
                && (a.a - b.a).abs() < 1e-5
        };
        let c = crate::theme::color::current();
        assert_eq!(combined_hover(0.2, 0.7), 0.7);
        assert_eq!(combined_hover(2.0, 0.0), 1.0);
        assert_eq!(combined_hover(-1.0, -2.0), 0.0);
        assert_eq!(title_color(true, 0.9), c.cream, "选中恒 CREAM");
        assert!(near(title_color(false, 0.0), c.dim));
        assert!(near(title_color(false, 1.0), c.gold));
        assert!(near(title_color(false, 7.0), c.gold), "插值因子夹在 0..=1");
        // 静止时关闭按钮透明,悬停时 alpha = hover,颜色 dim→gold 按 close_hover 插值。
        assert_eq!(close_color(0.0, 0.0).a, 0.0);
        assert!(near(close_color(1.0, 1.0), c.gold));
    }

    // —— 构造与边界输入 ——

    fn entry(title: &str, active: bool) -> TabOverflowEntry<'static, u8> {
        TabOverflowEntry {
            index: 0,
            prefix: None,
            title: title.into(),
            active,
            closable: true,
            hover_t: 0.0,
        }
    }

    fn menu(entries: Vec<TabOverflowEntry<'static, u8>>) -> El {
        tab_overflow_menu(TabOverflowMenuArgs {
            entries,
            anchor: (100.0, 100.0),
            window_size: (800.0, 600.0),
            on_select: |i| i as u8,
            on_close: |i| i as u8,
            on_dismiss: 0u8,
            on_row_hover: |i, h| (i as u8).wrapping_add(h as u8),
        })
    }

    #[test]
    fn overflow_menu_with_zero_one_and_many_entries_builds() {
        let _ = menu(Vec::new());
        let _ = menu(vec![entry("only", true)]);
        let many = (0..500)
            .map(|i| TabOverflowEntry {
                index: i,
                ..entry(&format!("tab {i}"), i == 3)
            })
            .collect();
        let _ = menu(many);
        // 超长标题、窗口比菜单还小也不 panic。
        let _ = tab_overflow_menu(TabOverflowMenuArgs {
            entries: vec![entry(&"很长的标题".repeat(200), false)],
            anchor: (5000.0, 5000.0),
            window_size: (100.0, 100.0),
            on_select: |i| i as u8,
            on_close: |i| i as u8,
            on_dismiss: 0u8,
            on_row_hover: |_, _| 0u8,
        });
    }

    #[test]
    fn tab_label_never_wraps_and_always_clips() {
        // `Element` 看不到文本的换行/裁切设置,所以读生产源码钉住:iced `Text` 默认 `Wrapping::Word`,
        // 不显式关掉,标题超宽会折成两行(验收反馈:较长标题换行)。
        let src = include_str!("panel_tabs.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        let label = production
            .split("pub fn tab_label")
            .nth(1)
            .and_then(|rest| rest.split("\npub fn ").next())
            .expect("tab_label is in the production source");
        assert!(label.contains("Wrapping::None"), "标题必须显式不换行");
        assert!(label.contains(".clip(true)"), "标题超宽必须裁切");
    }

    #[test]
    fn the_tab_widget_tree_shape_does_not_depend_on_show_tooltip() {
        let shape = |show: bool| {
            let el = panel_tab(PanelTabArgs {
                title: "t".into(),
                active: false,
                hover_t: 0.0,
                close_hover_t: 0.0,
                prefix: None,
                suffix: None,
                on_select: 1u8,
                on_close: 2u8,
                show_tooltip: show,
                title_hover: |_| 3u8,
                close_hover: |_| 4u8,
            });
            (el.as_widget().tag(), el.as_widget().children().len())
        };
        assert_eq!(shape(true), shape(false));
    }

    #[test]
    fn overflow_button_is_absent_for_an_empty_group_and_present_otherwise() {
        assert!(tab_overflow_button(0, 0.0, 1u8, |_| 2u8).is_none());
        assert!(tab_overflow_button(1, 0.0, 1u8, |_| 2u8).is_some());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tab_overflow_items_colors_active_tab_cream_others_body() {
        let entries = vec![
            (0usize, "a".to_string(), false),
            (1usize, "b".to_string(), true),
        ];
        let items = tab_overflow_items(&entries, |idx| idx);
        let color_at = |i: usize| match &items[i] {
            crate::native_menu::Item::Entry { color, .. } => *color,
            crate::native_menu::Item::Separator => panic!("expected entry"),
        };
        assert_eq!(color_at(0), crate::theme::color::current().body);
        assert_eq!(color_at(1), crate::theme::color::current().cream);
    }

    /// 只有选中行带 `chevron-right` 图标(CREAM)、未选中行 `icon: None`
    /// 不画箭头(2026-09 用户反馈:没选中的项不要显示 `>`);文字对齐由渲染端
    /// 无条件预留图标列保证,这里钉住"数据层只给选中行塞图标"的口径。
    #[cfg(target_os = "macos")]
    #[test]
    fn tab_overflow_items_only_active_row_carries_chevron() {
        let entries = vec![
            (0usize, "a".to_string(), false),
            (1usize, "b".to_string(), true),
        ];
        let items = tab_overflow_items(&entries, |idx| idx);
        let row_at = |i: usize| match &items[i] {
            crate::native_menu::Item::Entry {
                icon, icon_color, ..
            } => (*icon, *icon_color),
            crate::native_menu::Item::Separator => panic!("expected entry"),
        };
        let cream = crate::theme::color::current().cream;
        let (icon0, color0) = row_at(0);
        let (icon1, color1) = row_at(1);
        // 未选中行不画箭头:icon 与 icon_color 都应是 None。
        assert_eq!(icon0, None);
        assert_eq!(color0, None);
        // 选中行画 CREAM 的 `>`。
        assert_eq!(icon1, Some(icons::IconKind::ChevronRight));
        assert_eq!(color1, Some(cream));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tab_overflow_items_msg_carries_original_index_via_on_select() {
        let entries = vec![(5usize, "five".to_string(), false)];
        let items = tab_overflow_items(&entries, |idx| format!("picked:{idx}"));
        match &items[0] {
            crate::native_menu::Item::Entry { msg, .. } => assert_eq!(msg, "picked:5"),
            crate::native_menu::Item::Separator => panic!("expected entry"),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tab_overflow_items_empty_entries_produce_empty_menu() {
        assert!(tab_overflow_items(&[], |idx: usize| idx).is_empty());
    }
}

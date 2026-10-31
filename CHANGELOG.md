# Changelog

## 未发布

- **行为变化:** `theme::icon_size` 的缩放入口统一走 `normalize_scale`:非有限数(NaN/±∞)、零和负数一律拒绝并保留原有效值,有限正数钳制到 `[SCALE_MIN, SCALE_MAX]`。此前 `set_scale(f32::NAN)` 会把 NaN 写入全局缩放,环境变量 `BYTEUI_ICON_SCALE=99` 会直接超出上限。
- 新增公开纯函数 `theme::icon_size::normalize_scale(f32) -> Option<f32>`,供宿主在写入前后复用同一套校验。
- `init_scale` 现在把有效的环境覆盖**物化**到运行时状态,初始化后 accessor 不再逐次读进程环境;`reset_scale` 回到本次启动的环境覆盖,没有覆盖时回到当前 token 基准(运行时 `set_theme` 换 token 后仍然跟随,不会被启动值冻结)。落盘格式不变。
- `zoom_by` 拒绝非有限/非正乘数以及乘法溢出,不写入状态。
- **破坏性变更:** `native_menu::show` / `show_align_no_icon_left` / `install_content_view` 已删除,所有调用改用 `native_menu::show_for_window` / `show_align_no_icon_left_for_window`(仅 macOS):显式借用调用方窗口句柄弹菜单,不再依赖全局登记的单槽指针;主线程/AppKit view 校验失败返回 `None`,不回退到其它窗口。Dozer 用 `Weak<Window>`(浮窗销毁后自动失效),Digger 用 `window::run` + `Task` 回投选择结果。
- **行为变化:** `feedback::math_curve` 的自驱动动画从“每次重绘立即续订”改为按 30 FPS 期限 `request_redraw_at`;迟到事件不补画积压帧。`frame_interval()` 由 16ms 改为约 33.3ms,新增 `TARGET_FPS`。新增 `view_animated_paused` / `view_animated_with_color_paused`,暂停时冻结相位、不续订,恢复不追赶。
- **行为变化:** `theme::color` 改为单锁 `ThemeSnapshot { colors, scheme, source, revision }`。`set_theme` 不再把任意 palette 当成命名方案:来源标记为 `PaletteSource::Custom`,明暗提示按 `bg` 亮度推导;`set_scheme` 仍标记 `PaletteSource::Named`。新增 `current_snapshot()` / `revision()`;旧 `current()` / `current_scheme()` 转发到快照,签名不变。
- 新增 `layout::panel_heading::PanelHeading`:图标 + 标题 + 可选尾部操作 + 统一行高/可选分割线的最小组合接口,供两产品收敛重复的面板标题行。
- 新增测试共用串行设施 `theme::test_support::lock_global_state`,color/icon_size/region 的全局状态测试共用同一把锁。

## 0.5.0

迁入 Dozer 与 Digger 原先各抄一份的页签 / 菜单部件(X1)。全部是**新增**,既有公开项签名不变;但有一处**行为变化**(见第一条)。

- **行为变化:** 新增 `theme::region::context_menu()`,右键菜单 / 下拉 / 溢出菜单的外框区域,**背景 `panel` 色且完全不透明(alpha = 1.0)**。Dozer 原先是 alpha ≈ 0.98,wgpu 线性混合下仍会让底下的亮字约 15% 透出;Digger 早已是 1.0。度量(圆角 9、内边距 8、项间距 1,后两者乘全局 scale;描边 1px)与 Dozer `workspace.json` 的 `context_menu` 项相同,`theme::region::spec_dump()` 因此多了 `context_menu` 一项(外壳区域由 7 个变 8 个),跨仓库一致性检查脚本要同步。
- 新增 `interaction::menu`:菜单项 `item` / `item_locked` / `item_row` / `item_row_fill` / `icon_leading`、分隔线 `separator`、外框 `shell`;iced 自绘浮层 `MenuAnchor`(`clamped`)、`OverlayRow<Msg>`、`rows_size`、`view_rows`、`overlay_shell`(外框 + 整窗遮罩 + 定位,**与只是外框的 `shell` 不是一回事**)。来自 Dozer `chrome/menu.rs` 与 Digger `adapters/menu_overlay.rs`;Dozer 依赖本地噪点层的 `shell_frosted` 不在此。
- 新增 `interaction::panel_tabs`:面板页签 `panel_tab` / `PanelTabArgs`、`tab_label`、`tab_container_style`、溢出入口 `tab_overflow_button`、溢出菜单 `tab_overflow_menu` / `TabOverflowMenuArgs` / `TabOverflowEntry`、macOS 原生菜单项 `tab_overflow_items`,常量 `PANEL_TAB_PAD_LEFT/X/Y`、`NO_TAB_W_LIMIT`,纯函数 `combined_hover` / `title_color` / `close_color` / `tab_overflow_menu_width`(宽度公式不随 UI 缩放,夹在 220–560)。来自 Dozer `chrome/tab_widget.rs`(预览专属的三个模式切换按钮不在此)。悬停进度 `hover_t` 由调用方给。
- 新增 `feedback::tooltip::controlled_tooltip` 与 `TOOLTIP_DELAY`(2 秒):页签 / 顶栏标题全称气泡,控件树结构不随 `show` 翻转。
- 新增 `theme::font::top_bar_font()`:系统默认字体的 Medium 档。

## 0.4.6

- 新增 `native_menu`(仅 macOS):原生 NSMenu 右键 / 弹出菜单,自 Dozer 的 `dozer-app::chrome::native_menu` 原样搬入,供 Dozer 与 Digger 共用。原生菜单是真窗口层级,天然浮在 wry 原生页面之上,不需要"弹出时把页面藏起来"。接口:`Item`(`Entry` / `Separator`)、`show` / `show_align_no_icon_left`(同步阻塞,选中返回对应消息)、`install_content_view`(窗口初始化时登记一次;**签名变化**:接收 `&dyn raw_window_handle::HasWindowHandle`,不再依赖 `winit`)。日志改用 `tracing`(target `byteui::native_menu`)。依赖 `objc2` / `objc2-app-kit` / `objc2-foundation` / `objc2-quartz-core` / `resvg` / `usvg` / `tiny-skia`(后三者与 iced_wgpu 同一精确版本),全部只在 macOS 目标上引入。
- `examples/gallery`:`GALLERY_SCHEME=light` 切换浅色方案,并增加一行带全部强调色的字符徽标样例(0.4.5 之后就已存在于主干,随本版一并发布,只涉及示例)。

## 0.4.5

- 新增 `interaction::icons::glyph_view` / `glyph_button_entry` / `readable_ink`:字符徽标——`size × size` 的圆角方块,底色任意、字符居中(系统默认字体 + `Shaping::Advanced`,中文可用)。`glyph_button_entry` 与 `icon_button_entry` 共用同一个按钮外壳(卡片底、选中金框、hover、拖拽 dim、tooltip),外壳已抽成私有 `button_shell`;`icon_button_entry` 行为逐字不变。`readable_ink(fill, light, dark)` 按 WCAG 对比度在两个候选里选更易读的字符色。供 dozer 给已安装的应用图标栏入口画首字母徽标。`examples/gallery` 增加了徽标的普通 / 选中 / 拖拽 dim 三态样例。
- 新增颜色 token `teal`(深色 `#2DD4BF` / 浅色 `#0D9488`)。**注意:** 这是 0.4.4 之后、本版之前就已存在于主干的提交(`feat(theme): 新增 teal 语义色 token`),随本版一并发布。`ColorTokens` 因此多了一个字段——任何用结构体字面量构造 `ColorTokens` 的下游代码需要补上 `teal`。

## 0.4.4

- 新增 `IconKind::QoderLight` / `IconKind::QoderDark`:Qoder 品牌 logo,明暗主题各一份,自带固定色(`preserves_original_color()` 为 `true`),用法同 `GooseLight` / `GooseDark`。素材由官方 PNG 矢量化,`assets/icons/qoder-{light,dark}.svg` 除填充色外逐字节相同。供 dozer 新增的 qodercn agent 使用。

## 0.4.3

- 新增 `theme::region`:外壳区域样式(`top_bar`、`left_icon_rail`、`right_icon_rail`、`status_bar`、`left_zone`、`right_zone`、`maximize_overlay`、`window_background`)。数值与 Dozer `workspace.json` 的同名区域一致;颜色令牌每次现读,`padding` / `margin` / `gap` 与放大浮层尺寸乘全局 UI scale。供共享窗口骨架 `bytehost-chrome` 使用。
- 新增 `examples/dump_shell_regions.rs`:导出这些区域的规格 JSON,供跨仓库一致性检查使用。

## 0.4.2

- `form::button`、`form::checkbox`、`form::switch` 的 `label`，以及 `form::text_area` 的 `placeholder`，参数类型由 `&'a str` 放宽为 `&str`（内部转成拥有的 `String`），调用方不必再让字符串活得和返回的 `Element` 一样久。源码兼容：原来传 `&'a str` 的调用点不用改。

## 0.4.1

- 新增 `IconKind::SquareSparkles`（Lucide square-sparkles），供 dozer 群聊面板 rail 图标使用。

## 0.4.0

- 新增 `feedback::dialog`：弹窗外观原语，自 dozer-app 的 `dialog.rs` 搬来，行为不变。
  `card_style`、`actions`、`action_button_border_color`、`action_button_style`、
  `ConfirmDialog`、`confirm`。
- `card_style` 的 PANEL 底、金色描边（1.5）与圆角（8）改为组件内常量，取值与搬迁前一致。

## 0.3.0

- 新增 `interaction::tab_strip`：页签栏纯逻辑（不含 iced 类型），自 dozer-app 搬来、行为不变。
  `TabOverflow`、`tab_window`、`tab_window_reveal`（溢出窗口与选中自动带入可见区），
  `tab_drag_past_threshold` 与 `TAB_DRAG_CONFIRM_THRESHOLD_PX`（拖拽确认阈值）。

## 0.2.1

- gallery：各行内容垂直居中对齐；补充带前置图标的 Button 与数量为 0 的 Badge。无库代码改动。

## 0.2.0

- 新增 `data::tag`（带文字标签，`Tone` 选语义色）。
- 新增 `data::badge`（数字角标与红点角标）。
- 新增 `form::button`（Primary / Secondary / 禁用态，可带前置图标）。
- 这是对 2026-08-18 byteui 计划中"不做 Badge/Tag"的修订：digger 设计稿已明确需要。

## 0.1.0

- 自 dozer 仓库的 `crates/byteui` 拆出，带完整提交历史；内容与拆出前一致。
- 新增环境变量 `BYTEUI_ICON_SCALE`；`DOZER_ICON_SCALE` 作为旧名继续兼容。

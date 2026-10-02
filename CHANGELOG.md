# Changelog

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

# Changelog

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

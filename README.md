# byteui

ByteBoy 系产品（dozer、digger）共用的 iced 0.14 组件库。组件命名对齐 amis 的分类与名字。

## 使用

    byteui = { git = "https://github.com/byteboyai/byteui", tag = "v0.1.0" }

一律用 `tag`，不要用 `branch`。

## iced 版本

所有消费方必须与本库使用同一个 iced 版本（当前 0.14），否则 `Element` 类型不兼容。
升级 iced 时 byteui、dozer、digger 一起升。

## 主题

默认值是 ByteBoy2077。应用在启动时调用 `theme::{color,font,geometry,icon_size}::set_theme()`
整体替换。`icon_size::{init,persist,reset}_scale` 的落盘路径由应用传入。
环境变量 `BYTEUI_ICON_SCALE` 可覆盖启动缩放（`DOZER_ICON_SCALE` 为旧名，仍兼容）。

## 本地联调（不提交）

在消费方的 `.cargo/config.toml` 里加：

    [patch."https://github.com/byteboyai/byteui"]
    byteui = { path = "../byteui" }

## 什么组件可以进库

1. 至少两个项目需要，或设计稿里已明确出现；只有一个项目用的留在该应用里。
2. 命名沿用 amis 组件名。
3. 组件无状态：`fn(参数) -> Element`；颜色、字号、间距一律读 token，不写死数值。
4. 每个组件有最小测试，覆盖各种状态。

## 版本

语义化版本。加组件是 minor；改已有签名是 breaking（0.x 阶段也要在 CHANGELOG 写明）。

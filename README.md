# Bili Live 项目

获取哔哩哔哩直播间的推流码信息

## 功能
- 获取直播间的推流码信息
- 自动获取登录cookie
- 多级菜单选择直播分区
- 多平台支持

## 安装

1. 确保已安装Rust工具链
2. 克隆本项目
3. 运行 `cargo build`

## 使用

1. 配置 `cookies.json` 文件
2. 运行 `cargo run`

## 手动配置（不推荐）

在 `cookies.json` 中设置:
- room_id: 直播间ID
- sessdata: 登录cookie
- csrf_token: CSRF token

注意: `cookies.json` 已被添加到 `.gitignore`，请勿提交敏感信息。
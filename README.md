# Bili Live 项目

获取哔哩哔哩直播间的推流码信息

> [!WARNING]  
> 此项目仅供学习交流使用，请勿用于商业用途。所有接口及类型均来自于网络文档收集，本项目仅对其进行整理和聚合，如有侵权请联系删除。
> 
> 本项目仅用于学习交流，不保证其准确性和可靠性。使用本项目产生的一切后果，与作者无关。

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
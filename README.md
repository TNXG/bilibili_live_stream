# Bili Live Stream

一个使用 **Rust** 编写的哔哩哔哩直播推流码获取工具。

---

> [!WARNING]
> 本项目仅供学习交流使用，禁止用于任何商业用途。所有接口与类型均来源于公开网络文档，项目仅对其整理与聚合。若有侵权请联系删除。  
> 本项目不保证功能的准确性与可靠性，使用本项目所导致的一切后果与作者无关。

## ✨ 功能特点

- 获取指定直播间的推流码信息  
- 自动获取登录 Cookie 信息  
- 多级菜单选择直播分区  
- 跨平台支持：Windows / Linux / macOS  

## 📦 安装方式

### 预编译版本

支持以下平台，可前往 [Release 页面](https://github.com/TNXG/bilibili_live_stream/releases) 下载最新版本：

- Windows x86_64  
- Linux x86_64 / aarch64  
- macOS Intel (x86_64) / Apple Silicon (aarch64)  

### 自行编译

如果预编译版本不满足需求，你可以自行构建：

#### 1. 安装 Rust

- 在 Unix / Linux 系统中执行：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
````

- 在 Windows 系统中使用：

```bash
winget install Rustlang.Rustup
```

#### 2. 克隆仓库

```bash
git clone https://github.com/TNXG/bilibili_live_stream.git --depth=1
cd bilibili_live_stream
```

#### 3. 编译项目

```bash
cargo build --release
```

如果在编译过程中遇到依赖安装缓慢或者出现错误，可以尝试使用 [RsProxy](https://rsproxy.cn/)

编译完成后，可执行文件将在 `target/release/` 目录下生成。

## 🔐 使用须知

1. **信息安全**

   * 本项目**不收集**任何用户信息，包括用户名、密码、Cookie 等。
   * 使用中的 `csrf_token` 和 `SESSDATA` 等 Cookie 字段均为用户登录后本地获取，**属于高度敏感信息**，请务必妥善保管。
   * `SESSDATA` 的敏感程度等同于“密码 + 验证码”，切勿泄露。

2. **合法使用**

   * 本工具仅限用于技术学习与研究，**禁止用于违反哔哩哔哩用户协议的行为**。
   * 作者不对使用本工具产生的任何封号、风控等后果负责。
   * 严禁将获取的推流码用于未授权的多平台转播等违规行为。

> [!WARNING]
> 哔哩哔哩推流码为敏感信息，请严格遵守 [哔哩哔哩直播服务协议](https://live.bilibili.com/p/html/live-app-help/index.html#/live-protocol)。
> **高频调用接口可能导致账号风控或封禁，请谨慎使用！**

## 反馈建议与贡献
如果你在使用过程中遇到任何问题或有任何建议，欢迎新开立 Issue 或提交 Pull Request。
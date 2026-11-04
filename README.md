# Bili Live Stream

一个使用 **Rust** 编写的哔哩哔哩直播推流码获取工具，提供终端交互界面（TUI），支持扫码登录、分区开播、推流信息获取与下播统计。

---

> [!WARNING]
> 本项目仅供学习交流使用，禁止用于任何商业用途。所有接口与类型均来源于公开网络文档，项目仅对其整理与聚合。若有侵权请联系删除。  
> 本项目不保证功能的准确性与可靠性，使用本项目所导致的一切后果与作者无关。
> 本项目随时可能因为接口变动、许可协议变更等原因导致无法使用或者停止维护。

## ✨ 功能特点

- 终端内扫码登录，自动保存并复用 `cookies.json` 登录状态
- 双栏选择直播分区，支持按上次分区一键快捷开播
- 获取 RTMP、SRT、WHIP 主备线路，支持一键复制完整推流地址、服务器地址或推流码
- 默认隐藏敏感推流信息，避免截图或录屏时意外泄露
- 支持刷新推流信息、关闭直播间及查看本场直播统计数据
- 跨平台支持：Windows / Linux / macOS

## 🖥️ 操作说明

支持键盘快捷键与鼠标操作（可直接点击底部按钮，或使用滚轮切换列表、翻阅日志）。登录二维码会根据窗口尺寸自动缩放，并在支持 Kitty / Sixel / iTerm2 图片协议的终端中自动使用高清图片显示。

| 页面 | 按键 | 说明 |
|---|---|---|
| 登录 | `r` | 刷新二维码 / 重试登录 |
| 分区选择 | `↑` / `↓`、`←` / `→` / `Tab` | 切换一级 / 二级分区 |
| 分区选择 | `Enter` | 进入二级分区，或选中分区开播 |
| 分区选择 | `l` | 使用上次分区直接开播 |
| 推流信息 | `↑` / `↓` | 切换推流线路 |
| 推流信息 | `c` / `a` / `k` | 复制完整推流地址 / 服务器地址 / 推流码 |
| 推流信息 | `h` | 显示 / 隐藏敏感推流信息 |
| 推流信息 | `r` / `s` | 刷新推流信息 / 关闭直播间 |
| 全局 | `PgUp` / `PgDn` | 翻阅底部日志 |
| 全局 | `q` / `Ctrl+C` | 退出程序（开播状态下会提示确认是否先关播） |
| 退出确认 | `y` / `n` / `Esc` | 确认关播并退出 / 取消 |
| 关播统计 | `Enter` | 返回分区选择 |

## 📦 安装方式

### 预编译版本

前往 [Release 页面](https://github.com/TNXG/bilibili_live_stream/releases) 下载适合自己系统和架构的安装包：

| 平台 | 架构 / ABI |
|---|---|
| Windows | x86_64、i686、ARM64（MSVC） |
| macOS | Intel x86_64、Apple Silicon ARM64 |
| Linux GNU / musl | x86_64、i686、ARM64、ARM / ARMv7（软浮点及硬浮点）、ARMv5TE（软浮点）、RISC-V 64、LoongArch64 |
| Linux GNU | PowerPC、PowerPC64、PowerPC64LE、s390x、SPARC64 |
| FreeBSD | x86_64、i686 |

- **选择安装包**：Windows 使用 `.zip`，其他平台使用 `.tar.gz`；文件名包含版本和完整 Rust 目标三元组，压缩包包含程序、README 和许可证，不包含 Cookie。
- **Linux 兼容性**：GNU 包依赖构建工具链对应的 glibc，旧发行版或 Alpine 优先尝试 musl 包。musl 包静态链接 libc，但仍需系统 CA 证书；剪贴板依赖可用的桌面会话。
- **兼容性**：FreeBSD 包需匹配系统 ABI；macOS 与 Windows 包未签名，首次运行可能出现系统安全提示。各平台的验证范围见对应 Release。
- **完整性**：每次 Release 提供 `SHA256SUMS`。Linux 可将已下载的安装包与校验文件放在同一目录，执行 `sha256sum --ignore-missing -c SHA256SUMS`；macOS 使用 `shasum -a 256 <安装包>`，Windows 使用 `Get-FileHash <安装包> -Algorithm SHA256` 与对应行比较。

### 自行编译

如果预编译版本不满足需求，你可以自行构建：

#### 1. 安装 Rust

- 在 Unix / Linux 系统中执行：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

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

## 🍎 macOS 下 Release 包使用说明

1. 前往 [Release 页面](https://github.com/TNXG/bilibili_live_stream/releases) 下载适合你设备架构的 macOS 版本：
   - `x86_64`：适用于 Intel 处理器的 Mac 设备
   - `aarch64`：适用于 Apple Silicon (M1/M2/M3/M4/M5) 处理器的 Mac 设备
2. 下载后，解压压缩包（如有）。
3. 打开终端，进入解压目录，赋予可执行权限：
   ```bash
   chmod +x bili_live
   ```
4. 运行程序：
   ```bash
   ./bili_live
   ```
5. 如遇“无法打开，因为它来自身份不明的开发者”，可在“系统设置 → 隐私与安全性”中允许该程序运行，或在终端执行：
   ```bash
   xattr -d com.apple.quarantine bili_live
   ```

## 🐧 Linux 下 Release 包使用说明

1. 前往 [Release 页面](https://github.com/TNXG/bilibili_live_stream/releases) 下载适合你设备架构的 Linux 版本。
2. 下载后，解压压缩包（如有）。
3. 打开终端，进入解压目录，赋予可执行权限：
   ```bash
   chmod +x bili_live
   ```
4. 运行程序：
   ```bash
   ./bili_live
   ```
5. 如遇"权限不足"或"找不到命令"，请确认当前目录下有`bili_live`文件，并已赋予可执行权限。

## 🔐 使用须知

1. **信息安全**

   * 本项目**不收集**任何用户信息，包括用户名、密码、Cookie 等。
   * 使用中的 `csrf_token` 和 `SESSDATA` 等 Cookie 字段均为用户登录后本地获取，**属于高度敏感信息**，请务必妥善保管。
   * `SESSDATA` 的敏感程度等同于“密码 + 验证码”，切勿泄露。

2. **合法使用**

   * 本工具仅限用于技术学习与研究，**禁止用于违反哔哩哔哩用户协议的行为**。
   * 作者不对使用本工具产生的任何封号、风控等后果负责。
   * 严禁将获取的推流码用于未授权的多平台转播等违规行为。

> **提示**：登录成功后，程序会在你运行命令时的当前目录（即终端的`pwd`命令输出的路径下）下生成 `cookies.json` 文件，请妥善保管。

> [!WARNING]
> 哔哩哔哩推流码为敏感信息，请严格遵守 [哔哩哔哩直播服务协议](https://live.bilibili.com/p/html/live-app-help/index.html#/live-protocol)。
> **高频调用接口可能导致账号风控或封禁，请谨慎使用！**

## 反馈建议与贡献
如果你在使用过程中遇到任何问题或有任何建议，欢迎新开立 Issue 或提交 Pull Request。
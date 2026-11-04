# Bili Live Stream

一个使用 **Rust + ratatui** 编写的哔哩哔哩直播推流信息获取工具。只管理 B 站房间开播/关播并获取推流信息，**不采集、编码或发送音视频**。

---

> [!WARNING]
> 本项目仅供学习交流使用，禁止用于任何商业用途。所有接口与类型均来源于公开网络文档，项目仅对其整理与聚合。若有侵权请联系删除。  
> 本项目不保证功能的准确性与可靠性，使用本项目所导致的一切后果与作者无关。
> 本项目随时可能因为接口变动、许可协议变更等原因导致无法使用或者停止维护。

## ✨ 功能特点

- 终端内扫码登录，兼容已有 `cookies.json`
- 双栏分区选择，以及上次分区快捷操作
- 展示服务端返回的 RTMP、SRT、WHIP 主/备用线路、地址和推流码
- 完整端点、原始地址、推流码分别复制；默认隐藏全部敏感字段
- 刷新推流信息、关闭直播间、查看本场统计
- 内存日志面板，不向终端穿插输出日志；支持窗口缩放
- Windows / Linux / macOS 终端后端

## 使用 TUI

```bash
cargo run
# 如需启动时显示全部地址和推流凭据：
cargo run -- --show-full-code
# 默认开播前会读取测速节点并索取 SRT 线路；如需只用服务端默认线路：
cargo run -- --no-request-srt
```

需要交互式终端，建议至少 **80×24**。界面底部始终显示可点击按钮，也可用对应快捷键；部分终端不支持鼠标事件，此时快捷键仍然有效。登录页二维码按终端尺寸自适应放大/缩小：终端支持图片协议（Kitty / Sixel / iTerm2）时渲染像素正方形图片，否则使用黑白块；两种方式都保持正方形、不裁剪、不做非等比拉伸。

| 页面 | 按键 | 操作 |
|---|---|---|
| 登录 | `r` | 刷新二维码或重试登录 |
| 分区 | `↑` / `↓`、`←` / `→` / `Tab` | 选择一级/二级分区 |
| 分区 | `Enter` | 进入二级列表；在二级分区上开播并获取推流信息 |
| 分区 | `l` | 使用上次分区开播并取码 |
| 推流信息 | `↑` / `↓` | 查看不同的服务端记录，**不切换推流连接** |
| 推流信息 | `c` / `a` / `k` | 分别复制完整端点 / 原始地址 / 推流码 |
| 推流信息 | `h` | 显示或隐藏地址、推流码和优选链接 |
| 推流信息 | `r` / `s` | 刷新信息 / 关闭 B 站直播间 |
| 全局 | `PgUp` / `PgDn` | 滚动日志 |
| 全局 | 鼠标左键 | 点击底部按钮执行同一操作（需终端支持鼠标事件） |
| 全局 | 鼠标滚轮 | 指针在日志上滚动日志；在列表上移动选中项 |
| 全局 | `q` / `Ctrl+C` | 退出；房间已开播时须确认关播，失败则保留当前状态 |
| 退出确认 | `y` / `n` / `Esc` | 确认关播并退出 / 取消 |
| 关播结果 | `Enter` | 返回分区选择 |

复制失败时不会清空已有信息，可按 `h` 显示后手动复制。系统剪贴板需要桌面会话支持；纯 SSH / 无头环境可能不可用。

### SRT 与 WHIP 的边界

- 优先读取 `data.protocols`，保留顺序并标记 `protocols_backup`；缺少相应列表时兼容旧 `rtmp` / `rtmp_backup` 字段。
- **SRT**：直播姬的 `srt` / `rtmpsrt` 通常是私有 **RTMP-over-SRT**，不是普通 MPEG-TS-over-SRT；不会把 RTMP 地址改写成 `srt://`，不保证 OBS / ffmpeg 通用 SRT 输入兼容。
- SRT 是否出现由服务端决定，而且只发 `recommend_protocol=srt` 不够：实测必须同时提供 `recommend_ip`，且该 IP 要来自 `/xlive/app-blink/v1/upStreamConfig/GetUpStreamSpeedTestInfo` 返回的 `srt` 节点（`rtmpsrt://`）。默认开播会先拉取测速节点，选中一条 SRT 节点后把 `recommend_protocol=srt` 与 `recommend_ip` 一起附到 `startLive`（`--no-request-srt` 可关闭这一步）；账号或分区不支持时服务端仍只返回 RTMP。`GetUpStreamRtmp` 是 App 侧刷新接口，当前纯 Cookie 调用实测返回 `-400`，所以 SRT 以开播响应为准；页面顶部的 `SRT:暂无` 表示服务端确实没有下发。
- **WHIP**：展示服务端实际返回的 HTTPS 端点和原始凭据，需要另外使用兼容的 WHIP 编码器；不会把 `code` 猜成 Bearer 令牌。
- 某个协议未下发时明确显示“未返回”，不伪造地址、不自动选择协议、不向媒体端点发起连接。
- USB / OSI / 未知协议只作信息展示，不生成网络端点。
- 协议可用性受 B 站账号、分区、平台和服务端开关影响。结构证据与限制见 [逆向资料](docs/reverse-engineering/bilibililive-7.12.0.md)。

网络请求在独立工作线程执行。请求中的退出不会跳过已发起的开播操作：确认后等待结果，再关播退出。`kill -9`、断电等强制终止无法保证关播，请自行核实 B 站房间状态。

## 📦 安装方式

### 预编译版本

本项目支持多个平台和架构，提供预编译二进制文件降低客户端构建的碳排放。可前往 [Release 页面](https://github.com/TNXG/bilibili_live_stream/releases) 下载最新版本：

- **Windows**
  - x86_64 (64位 Intel/AMD)
  - i686 (32位 Intel/AMD)
  - aarch64 (ARM64 架构，如 Surface Pro X)

- **macOS**
  - x86_64 (Intel 处理器)
  - aarch64 (Apple Silicon M1/M2/M3/M4/M5)

- **Linux**
  - x86_64 (64位)
  - i686 (32位)
  - aarch64-gnu (ARM64 架构，如树莓派 3+、服务器)
  - x86_64-musl (x64 静态链接版本，适用于 Alpine、容器)
  - armv7-gnueabihf (ARMv7 硬浮点，如树莓派 2)
  - arm-gnueabihf (ARM 硬浮点，旧设备)
  - aarch64-musl (ARM64 静态链接，Alpine on ARM)
  - armv7-musleabihf (ARMv7 静态链接，容器)
  - i686-musl (32位 x86 静态链接)  

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

## 代码组织与验证

```text
src/
  api/       HTTP 客户端、登录、凭据存储、房间接口
  model/     分区、登录、协议记录、会话与统计数据
  app/       页面状态、键盘操作、异步结果状态迁移
  runtime/   终端循环、剪贴板所有权、网络工作线程
  ui/        登录、分区、协议详情、统计、日志与公共绘制
  logging/   有界内存日志缓冲
  lib.rs     模块入口
  main.rs    命令行参数与程序入口
tests/       独立集成测试及本地 HTTP fixture
old_src/     旧实现参考，不参与构建
```

业务文件按职责拆分，单文件不超过 300 行；测试不放在业务文件中。

```bash
cargo test
cargo fmt --check
```

本地联调可同时设置 `BILI_API_BASE`、`BILI_PASSPORT_BASE`、`BILI_WEB_BASE`，三者必须全部配置为仅包含 origin 的 HTTP(S) 字面量回环地址，例如 `http://127.0.0.1:18080`。不接受远程主机、路径、用户信息、query 或 fragment。联调时请在临时工作目录使用测试 `cookies.json`，不要加载真实账号凭据。集成测试通过 `Api::with_origins` 和独立临时目录隔离，不修改全局环境或工作目录。

HTTP 连接超时 10 秒，单次请求超时 30 秒；API 响应限制为 4 MiB。日志级别通过 `BILI_LOG=info|debug|warn|error` 设置，仅应用日志进入面板，不记录 HTTP 库调试内容、Cookie 或完整推流信息。

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
6. **依赖说明**：
   - 使用 Rustls，不需要 OpenSSL / `libssl`；HTTPS 需要系统信任根证书。
   - GNU 构建仍需要相应平台的 C 运行时；musl 构建适用于匹配架构的 musl 环境。
   - Linux 系统剪贴板依赖可用的桌面会话；不可用时在 TUI 中提示，不影响获取推流信息。

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

Unix 上新保存的凭据文件权限为 `0600`，使用同目录临时文件替换；Windows 请使用受当前用户访问控制保护的目录。默认显示隐藏值并不等于加密存储；按 `h` 或使用 `--show-full-code` 后，请勿分享终端截图。

> [!WARNING]
> 哔哩哔哩推流码为敏感信息，请严格遵守 [哔哩哔哩直播服务协议](https://live.bilibili.com/p/html/live-app-help/index.html#/live-protocol)。
> **高频调用接口可能导致账号风控或封禁，请谨慎使用！**

## 反馈建议与贡献
如果你在使用过程中遇到任何问题或有任何建议，欢迎新开立 Issue 或提交 Pull Request。
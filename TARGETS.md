# 支持的编译目标 (Supported Build Targets)

此文档列出了 bili_live 项目支持的所有编译目标。

## 编译目标列表

### Windows 系统

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `x86_64-pc-windows-msvc` | Windows 64-bit (Intel/AMD) | MSVC 工具链 |
| `i686-pc-windows-msvc` | Windows 32-bit (Intel/AMD) | MSVC 工具链 |
| `aarch64-pc-windows-msvc` | Windows ARM64 | MSVC 工具链，用于 Snapdragon 处理器 |
| `x86_64-pc-windows-gnu` | Windows 64-bit (GNU) | MinGW 工具链 |
| `i686-pc-windows-gnu` | Windows 32-bit (GNU) | MinGW 工具链 |

### macOS / Apple 平台

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `x86_64-apple-darwin` | macOS Intel (x64) | Intel 处理器 |
| `aarch64-apple-darwin` | macOS Apple Silicon | M1/M2/M3 芯片 |

### Linux 系统 (GNU - 动态链接)

#### 主流架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `x86_64-unknown-linux-gnu` | Linux x64 | 最常见的 Linux 架构 |
| `i686-unknown-linux-gnu` | Linux x86 | 32-bit Intel/AMD |
| `i586-unknown-linux-gnu` | Linux x586 | 较旧的 Intel/AMD |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | Raspberry Pi 4+, 服务器 |

#### ARM 架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `armv7-unknown-linux-gnueabihf` | Linux ARMv7 (硬件浮点) | Raspberry Pi 2/3, 嵌入式设备 |
| `armv7-unknown-linux-gnueabi` | Linux ARMv7 (软件浮点) | 旧的 ARM 设备 |
| `arm-unknown-linux-gnueabihf` | Linux ARM (硬件浮点) | 通用 ARM 支持 |

#### 其他架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `powerpc-unknown-linux-gnu` | Linux PowerPC | IBM POWER 系统 |
| `powerpc64-unknown-linux-gnu` | Linux PowerPC64 | IBM POWER 系统 64-bit |
| `powerpc64le-unknown-linux-gnu` | Linux PowerPC64LE | IBM POWER 系统小端序 |
| `riscv64gc-unknown-linux-gnu` | Linux RISC-V 64-bit | 开源 ISA |
| `s390x-unknown-linux-gnu` | Linux s390x | IBM System z 大型机 |
| `sparc64-unknown-linux-gnu` | Linux SPARC64 | SPARC 架构 |
| `loongarch64-unknown-linux-gnu` | Linux LoongArch 64-bit | 龙芯架构 |

### Linux 系统 (musl - 静态链接)

静态链接版本，不依赖 glibc，适合在多个 Linux 发行版间运行。

#### 主流架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `x86_64-unknown-linux-musl` | Linux x64 (静态) | Alpine Linux 等 |
| `i686-unknown-linux-musl` | Linux x86 (静态) | 32-bit 静态链接 |
| `i586-unknown-linux-musl` | Linux x586 (静态) | 旧型 Intel/AMD 静态链接 |
| `aarch64-unknown-linux-musl` | Linux ARM64 (静态) | ARM64 静态链接 |

#### ARM 架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `armv7-unknown-linux-musleabihf` | Linux ARMv7 (硬件浮点, 静态) | Raspberry Pi 静态版 |
| `armv7-unknown-linux-musleabi` | Linux ARMv7 (软件浮点, 静态) | 旧 ARM 设备静态版 |

#### 其他架构

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `powerpc64le-unknown-linux-musl` | Linux PowerPC64LE (静态) | IBM POWER 静态链接 |
| `riscv64gc-unknown-linux-musl` | Linux RISC-V 64-bit (静态) | RISC-V 静态链接 |
| `loongarch64-unknown-linux-musl` | Linux LoongArch 64-bit (静态) | 龙芯静态链接 |

### Android 系统

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `aarch64-linux-android` | Android ARM64 | 现代 Android 设备 |
| `i686-linux-android` | Android x86 | 模拟器和 x86 设备 |

### HarmonyOS (鸿蒙/OpenHarmony)

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `aarch64-unknown-linux-ohos` | HarmonyOS ARM64 | 华为鸿蒙系统 |
| `armv7-unknown-linux-ohos` | HarmonyOS ARMv7 | HarmonyOS ARMv7 架构 |

### WebAssembly

| 目标三元组 | 描述 | 说明 |
|----------|------|------|
| `wasm32-unknown-unknown` | WASM (独立) | 浏览器/独立 WASM |
| `wasm32-wasi` | WASM + WASI | 带系统接口的 WASM |

## 编译命令示例

### 本地编译 (native compilation)

```bash
# 直接为当前平台编译
cargo build --release
```

### 交叉编译 (cross-compilation)

使用 `cross` 工具进行交叉编译：

```bash
# 安装 cross 工具
cargo install cross

# 为特定目标编译
cross build --target aarch64-unknown-linux-gnu --release

# 其他示例
cross build --target armv7-unknown-linux-gnueabihf --release
cross build --target x86_64-unknown-linux-musl --release
cross build --target aarch64-linux-android --release
```

### 添加目标

```bash
# 如果尚未添加目标，需先添加
rustup target add aarch64-unknown-linux-gnu

# 列出所有已安装的目标
rustup target list | grep installed
```

## 架构说明

### 按处理器类型分类

| 处理器家族 | 包含的目标 |
|----------|----------|
| **x86/x64** | x86_64, i686, i586 |
| **ARM** | aarch64, armv7, arm |
| **PowerPC** | powerpc, powerpc64, powerpc64le |
| **RISC-V** | riscv64gc |
| **其他** | s390x, sparc64, loongarch64 |

### 按链接方式分类

| 链接方式 | 描述 | 目标后缀 |
|--------|------|--------|
| **GNU** | 动态链接 glibc | `-gnu` |
| **musl** | 静态链接 musl | `-musl` |
| **MSVC** | Windows MSVC 工具链 | `-msvc` |
| **Android** | Android NDK | `-android` |
| **OHOS** | 鸿蒙系统 | `-ohos` |

## 浮点支持

某些 ARM 目标分为两种变体：

- **`-eabihf`** (extended application binary interface, hard float)
  - 硬件浮点单元 (FPU)
  - 性能更好
  - 例如：`armv7-unknown-linux-gnueabihf`

- **`-eabi`** 或 **无后缀** (soft float)
  - 软件浮点运算
  - 兼容性更好
  - 例如：`armv7-unknown-linux-gnueabi`

## CI/CD 构建状态

GitHub Actions 工作流会自动为所有支持的目标编译二进制文件。详见 `.github/workflows/build.yml`。

## 常见问题

### Q: 如何为特定平台编译？
A: 使用 `cargo build --target <target-triplet> --release` 或使用 `cross` 工具进行交叉编译。

### Q: 什么时候使用 musl 版本？
A: 当需要在多个 Linux 发行版间运行二进制文件时，musl 版本更好，因为它不依赖 glibc。

### Q: ARM 和 ARMv7 有什么区别？
A: ARM 是通用的 ARM 支持，ARMv7 是针对 ARMv7 指令集的优化版本。

### Q: 我的平台不在列表中怎么办？
A: 提交 issue 或 PR 来添加新的目标平台。

## 更多信息

- [Rust 平台支持](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [cross 工具](https://github.com/cross-rs/cross)
- [rustup 目标列表](https://rust-lang.github.io/rustup/cross-compilation.html)

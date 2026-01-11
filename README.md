# AndroidChecker

![Version](https://img.shields.io/badge/version-3.4.72-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)

**AndroidChecker** 是由 **SmartDolphin Team** 开发的一款基于 Rust 的高性能 Android 设备安全审计与综合管理工具。它不仅提供了强大的命令行工具，还集成了现代化的 GUI 监控界面，旨在为开发者和安全研究人员提供一站式的 Android 设备管理解决方案。

## 下载安装

您可以直接在 [Releases](https://github.com/Astraeuszhao/AndroidChecker/releases) 页面下载最新的安装包。

- **Windows**: 下载 `AndroidChecker_Setup.exe`

## 核心功能

### 1. 深度安全审计
- **Root 环境检测**: 智能识别 `su` 二进制文件、Magisk、SuperSU 等 Root 权限管理工具及其残留痕迹。
- **Bootloader 状态**: 检查 Bootloader 解锁状态及完整性验证状态 (`verifiedbootstate`)。
- **系统完整性**: 验证 Android 安全补丁级别、SELinux 状态及构建指纹。

### 2. 备份与恢复
- **全量/增量备份**: 支持应用列表、用户文件 (`/sdcard`) 及应用数据 (需 Root) 的备份。
- **标准化格式**: 采用通用 `tar.gz` 格式打包 (`.adbbackup`)，便于手动提取和跨平台迁移。
- **一键恢复**: 支持从备份文件快速还原数据到设备。

### 3. 实时系统监控 (GUI)
- **资源概览**: 实时可视化展示 CPU、内存、磁盘 IO 和网络流量。
- **进程管理**: 查看运行中的进程详情，支持按名称过滤和强制结束进程。
- **应用管理**: 查看已安装应用详情、权限申请情况及签名信息。

### 4. 压力测试
- **ADB 稳定性测试**: 通过高频并发指令测试 ADB 连接的稳定性。

## 快速开始

### 运行环境
- Windows / macOS / Linux
- 需安装 [ADB](https://developer.android.com/studio/command-line/adb) (程序会自动尝试使用内置 vendor 目录)

### 构建指南

如果您希望从源码构建：

```bash
# 克隆仓库
git clone https://github.com/Astraeuszhao/AndroidChecker.git
cd AndroidChecker

# 构建 Release 版本
cargo build --release

# 运行
cargo run
```

## 注意事项

本项目目前处于 **v3.4.72** 稳定版，但仍请注意：
*   部分高级功能 (如应用数据备份) 需要设备拥有 Root 权限。
*   GUI 在 Windows 下体验最佳，Linux/macOS 可能需要额外配置字体。

## 许可证

本项目采用 [MIT License](LICENSE.txt) 开源授权。
Copyright (c) 2025 **SmartDolphin Team**.

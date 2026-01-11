# AndroidChecker

AndroidChecker 是一个基于 Rust 编写的 Android 设备安全审计与管理工具。集成了 ADB 通信、安全检查、数据备份以及实时系统监控（GUI）等功能。

## 功能特性

*   **设备连接管理**：自动检测并连接 ADB 设备。
*   **安全审计**：
    *   Root 环境检测 (检测 su 二进制文件、Magisk 等)。
    *   Bootloader 解锁状态检查。
    *   系统完整性与安全补丁检查。
*   **备份与恢复**：
    *   支持应用列表、用户文件、应用数据（需 Root）的备份与恢复。
    *   使用 tar.gz 格式打包。
*   **系统监控 (GUI)**：
    *   实时查看 CPU、内存、磁盘、网络使用情况。
    *   进程管理（查看、结束进程）。
    *   应用管理（查看详情、权限、签名）。
*   **压力测试**：简单的 ADB 连接稳定性测试。

## 运行环境

*   Windows / macOS / Linux
*   需安装 [ADB](https://developer.android.com/studio/command-line/adb) (或使用内置 vendor 目录)
*   构建需安装 [Rust](https://www.rust-lang.org/) 环境

## 构建与运行

```bash
# 构建 Release 版本
cargo build --release

# 运行
cargo run
```

## 注意事项

本项目目前处于开发阶段 (WIP)，代码中可能存在以下问题：
*   部分功能依赖 Root 权限。
*   GUI 部分使用了硬编码的中文字体路径 (Windows)。
*   部分错误处理尚待完善。

## License

MIT License

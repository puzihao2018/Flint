# 第三方组件

[English](../EN/THIRD-PARTY-NOTICES.md)

Flint 源码采用 MIT 许可证。第三方软件和模型各自适用上游许可证。

- **uv 0.12.5** 由 Astral 发布，适用 MIT 或 Apache-2.0。构建时从官方 Release 下载 macOS arm64 可执行文件，使用 scripts/uv.lock.json 中的两个 SHA-256 分别校验下载包和二进制。上游许可证位于 src-tauri/resources/licenses/，随应用分发。[uv 源码](https://github.com/astral-sh/uv/tree/0.12.5)
- **MinerU 4.0.12 / 4.1.2** 由 OpenDataLab 发布，按用户选择单独安装。其代码、依赖与模型保留各自许可证。Flint 的安装包不包含 MinerU 软件包或模型权重。[MinerU 源码](https://github.com/opendatalab/MinerU)
- Tauri、Rust crates、React、Vite、Lucide 等依赖由 Cargo.lock 和 package-lock.json 锁定。scripts/generate-notices.py 将依赖信息及可用的上游许可文本收集到应用的 Contents/Resources/resources/licenses/。该清单也包含构建依赖，不能据此推断每个包都链接到最终二进制。

更新依赖时请审查生成的清单。分发者需要根据实际用途评估许可证兼容性和模型再分发权利。本仓库不提供模型分发。

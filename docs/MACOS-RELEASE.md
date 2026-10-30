# macOS 发布指南

首发仅生成 **Apple Silicon / arm64** 安装包。Intel 需要单独的 uv、Python 依赖锁及真实 MinerU 验证；不要把当前二进制标记为 universal。

## 本地构建

在干净克隆中安装 Xcode Command Line Tools、Python 3.9+、Node 24 和 rustup，然后执行：

```sh
npm ci
npm test
npm run release:mac
cd release/0.1.12
shasum -a 256 -c SHA256SUMS.txt
```

产物：DMG（Flint.app + Applications 快捷链接）、ZIP、SHA256SUMS.txt、BUILD-INFO.json、MIT LICENSE 和 Release 说明。DMG 和 ZIP 中是同一个经过签名验证的应用。MinerU 包、模型、用户配置与日志不在发行包里。

资源脚本会校验 uv 固定下载包及二进制的 SHA-256；构建时从锁定依赖收集第三方声明，放入 app 资源。许可证清单覆盖本机目标依赖及构建依赖，是可审查的清单，不是法律兼容性保证。

当前签名为 **ad-hoc**。它用于验证本地包完整性，并非 Apple 对开发者身份或软件的公证。Release 必须保留这一说明。

## GitHub Actions

`.github/workflows/macos.yml` 使用 `macos-14` arm64 runner：PR / main 推送执行测试和打包，上传 Actions artifacts；版本标签再创建 **draft + prerelease**，供人工检查后发布。不会自动把草稿公开。

1. 确认 GitHub 仓库名称、所有者和可见性，推送源码。
2. 确认 main 的 CI 通过。
3. 核对 package.json、package-lock.json、Cargo.toml、Cargo.lock 和 tauri.conf.json 的版本一致；更新 CHANGELOG 和 RELEASE-NOTES。
4. 创建与版本一致的标签，例如 `v0.1.12`，推送标签。
5. 检查 Release 草稿附件，核验安装包和校验值后发布。

工作流拒绝不匹配的标签，以及覆盖已公开的同名 Release。GitHub 自动生成的源码归档不包含 uv 与模型；构建时按固定校验下载 uv。

GitHub 托管执行本身尚需在真实远端仓库验证。runner 类型依据 [GitHub 官方 runner 表](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)。

## Developer ID 签名与公证

如需让普通用户顺畅安装，准备 Apple Developer ID Application 证书和 Apple 公证凭据；不要将私钥或密码提交到仓库。按 [Tauri macOS 签名文档](https://v2.tauri.app/distribute/sign/macos/) 配置 `APPLE_SIGNING_IDENTITY` 和公证环境变量。Tauri 构建阶段可签名并公证 app。

打包已签名的 app 时使用：

```sh
python3 scripts/release-macos.py --skip-build --signing preserve
```

该模式拒绝 ad-hoc 签名，不会重新签坏已有 Developer ID 签名。它不替你申请证书，也不自动公证 DMG。对最终 DMG 使用 `xcrun notarytool submit --wait`，成功后 `xcrun stapler staple` 并验证，再重新生成校验值。只有实际通过验证后才能把 Release 的“未公证”说明改掉；ZIP 应包含已 stapled 的 app。

当前仓库没有配置 Developer ID 凭据或启用自动公证。

## 安装验证

- 打开 DMG，拖入 Applications，卸载镜像后启动应用。
- 从菜单栏打开/关闭主窗口；后台应不占 Dock。
- 用示例服务检查启动、健康状态、端口修改、停止与退出。
- 用选定 MinerU Release 实测 PDF 解析；模型下载耗时不属于普通 CI 测试。
- 更新前正常退出旧版 Flint，避免打断解析；数据目录保持不变。

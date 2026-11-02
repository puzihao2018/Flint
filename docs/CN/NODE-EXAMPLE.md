# Node Web App / Flint 0.2 示例

[English](../EN/NODE-EXAMPLE.md)

[示例项目和清单](../../examples/node-webapp)

前置条件：Flint 0.1.9+，登录环境可以找到 Node.js 20+ 和 npm。示例没有第三方依赖；package-lock.json 固定包结构。macOS 已验证，Linux 尚未实测；Windows 未声明支持。

1. 在 Flint 中导入本目录的 launcher.yaml，检查配置并保存。
2. 点击「安装项目依赖」（npm ci），完成后点击启动。
3. 打开 Flint 显示的地址，应该看到 Node Web App is running；/healthz 返回 200 JSON。
4. 停止，在配置中改端口，再启动验证地址随之改变。
5. 停止后原端口应释放。

默认端口 18373。程序读取 LAUNCHER_PORT/LAUNCHER_HOST，所以不用额外传 --port；独立运行也可用 `PORT=3000 npm start`（macOS/Linux）。清单运行采用 `node server.mjs`，不依赖 npm 的 shell 包装器。

Flint 注入的数据目录会创建；独立运行使用 .data。删除 Flint 服务不会删除这个项目及其依赖。`npm ci --ignore-scripts` 适用于这个无依赖示例，其他项目若需要安装脚本应按实际需求调整。

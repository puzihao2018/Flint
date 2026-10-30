# 架构与数据

React 主窗口和菜单栏 WebView 通过 Tauri commands 访问同一个 Rust Engine。Engine 管理配置、安装任务、服务进程组、健康检查和日志；关闭窗口不会关闭 Engine，明确退出会停止受管服务。

macOS 数据根目录：`~/Library/Application Support/local.flint.services/`。开发可用 `FLINT_DATA_DIR` 覆盖。

| 路径 | 内容 |
|---|---|
| `services.json` | 服务配置，原子写入 |
| `apps/mineru/<version>/env/` | 按版本独立的 Python 环境 |
| `runtimes/python/` | uv 管理的 Python |
| `models/mineru/`、`models/mineru-4.0.12/` | 模型；保留 4.1 旧路径以兼容既有安装 |
| `instances/<id>/data/` | 应用配置及持久数据 |
| `instances/<id>/service.log` | 安装和服务日志 |
| `cache/` | 可复用下载缓存 |

服务固定监听选定端口，冲突时失败；启动后检查端口是否归属受管进程组。停止先发 SIGINT，等待后升级到 SIGTERM/SIGKILL。意外退出保留中断记录，不根据旧 PID 自动接管或终止其他进程。主动脱离进程组的后台程序不受完整管理。

自定义命令在当前用户权限下执行，独立环境不是安全沙箱。命令参数 `${port}` 仅替换为已验证整数；其他 shell 语法保持原意。导入清单先展示配置，不立即执行。

当前没有独立 supervisor、自动重启和持续运行日志轮换。日志在下次启动时超过 8 MiB 才轮换；UI 返回最后 64 KiB。模型源目前没有锁定到不可变 commit。

清单 0.2 导入为自定义实例配置：appId/应用 version 与本机 id 分离，run/installSteps 持久化；旧配置缺省为空，兼容旧 shell command。结构化命令不拼接 shell 源码；args/env 只展开规定的四个变量。安装复用已有进程组、日志、取消流程，失败保持 installed=false，改工作目录或安装步骤使安装状态失效。导入器与离线 flint-manifest 校验器共用解析逻辑。

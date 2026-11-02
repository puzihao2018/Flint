# Python / uv Web App / Flint 0.2 示例

[English](../EN/PYTHON-EXAMPLE.md)

[示例项目和清单](../../examples/python-webapp)

前置条件：Flint 0.1.9+，登录环境可以找到 uv；需要 Python 3.9+，uv 也可能按其自身配置下载 Python。示例没有第三方依赖，附 uv.lock。macOS 已验证，Linux 尚未实测；Windows 未声明支持。

1. 在 Flint 中导入本目录 launcher.yaml，检查安装与启动配置后保存。
2. 点击「安装项目依赖」：uv sync --frozen --no-dev 在项目内创建 .venv。
3. 点击启动。默认端口 18374，首页显示 Python Web App is running，/healthz 返回 200 JSON。
4. 停止后改端口再启动，最后停止并确认端口释放。

示例展示 `${host}` / `${port}` 参数映射，启动使用 `uv run --frozen --no-sync python -u ...`，不会隐式同步依赖。也可独立运行 `python3 server.py --port 3000`。

持久数据使用 LAUNCHER_DATA_DIR；独立运行使用 .data。项目 .venv 属于外部项目，不会随着 Flint 中删除服务而卸载。Flint 自带的 MinerU uv 路径不是此清单的公共 API，清单要求可用的 uv 可执行程序。

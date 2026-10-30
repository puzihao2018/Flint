# Flint Release 安装约定（0.1.3）

应用身份 recipe 与上游发布版本 version 分开记录。应用市场以 releases 数组展示可安装版本，而不是只提供一个 latest。选择已有版本返回既有服务；选择新版本创建独立服务实例。安装操作始终使用实例记录的精确版本。

## 当前实现

- registry/mineru.json 的 releases：version、label、python、api、releaseUrl、notes。
- 每个支持的 Release 都有独立的 macOS arm64 Python 3.12 依赖锁；4.0.12 与 4.1.2 分别锁定全部依赖，不能只替换主包版本。
- apps/<recipe>/<version>/env 与 requirements.lock 保存环境及精确依赖。实例 ID、数据目录和模型目录隔离；4.1.2 保留原路径，避免迁移或重下现有环境。
- 同一应用每个版本一个实例；多个版本可以分别启停。创建新实例时选择未使用端口，此后冲突报错，不会默默换端口。
- 运行中不能改配置，版本不能原地覆盖，健康检查校验实际 version、API 特征和 model ID。
- 旧 services.json 没有 version 的 MinerU 实例按 4.1.2 兼容读取，并保留安装状态、配置及端口。

## 增加一个适配 Release

1. 确认上游官方 Release/tag 与包版本对应关系，在 releases 登记准确版本和协议差异。
2. 使用配方的 Python 与目标平台生成依赖锁；独立安装，检查系统依赖和命令参数。
3. 在安装器的嵌入锁映射中添加该版本；如启动命令、模型或 API 不同，添加对应适配。不能将所有版本视为相同 API。
4. 验证安装、取消/恢复、启动、端口、停止、真实文件解析，以及至少一个目标客户端的协议。通过后才加入随包发布的支持列表。

本版是“经过适配的多个 Release 可选安装”。尚未实现任意 GitHub 仓库 Release 的自动发现、远程配方更新、二进制资产下载及跨平台自动适配。未知版本会被拒绝；不会自动降级或用其他版本冒充。

## MinerU 兼容性

4.0.12 对外保留 /v1/uploads、/v1/parse/jobs 和 ZIP 文件结果，适配 llm-for-zotero 3.9.15。4.1.2 使用新版 /v1/tasks 协议。两者都仍叫 v1，不代表请求格式兼容。端口与 API Key 不会解决协议不兼容。

验证脚本：scripts/verify-mineru-4.0.py 与 scripts/verify-mineru.py。均仅向指定本地服务提交明确传入的测试文件，不遍历用户文献库。

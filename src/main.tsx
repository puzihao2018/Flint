import { version } from "../package.json";
import { BrandMark } from "./BrandMark";
import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Play,
  Square,
  ArrowUpRight,
  Copy,
  Check,
  Plus,
  Settings2,
  LayoutGrid,
  Package,
  Terminal,
  FolderOpen,
  X,
  ChevronRight,
  LoaderCircle,
  RefreshCw,
  BookOpen,
  Power,
  AlertCircle,
  Download,
  Server,
  Globe,
  ArrowLeft,
  Trash2,
} from "lucide-react";
import "./style.css";

type CommandSpec = {
  program: string;
  args: string[];
  env: Record<string, string>;
};
type Config = {
  appId?: string | null;
  run?: CommandSpec | null;
  installSteps?: CommandSpec[];
  version: string;
  id: string;
  name: string;
  recipe: string;
  port: number;
  command: string;
  cwd: string;
  protocol: string;
  healthPath: string;
  tier: string;
  modelSource: string;
  installed: boolean;
  modelsReady: boolean;
};
type Service = {
  config: Config;
  status: string;
  stage: string;
  error: string | null;
  pid: number | null;
  startedAt: number | null;
  endpoint: string;
  commandPreview: string;
};
type RemovalOptions = { environment: boolean; models: boolean; data: boolean };
type RemovalPreview = {
  managed: boolean;
  shared: boolean;
  items: { key: keyof RemovalOptions; path: string; bytes: number }[];
};
const formatBytes = (bytes: number) =>
  bytes >= 1024 ** 3
    ? `${(bytes / 1024 ** 3).toFixed(2)} GiB`
    : bytes >= 1024 ** 2
      ? `${(bytes / 1024 ** 2).toFixed(1)} MiB`
      : `${Math.ceil(bytes / 1024)} KiB`;

type Snapshot = {
  services: Service[];
  dataRoot: string;
  platform: string;
  catalog: {
    name: string;
    version: string;
    description: string;
    upstream: string;
    releases: {
      version: string;
      label: string;
      notes: string;
      releaseUrl: string;
    }[];
  };
};
const labels: Record<string, string> = {
  stopped: "已停止",
  starting: "启动中",
  running: "运行中",
  installing: "部署中",
  stopping: "停止中",
  removing: "删除中",
  error: "异常",
  unhealthy: "检查未通过",
  interrupted: "上次运行中断",
};
const active = (s: Service) =>
  [
    "running",
    "starting",
    "unhealthy",
    "installing",
    "stopping",
    "removing",
  ].includes(s.status);
const fresh = (): Config => ({
  version: "4.1.2",
  id: "",
  name: "",
  recipe: "custom",
  port: 3000,
  command: "",
  cwd: "",
  protocol: "http-api",
  healthPath: "/healthz",
  tier: "standard",
  modelSource: "huggingface",
  installed: true,
  modelsReady: true,
});
const tray = new URLSearchParams(location.search).has("tray");

function App() {
  const [data, setData] = useState<Snapshot | null>(null),
    [page, setPage] = useState("services"),
    [releaseVersion, setReleaseVersion] = useState("4.0.12"),
    [selected, setSelected] = useState("mineru"),
    [tab, setTab] = useState("overview"),
    [logs, setLogs] = useState(""),
    [modal, setModal] = useState<Config | null>(null),
    [error, setError] = useState(""),
    [notice, setNotice] = useState(""),
    [pending, setPending] = useState(false),
    [copied, setCopied] = useState("");
  const [removal, setRemoval] = useState<Config | null>(null);
  const [removalPreview, setRemovalPreview] = useState<RemovalPreview | null>(
    null,
  );
  const [removalOptions, setRemovalOptions] = useState<RemovalOptions>({
    environment: false,
    models: false,
    data: false,
  });
  const [removalError, setRemovalError] = useState("");
  const removalBytes =
    removalPreview?.items.reduce(
      (sum, item) => sum + (removalOptions[item.key] ? item.bytes : 0),
      0,
    ) ?? 0;

  const refresh = async () => {
    try {
      setData(await invoke<Snapshot>("snapshot"));
    } catch (e) {
      setError(String(e));
    }
  };
  useEffect(() => {
    void refresh();
    const t = setInterval(refresh, 1200);
    return () => clearInterval(t);
  }, []);
  useEffect(() => {
    if (tray && data)
      void getCurrentWindow()
        .setSize(
          new LogicalSize(390, Math.min(560, 144 + data.services.length * 130)),
        )
        .catch(() => {});
  }, [data?.services.length]);
  const current =
    data?.services.find((s) => s.config.id === selected) ?? data?.services[0];
  useEffect(() => {
    if (tab !== "logs" || !current) return;
    let alive = true;
    const read = () =>
      invoke<string>("service_logs", { id: current.config.id })
        .then((s) => {
          if (alive) setLogs(s);
        })
        .catch((e) => {
          if (alive) setLogs(String(e));
        });
    void read();
    const t = setInterval(read, 1200);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [tab, current?.config.id]);
  useEffect(() => {
    if (!notice) return;
    const t = setTimeout(() => setNotice(""), 3500);
    return () => clearTimeout(t);
  }, [notice]);
  const action = async (s: Service, a: string) => {
    setPending(true);
    try {
      await invoke("service_action", { id: s.config.id, action: a });
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setPending(false);
    }
  };
  const launch = async (s: Service) => {
    if (active(s)) await action(s, "stop");
    else await action(s, "start");
  };
  const removeService = async (config: Config) => {
    setModal(null);
    setRemoval(config);
    setRemovalOptions({ environment: false, models: false, data: false });
    setRemovalPreview(null);
    setRemovalError("");
    setPending(true);
    try {
      setRemovalPreview(
        await invoke<RemovalPreview>("removal_preview", { id: config.id }),
      );
    } catch (e) {
      setRemovalError(String(e));
    } finally {
      setPending(false);
    }
  };
  const confirmRemoval = async () => {
    if (!removal || !removalPreview) return;
    setPending(true);
    setRemovalError("");
    try {
      await invoke("remove_service", {
        id: removal.id,
        options: removalOptions,
      });
      const deletedFiles = Object.values(removalOptions).some(Boolean);
      setRemoval(null);
      setSelected("");
      setTab("overview");
      setLogs("");
      await refresh();
      setNotice(
        deletedFiles ? "服务及所选文件已删除" : "服务登记已删除，文件已保留",
      );
    } catch (e) {
      setRemovalError(String(e));
    } finally {
      setPending(false);
    }
  };
  const copy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(text);
      setTimeout(() => setCopied(""), 1600);
    } catch {
      setError("复制失败，请手动复制地址");
    }
  };
  const link = async (s: Service, target: string) => {
    try {
      await invoke("open_service", { id: s.config.id, target });
    } catch (e) {
      setError(String(e));
    }
  };
  const importManifest = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [
          { name: "Launcher manifest", extensions: ["yaml", "yml", "json"] },
        ],
      });
      if (typeof path === "string") {
        const config = await invoke<Config>("read_manifest", { path });
        setModal(config);
        setNotice("清单已读入，请检查命令后保存。尚未执行。");
      }
    } catch (e) {
      setError(String(e));
    }
  };
  const save = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!modal) return;
    setPending(true);
    try {
      const id = await invoke<string>("save_service", { config: modal });
      setModal(null);
      setSelected(id);
      setPage("services");
      await refresh();
      setNotice("配置已保存");
    } catch (e) {
      setError(String(e));
    } finally {
      setPending(false);
    }
  };
  const quit = async () => {
    if (
      data?.services.some(active) &&
      !confirm(
        "退出 Flint 将停止所有由它启动的服务，并取消进行中的部署。继续？",
      )
    )
      return;
    await invoke("quit_app");
  };
  const status = (s: Service) => (
    <span className={`status ${s.status}`}>
      <i />
      {labels[s.status] ?? s.status}
    </span>
  );
  const badge = (s: Service) => (
    <div className={`app-icon ${s.config.recipe === "mineru" ? "mineru" : ""}`}>
      {s.config.recipe === "mineru" ? "M" : <Server size={24} />}
    </div>
  );
  const control = (s: Service, small = false) => {
    const working = ["starting", "installing", "stopping", "removing"].includes(
      s.status,
    );
    return (
      <button
        className={`${small ? "icon-button" : "button"} ${active(s) ? "secondary" : "primary"}`}
        disabled={
          pending ||
          s.status === "stopping" ||
          s.status === "interrupted" ||
          (!active(s) && !s.config.installed) ||
          (!active(s) && s.config.recipe === "mineru" && !s.config.modelsReady)
        }
        role="switch"
        aria-checked={active(s)}
        aria-label={`${s.config.name} ${active(s) ? "停止" : "启动"}`}
        onClick={() => launch(s)}
        title={active(s) ? "停止" : "启动"}
      >
        <span className={`switch-track ${active(s) ? "on" : ""}`}>
          <span />
        </span>
        {working ? (
          <LoaderCircle className="spin" size={16} />
        ) : active(s) ? (
          <Square size={15} />
        ) : (
          <Play size={15} />
        )}
        {!small &&
          (active(s)
            ? s.status === "installing"
              ? "取消部署"
              : "停止服务"
            : "启动服务")}
      </button>
    );
  };
  if (tray)
    return (
      <div className="tray-shell">
        <header className="tray-head">
          <span className="brand">
            <BrandMark size={23} monochrome />
            Flint
          </span>
          <span className="muted">
            {data?.services.filter((s) => s.status === "running").length ?? 0}{" "}
            个运行中
          </span>
          <button className="icon-button" title="退出" onClick={quit}>
            <Power size={16} />
          </button>
        </header>
        {error && (
          <div className="tray-error" onClick={() => setError("")}>
            {error}
          </div>
        )}
        <div className="tray-list">
          {data?.services.map((s) => (
            <div className="tray-service" key={s.config.id}>
              <div className="tray-service-top">
                {badge(s)}
                <div className="tray-info">
                  <strong>{s.config.name}</strong>
                  {status(s)}
                </div>
                {s.config.installed || active(s) ? (
                  control(s, true)
                ) : (
                  <button
                    className="icon-button"
                    title="打开主窗口安装"
                    onClick={() => invoke("show_main")}
                  >
                    <Download size={17} />
                  </button>
                )}
              </div>
              <button
                className="tray-address"
                title="复制地址"
                onClick={() => copy(s.endpoint)}
              >
                {s.endpoint}
                {copied === s.endpoint ? (
                  <Check size={13} />
                ) : (
                  <Copy size={13} />
                )}
              </button>
              {s.stage && <p className="tray-stage">{s.stage}</p>}
            </div>
          ))}
        </div>
        <footer>
          <button onClick={() => invoke("show_main")}>
            <LayoutGrid size={16} />
            打开控制台
            <ArrowUpRight size={16} />
          </button>
        </footer>
      </div>
    );
  return (
    <div className="shell">
      <div
        className="titlebar"
        onMouseDown={(e) => {
          if (e.buttons === 1 && !(e.target as HTMLElement).closest("button"))
            void getCurrentWindow().startDragging();
        }}
      >
        <button
          className="text-button"
          onClick={() => invoke("toggle_quick_controls")}
        >
          快捷面板 <LayoutGrid size={13} />
        </button>
        <span>LOCAL SERVICES / 本地服务</span>
      </div>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <BrandMark size={27} />
          </div>
          <span>
            Flint<small>火石 · 本地运行</small>
          </span>
        </div>
        <nav>
          <button
            className={page === "services" ? "chosen" : ""}
            onClick={() => setPage("services")}
          >
            <LayoutGrid size={18} />
            我的服务
            <span className="nav-count">{data?.services.length ?? 0}</span>
          </button>
          <button
            className={page === "market" ? "chosen" : ""}
            onClick={() => setPage("market")}
          >
            <Package size={18} />
            应用市场
          </button>
        </nav>
        <div className="sidebar-section">
          <span>服务列表</span>
          <button
            className="icon-button"
            title="添加服务"
            onClick={() => setModal(fresh())}
          >
            <Plus size={15} />
          </button>
        </div>
        <div className="sidebar-services">
          {data?.services.map((s) => (
            <button
              key={s.config.id}
              className={
                selected === s.config.id && page === "services"
                  ? "selected"
                  : ""
              }
              onClick={() => {
                setSelected(s.config.id);
                setPage("services");
                setTab("overview");
              }}
            >
              <i className={`dot ${s.status}`} />
              {s.config.name}
              <span>:{s.config.port}</span>
            </button>
          ))}
        </div>
        <div className="sidebar-bottom">
          <div className="local-note">
            <span className="dot running" />
            <span>
              在你的电脑上运行<small>数据与环境由你掌控</small>
            </span>
          </div>
          <button
            className={page === "settings" ? "chosen" : ""}
            onClick={() => setPage("settings")}
          >
            <Settings2 size={18} />
            设置与存储
          </button>
        </div>
      </aside>
      <main>
        <div className="topline">
          <span>
            工作空间 <ChevronRight size={13} />{" "}
            {page === "market"
              ? "应用市场"
              : page === "settings"
                ? "设置"
                : (current?.config.name ?? "我的服务")}
          </span>
          <div className="top-actions">
            <button className="button quiet small" onClick={importManifest}>
              <FolderOpen size={15} />
              导入清单
            </button>
            <button
              className="button secondary small"
              onClick={() => setModal(fresh())}
            >
              <Plus size={15} />
              添加服务
            </button>
          </div>
        </div>
        {!data && (
          <div className="empty">
            <LoaderCircle className="spin" />
            <p>正在连接本地管理核心…</p>
          </div>
        )}
        {data && page === "market" && (
          <section className="page-body">
            <div className="eyebrow">THE TOOL SHELF</div>
            <h1>准备好，就能运行。</h1>
            <p className="lead">
              从独立环境到本地端点，Flint 帮你把服务安顿好。
            </p>
            <div className="market-card">
              <div className="market-cover">
                <span className="giant-m">M</span>
                <div>
                  DOCUMENT INTELLIGENCE<span>文档，变成可以使用的内容。</span>
                </div>
                <span className="version-chip">v{releaseVersion}</span>
              </div>
              <div className="market-content">
                <div>
                  <h2>MinerU</h2>
                  <span className="tag">HTTP API</span>
                  <span className="tag">独立 Python 环境</span>
                </div>
                <p>{data.catalog.description}</p>
                <label>
                  选择 Release
                  <select
                    value={releaseVersion}
                    onChange={(e) => setReleaseVersion(e.target.value)}
                  >
                    {data.catalog.releases.map((r) => (
                      <option key={r.version} value={r.version}>
                        {r.label}
                        {data.services.some(
                          (s) =>
                            s.config.recipe === "mineru" &&
                            s.config.version === r.version &&
                            s.config.modelsReady,
                        )
                          ? " · 已部署"
                          : ""}
                      </option>
                    ))}
                  </select>
                </label>
                <p className="form-hint">
                  {
                    data.catalog.releases.find(
                      (r) => r.version === releaseVersion,
                    )?.notes
                  }{" "}
                  各版本独立安装，可并存并分别设置端口。
                </p>
                <div className="market-footer">
                  <button
                    className="text-button"
                    onClick={() =>
                      current &&
                      link(
                        data.services.find((s) => s.config.id === "mineru")!,
                        "source",
                      )
                    }
                  >
                    GitHub · opendatalab
                    <ArrowUpRight size={14} />
                  </button>
                  <button
                    className="button primary"
                    disabled={pending}
                    onClick={async () => {
                      setPending(true);
                      try {
                        const id = await invoke<string>("add_release", {
                          recipe: "mineru",
                          version: releaseVersion,
                        });
                        await refresh();
                        setSelected(id);
                        setPage("services");
                      } catch (e) {
                        setError(String(e));
                      } finally {
                        setPending(false);
                      }
                    }}
                  >
                    查看与部署
                    <ChevronRight size={15} />
                  </button>
                </div>
              </div>
            </div>
            <div className="quiet-note">
              <BookOpen size={17} />
              <span>
                市场配方随此版本提供，源项目公开在
                GitHub。自定义服务可以直接添加，无需上架。
              </span>
            </div>
          </section>
        )}
        {data && page === "settings" && (
          <section className="page-body">
            <div className="eyebrow">A PLACE FOR EVERYTHING</div>
            <h1>各有其位。</h1>
            <p className="lead">应用、模型与下载缓存分开存放。</p>
            <div className="settings-card">
              <h3>数据目录</h3>
              <code className="path">{data.dataRoot}</code>
              <button
                className="button secondary small"
                onClick={() => copy(data.dataRoot)}
              >
                <Copy size={14} />
                复制路径
              </button>
              <div className="storage-grid">
                <div>
                  <strong>apps/</strong>
                  <span>应用独立环境</span>
                </div>
                <div>
                  <strong>models/</strong>
                  <span>持久模型文件</span>
                </div>
                <div>
                  <strong>instances/</strong>
                  <span>服务配置与日志</span>
                </div>
                <div>
                  <strong>cache/</strong>
                  <span>可重新获取的缓存</span>
                </div>
              </div>
            </div>
            <div className="settings-card">
              <h3>窗口与后台</h3>
              <p>
                macOS
                关闭主窗口后只保留菜单栏图标，服务继续运行。打开主窗口时恢复
                Dock 图标。
              </p>
              <p>
                明确退出会停止受管服务。进程意外中断时保留诊断记录，不接管其他应用。
              </p>
              <button className="button secondary" onClick={quit}>
                <Power size={15} />
                停止服务并退出
              </button>
            </div>
            <p className="footnote">
              Flint {version} · macOS Apple Silicon alpha · 三端适配仍需分别验证
            </p>
          </section>
        )}
        {data && page === "services" && !current && (
          <section className="page-body">
            <div className="eyebrow">YOUR SERVICES</div>
            <h1>还没有服务</h1>
            <p className="lead">
              从应用市场添加工具，或管理已有项目的启动命令。
            </p>
            <div className="actions">
              <button
                className="button primary"
                onClick={() => setPage("market")}
              >
                <Package size={16} />
                浏览应用市场
              </button>
              <button
                className="button secondary"
                onClick={() => setModal(fresh())}
              >
                <Plus size={16} />
                添加已有服务
              </button>
            </div>
          </section>
        )}
        {data && page === "services" && current && (
          <section className="page-body service-detail">
            <div className="service-heading">
              {badge(current)}
              <div>
                <div className="eyebrow">
                  {current.config.recipe === "mineru"
                    ? "DOCUMENT PARSING"
                    : "LOCAL SERVICE"}
                </div>
                <h1>{current.config.name}</h1>
              </div>
              <div className="heading-status">{status(current)}</div>
            </div>
            <p className="lead">
              {current.config.recipe === "mineru"
                ? "把文档解析留在本地。一个服务，随时可用。"
                : "你的命令，你的环境。运行状态，在这里一目了然。"}
            </p>
            <div className="tabs">
              <button
                className={tab === "overview" ? "active" : ""}
                onClick={() => setTab("overview")}
              >
                概览
              </button>
              <button
                className={tab === "logs" ? "active" : ""}
                onClick={() => setTab("logs")}
              >
                运行日志
              </button>
              <span>
                {current.config.recipe === "mineru"
                  ? `MinerU ${current.config.version}`
                  : current.config.protocol.toUpperCase()}
              </span>
            </div>
            {tab === "logs" ? (
              <div className="logs-panel">
                <div>
                  <span>
                    <Terminal size={15} />
                    服务输出
                  </span>
                  <button
                    className="text-button"
                    onClick={() => link(current, "logs")}
                  >
                    <FolderOpen size={14} />
                    打开日志文件
                  </button>
                </div>
                <pre>{logs}</pre>
              </div>
            ) : (
              <>
                <div className="endpoint-card">
                  <div className="endpoint-label">
                    <span className="section-label">服务端点</span>
                    <span className="tag">
                      {current.config.protocol === "mcp-http"
                        ? "MCP · HTTP"
                        : "HTTP API"}{" "}
                      · localhost
                    </span>
                  </div>
                  <div className="endpoint-row">
                    <code>{current.endpoint}</code>
                    <button
                      className="icon-button"
                      title="复制服务地址"
                      onClick={() => copy(current.endpoint)}
                    >
                      {copied === current.endpoint ? (
                        <Check size={19} />
                      ) : (
                        <Copy size={19} />
                      )}
                    </button>
                  </div>
                  <div className="endpoint-footer">
                    <span>
                      {current.status === "running"
                        ? current.config.protocol === "mcp-http"
                          ? "初始化通过 · 持续检查端口存活"
                          : "API 检查通过"
                        : current.config.installed
                          ? "启动后即可使用此地址"
                          : "完成部署后，服务会在这里启动"}
                    </span>
                    {current.config.recipe === "mineru" && (
                      <button
                        className="text-button"
                        disabled={current.status !== "running"}
                        onClick={() => link(current, "docs")}
                      >
                        API 文档
                        <ArrowUpRight size={14} />
                      </button>
                    )}
                  </div>
                </div>
                {current.stage && (
                  <div className="progress">
                    <LoaderCircle className="spin" size={17} />
                    <span>
                      {current.stage}
                      <small>详细进度可在运行日志中查看</small>
                    </span>
                    <button
                      className="text-button"
                      onClick={() => setTab("logs")}
                    >
                      查看日志
                      <ChevronRight size={14} />
                    </button>
                  </div>
                )}
                {current.error && (
                  <div className="service-error">
                    <AlertCircle size={18} />
                    <div>
                      {current.error}
                      {current.status === "interrupted" && (
                        <button
                          className="text-button"
                          onClick={() => action(current, "acknowledge")}
                        >
                          已确认旧进程结束，清除记录
                        </button>
                      )}
                    </div>
                  </div>
                )}
                <div className="detail-grid">
                  <div>
                    <label>监听端口</label>
                    <strong>
                      {current.config.port}
                      <button
                        className="text-button"
                        disabled={
                          active(current) || current.status === "interrupted"
                        }
                        onClick={() => setModal({ ...current.config })}
                      >
                        修改
                        <Settings2 size={13} />
                      </button>
                    </strong>
                    <small>固定端口，不自动换号</small>
                  </div>
                  <div>
                    <label>运行环境</label>
                    <strong>
                      {current.config.recipe === "mineru"
                        ? "Python 3.12.12"
                        : "已有项目环境"}
                    </strong>
                    <small>
                      {current.config.recipe === "mineru"
                        ? "Flint 独立管理"
                        : "保留项目原有文件"}
                    </small>
                  </div>
                  <div>
                    <label>部署状态</label>
                    <strong>
                      {!current.config.installed
                        ? "未安装"
                        : current.config.recipe === "mineru" &&
                            !current.config.modelsReady
                          ? "模型待准备"
                          : "已准备"}
                    </strong>
                    <small>
                      {current.config.recipe === "mineru"
                        ? {
                            standard: "标准解析 · 含 VLM",
                            basic: "基础解析 · OCR",
                            flash: "轻量模式 · 无模型预下载",
                          }[current.config.tier]
                        : "由你提供启动命令"}
                    </small>
                  </div>
                </div>
                <div className="actions">
                  {(!current.config.installed ||
                    (current.config.recipe === "mineru" &&
                      !current.config.modelsReady)) &&
                  !active(current) ? (
                    <button
                      className="button primary"
                      disabled={pending || current.status === "interrupted"}
                      onClick={() => action(current, "install")}
                    >
                      <Download size={16} />
                      {current.config.recipe === "mineru"
                        ? current.config.installed
                          ? "继续部署"
                          : "安装 MinerU"
                        : "安装项目依赖"}
                    </button>
                  ) : (
                    control(current)
                  )}
                  <button
                    className="button secondary"
                    disabled={
                      active(current) || current.status === "interrupted"
                    }
                    onClick={() => setModal({ ...current.config })}
                  >
                    <Settings2 size={16} />
                    配置服务
                  </button>
                  <button
                    className="button quiet"
                    onClick={() => setTab("logs")}
                  >
                    <Terminal size={16} />
                    查看日志
                  </button>
                  <button
                    className="button quiet danger"
                    disabled={
                      pending ||
                      current.status === "removing" ||
                      current.status === "interrupted"
                    }
                    title={
                      active(current)
                        ? "删除前会自动停止服务"
                        : current.status === "interrupted"
                          ? "请先处理上次中断记录"
                          : "删除服务，可选择卸载环境和模型"
                    }
                    onClick={() => removeService(current.config)}
                  >
                    <Trash2 size={16} />
                    删除服务
                  </button>
                </div>
                {!!current.config.installSteps?.length && (
                  <details className="command-details">
                    <summary>安装步骤（在项目工作目录执行）</summary>
                    <pre>
                      {JSON.stringify(current.config.installSteps, null, 2)}
                    </pre>
                    <button
                      className="button secondary"
                      disabled={
                        pending ||
                        active(current) ||
                        current.status === "interrupted"
                      }
                      onClick={() => action(current, "install")}
                    >
                      重新执行安装
                    </button>
                  </details>
                )}
                {current.config.appId && (
                  <p className="form-hint">
                    {current.config.appId} · {current.config.version}
                  </p>
                )}
                <details className="command-details">
                  <summary>
                    <Terminal size={14} />
                    启动命令
                    <ChevronRight size={14} />
                  </summary>
                  <pre>{current.commandPreview}</pre>
                  <p>停止由 Flint 处理，无需提供 stop command。</p>
                </details>
                <div className="quiet-note">
                  <Server size={16} />
                  <span>
                    {current.config.recipe === "mineru"
                      ? "首次部署会下载依赖和模型。进程、模型准备与 API 状态分别检查。"
                      : "命令中的 ${port} 会替换为上方端口；仅填写端口不会改变程序本身的行为。"}
                  </span>
                </div>
              </>
            )}
          </section>
        )}
      </main>
      {notice && (
        <div className="toast">
          <Check size={16} />
          {notice}
        </div>
      )}
      {error && (
        <div className="error-toast" role="alert">
          <AlertCircle size={18} />
          <span>{error}</span>
          <button
            className="icon-button"
            onClick={() => setError("")}
            title="关闭"
          >
            <X size={16} />
          </button>
        </div>
      )}
      {removal && (
        <div className="modal-backdrop">
          <section
            className="modal removal-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="removal-title"
            onKeyDown={(event) => {
              if (event.key === "Escape" && !pending) setRemoval(null);
            }}
          >
            <header>
              <h2 id="removal-title">删除 {removal.name}</h2>
            </header>
            <p>
              服务会先自动停止，再从列表中删除。选择是否同时清理以下内容；不勾选则仅删除登记。
            </p>
            {!removalPreview && !removalError && <p>正在统计占用空间…</p>}
            {removalPreview && (
              <>
                {(
                  [
                    ...(removalPreview.managed
                      ? ["environment", "models"]
                      : []),
                    "data",
                  ] as (keyof RemovalOptions)[]
                ).map((key) => (
                  <label className="removal-option" key={key}>
                    <input
                      type="checkbox"
                      checked={removalOptions[key]}
                      disabled={
                        pending || (key !== "data" && removalPreview.shared)
                      }
                      onChange={(event) =>
                        setRemovalOptions({
                          ...removalOptions,
                          [key]: event.target.checked,
                        })
                      }
                    />
                    <span>
                      <strong>
                        {
                          {
                            environment: "卸载独立运行环境",
                            models: "删除此版本的模型",
                            data: "删除服务数据、日志和专用缓存",
                          }[key]
                        }
                      </strong>
                      <small>
                        {formatBytes(
                          removalPreview.items
                            .filter((item) => item.key === key)
                            .reduce((sum, item) => sum + item.bytes, 0),
                        )}
                      </small>
                    </span>
                  </label>
                ))}
                {removalPreview.shared && (
                  <p>其他服务仍共用此版本，环境与模型会保留。</p>
                )}
                <p className="removal-summary">
                  所选内容约 {formatBytes(removalBytes)}
                  。实际释放空间可能因共享文件与磁盘快照而减少。
                </p>
                <p>
                  {removalPreview.managed
                    ? "共享 Python 和下载缓存保留，其他版本不受影响。"
                    : "外部项目目录和环境保留。"}
                  勾选的文件会直接删除，重新使用时可能需要下载。
                </p>
                <details>
                  <summary>查看清理位置</summary>
                  {removalPreview.items
                    .filter((item) => removalOptions[item.key])
                    .map((item) => (
                      <code className="removal-path" key={item.path}>
                        {item.path}
                      </code>
                    ))}
                </details>
              </>
            )}
            {removalError && (
              <p className="removal-error" role="alert">
                {removalError}
              </p>
            )}
            <footer>
              <button
                className="button secondary"
                disabled={pending}
                autoFocus
                onClick={() => setRemoval(null)}
              >
                取消
              </button>
              <button
                className="button primary"
                disabled={pending || !removalPreview}
                onClick={confirmRemoval}
              >
                {pending
                  ? "正在处理…"
                  : Object.values(removalOptions).some(Boolean)
                    ? "删除服务和所选文件"
                    : "仅删除登记"}
              </button>
            </footer>
          </section>
        </div>
      )}
      {modal && (
        <div className="modal-backdrop">
          <form className="modal" onSubmit={save}>
            <header>
              <div>
                <span className="eyebrow">SERVICE CONFIGURATION</span>
                <h2>{modal.id ? "配置服务" : "添加本地服务"}</h2>
              </div>
              <button
                type="button"
                className="icon-button"
                title="关闭"
                onClick={() => setModal(null)}
              >
                <X size={20} />
              </button>
            </header>
            <label>
              服务名称
              <input
                autoFocus
                required
                value={modal.name}
                onChange={(e) => setModal({ ...modal, name: e.target.value })}
              />
            </label>
            <div className="form-row">
              <label>
                端口
                <input
                  type="number"
                  required
                  min={1024}
                  max={65535}
                  value={modal.port}
                  onChange={(e) =>
                    setModal({ ...modal, port: Number(e.target.value) })
                  }
                />
              </label>
              <label>
                服务类型
                <select
                  disabled={modal.recipe === "mineru"}
                  value={modal.protocol}
                  onChange={(e) =>
                    setModal({
                      ...modal,
                      protocol: e.target.value,
                      healthPath:
                        e.target.value === "mcp-http" ? "/mcp" : "/healthz",
                    })
                  }
                >
                  <option value="http-api">HTTP API</option>
                  <option value="mcp-http">MCP · Streamable HTTP</option>
                </select>
              </label>
            </div>
            {modal.recipe === "mineru" ? (
              <>
                <p className="form-hint">
                  Release {modal.version} ·
                  在应用市场选择其他版本，已有环境将保留。
                </p>
                <div className="form-row">
                  <label>
                    解析配置
                    <select
                      value={modal.tier}
                      onChange={(e) =>
                        setModal({ ...modal, tier: e.target.value })
                      }
                    >
                      <option value="standard">标准 · OCR + VLM</option>
                      <option value="basic">基础 · OCR</option>
                      <option value="flash">轻量 · 原生文档 / 文本提取</option>
                    </select>
                  </label>
                  <label>
                    模型下载源
                    <select
                      value={modal.modelSource}
                      onChange={(e) =>
                        setModal({ ...modal, modelSource: e.target.value })
                      }
                    >
                      <option value="huggingface">Hugging Face</option>
                      <option value="modelscope">ModelScope</option>
                    </select>
                  </label>
                </div>
                <p className="form-hint">
                  改变解析配置后需重新准备对应模型。独立环境与已有下载会被复用。
                </p>
              </>
            ) : (
              <>
                <label>
                  工作目录
                  <div className="input-with-button">
                    <input
                      required
                      value={modal.cwd}
                      onChange={(e) =>
                        setModal({ ...modal, cwd: e.target.value })
                      }
                      placeholder="/path/to/project"
                    />
                    <button
                      type="button"
                      title="选择目录"
                      onClick={async () => {
                        const p = await open({
                          directory: true,
                          multiple: false,
                        });
                        if (typeof p === "string")
                          setModal({ ...modal, cwd: p });
                      }}
                    >
                      <FolderOpen size={18} />
                    </button>
                  </div>
                </label>
                {modal.appId && (
                  <p className="form-hint">
                    应用 {modal.appId} · {modal.version}
                  </p>
                )}
                {!!modal.installSteps?.length && (
                  <div>
                    <p className="form-hint">
                      保存不会执行命令。之后点击「安装项目依赖」，将按顺序在工作目录执行：
                    </p>
                    <pre
                      style={{
                        whiteSpace: "pre-wrap",
                        overflowWrap: "anywhere",
                      }}
                    >
                      {JSON.stringify(modal.installSteps, null, 2)}
                    </pre>
                  </div>
                )}
                <label>
                  {modal.run ? "结构化启动配置" : "启动命令"}
                  <textarea
                    readOnly={!!modal.run}
                    required={!modal.run}
                    rows={3}
                    value={
                      modal.run
                        ? JSON.stringify(modal.run, null, 2)
                        : modal.command
                    }
                    onChange={(e) =>
                      setModal({ ...modal, command: e.target.value })
                    }
                    placeholder={
                      "uv run myserver --host 127.0.0.1 --port ${port}"
                    }
                  />
                </label>
                <p className="form-hint">
                  {modal.run ? (
                    "参数按原样传递，不解释 shell 语法。修改命令或安装步骤请编辑清单后重新导入；每次导入创建新实例。"
                  ) : (
                    <>
                      使用前台常驻命令。支持 {"${port}"}{" "}
                      占位符；现有服务的依赖需已准备好。
                    </>
                  )}
                </p>
                <label>
                  {modal.protocol === "mcp-http"
                    ? "MCP 端点路径"
                    : "HTTP 健康检查路径"}
                  <input
                    value={modal.healthPath}
                    onChange={(e) =>
                      setModal({ ...modal, healthPath: e.target.value })
                    }
                    placeholder={
                      modal.protocol === "mcp-http"
                        ? "/mcp"
                        : "/（留空检查首页）"
                    }
                  />
                </label>
              </>
            )}
            <footer>
              {modal.id && (
                <button
                  type="button"
                  className="text-button danger"
                  disabled={pending}
                  onClick={() => removeService(modal)}
                >
                  删除服务
                </button>
              )}
              <button
                type="button"
                className="button secondary"
                onClick={() => setModal(null)}
              >
                取消
              </button>
              <button
                className="button primary"
                disabled={pending}
                type="submit"
              >
                保存配置
                <Check size={15} />
              </button>
            </footer>
          </form>
        </div>
      )}
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<App />);

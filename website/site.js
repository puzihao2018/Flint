const zh = {
  skip: "跳到正文",
  navFeatures: "功能",
  navDevelopers: "开发者",
  alpha: "macOS 测试版 · Apple Silicon",
  heroTitle: "让本地服务，<br>有一个<em>自己的家。</em>",
  heroDescription:
    "安装 MinerU，启动你的 Web App，随时管理后台服务。不必再到终端窗口里逐个寻找。",
  getFlint: "获取 Mac 版 Flint",
  viewSource: "查看源码",
  heroMeta: "MIT 许可证，在你的电脑上运行。",
  previewHeading: "你的本地工作空间",
  demoLabel: "交互演示",
  services: "我的服务",
  demoLocal: "本地服务",
  running: "运行中",
  stopped: "已停止",
  mineruKind: "文档解析 · HTTP API",
  webappKind: "你的项目 · HTTP API",
  mcpKind: "工具服务 · Streamable HTTP",
  endpoint: "服务端点",
  healthy: "✓ API 检查通过",
  mcpHealthy: "✓ MCP 初始化通过",
  stoppedHealth: "启动后即可使用此地址",
  port: "端口",
  environment: "运行环境",
  existing: "已有项目",
  stopService: "停止服务",
  startService: "启动服务",
  menuBar: "菜单栏",
  trayNote: "关闭主窗口，服务继续运行。",
  demoCaption: "试试开关。此演示不会启动真实服务。",
  builtFor: "为本地工作而造",
  everyday: "日常管理",
  featuresTitle: "少管理几个终端，<br><em>多做一点创造。</em>",
  featureInstall: "每个版本，各有其位。",
  featureInstallBody:
    "在独立环境中安装 MinerU 4.0 或 4.1。依赖、模型和端口分别管理，无需修改系统 Python。",
  featureInstallDetail: "多版本安装 · 独立运行环境",
  featureControl: "运行状态，一目了然。",
  featureControlBody:
    "启动、停止、修改端口，查看健康检查和运行日志。无需为每个服务再记一条停止命令。",
  featureControlDetail: "进程控制 · 健康检查 · 日志",
  featureTray: "关掉窗口，随时掌控。",
  featureTrayBody:
    "服务运行时，Flint 可以只留在 macOS 菜单栏。需要时打开快捷面板，用完便回到你的工作。",
  tryTray: "试试菜单栏演示",
  yourProject: "也支持你的项目",
  developerTitle: "一个小文件，<br><em>让启动更简单。</em>",
  developerBody:
    "继续使用熟悉的开发工具。添加一个 launcher.yaml，描述项目如何安装、启动，以及如何检查服务状态。",
  developerAI:
    "把接入指南交给编程 AI，让下一个 Web App 从创建之初就支持 Flint。",
  aiGuide: "给 AI 的接入指南",
  spec: "阅读接入规范",
  examples: "查看示例项目",
  copy: "复制",
  copied: "已复制",
  copyFailed: "复制失败，请选中文本复制",
  codeNote: "适用于 Python/uv 示例项目，使用项目已有目录。",
  getStarted: "开始使用",
  downloadTitle: "为下一个想法，<br><em>留好位置。</em>",
  downloadPlatform: "macOS 13+ · Apple Silicon",
  downloadBuild: "获取 macOS 构建",
  downloadNote:
    "打开一次成功的构建，在 Artifacts 中下载 Flint-macOS-arm64。需要登录 GitHub。",
  browseReleases: "查看 Releases",
  step1: "安装 Flint",
  step1Body: "解压构建附件，打开其中的 DMG，将 Flint 拖入 Applications。",
  step2: "添加你需要的服务",
  step2Body:
    "从应用市场选择 MinerU 版本，导入 launcher.yaml，或填写已有项目的启动命令。",
  step3: "打开启动开关",
  step3Body: "等待健康检查通过，将服务地址用于浏览器或需要它的客户端。",
  installationGuide: "完整安装指南",
  alphaNote:
    "早期测试版。安装包采用 ad-hoc 签名，尚未经过 Apple 公证。首次安装 MinerU 需要下载依赖和模型。",
  platformNote:
    "Windows、Linux、Intel Mac 和端口自动发现仍在计划中，尚未提供。",
  faqTitle: "你可能还想知道。",
  faq1: "MinerU 是 MCP server 吗？",
  faq1Body:
    "不是。内置 MinerU 配方提供 HTTP API。Flint 也支持使用 Streamable HTTP 的自定义 MCP 服务，目前不支持 MCP stdio 和旧版 SSE。",
  faq2: "我的项目必须使用专门的启动器吗？",
  faq2Body:
    "不用。可以直接添加前台启动命令与已有项目目录。launcher.yaml 让安装步骤、启动参数和健康检查更容易分享。程序本身需要真正支持配置的端口。",
  faq3: "删除服务时会删掉哪些文件？",
  faq3Body:
    "你可以只删除登记，也可以选择卸载受管环境、模型及服务数据。Flint 会保留外部项目目录和共享缓存。选中的受管文件会直接删除，不进入废纸篓。",
  faq4: "可以贡献新的应用适配吗？",
  faq4Body:
    "可以。先提交 Issue，说明应用和目标平台，并提供官方来源、固定版本、安装与启动命令和有效的健康检查。配方需要实际通过安装与启停测试，才能标记为已支持。",
  openDevelopment: "开放开发",
  communityTitle: "轻巧的工具，<br><em>一起慢慢完善。</em>",
  communityBody:
    "Flint 使用 MIT 许可证，以 Tauri、Rust 和 React 构建。查看路线图、反馈问题，或帮助更多服务接入 Flint。",
  contribute: "在 GitHub 上参与",
  roadmap: "查看路线图",
  issues: "问题反馈",
  changelog: "更新记录",
};
const nodes = [...document.querySelectorAll("[data-i18n], [data-i18n-html]")];
const en = Object.fromEntries(
  nodes.map((node) => [
    node.dataset.i18n || node.dataset.i18nHtml,
    node.dataset.i18nHtml ? node.innerHTML : node.textContent,
  ]),
);
Object.assign(en, {
  stopped: "Stopped",
  webappKind: "Your project · HTTP API",
  mcpKind: "Tools · Streamable HTTP",
  mcpHealthy: "✓ MCP initialization passed",
  stoppedHealth: "Start the service to use this address",
  existing: "Existing project",
  startService: "Start service",
  copied: "Copied",
  copyFailed: "Copy failed. Select the text to copy it.",
});
let locale = "en";
try {
  if (localStorage.getItem("flint-site-language") === "zh-CN") locale = "zh-CN";
} catch {}
let selected = "mineru";
const serviceData = {
  mineru: {
    name: "MinerU 4.0.12",
    short: "MinerU",
    icon: "M",
    port: 8001,
    environment: "Python 3.12",
    running: true,
  },
  webapp: {
    name: "My Web App",
    short: "Web App",
    icon: "↗",
    port: 3000,
    environment: "Node.js",
    running: false,
  },
  mcp: {
    name: "MCP Server",
    short: "MCP Server",
    icon: "⌘",
    port: 8080,
    environment: "existing",
    running: false,
  },
};
const $ = (id) => document.getElementById(id);
const t = (key) => (locale === "zh-CN" ? zh[key] : en[key]) || en[key] || key;
function renderService(announce = false) {
  const service = serviceData[selected];
  document.querySelectorAll("[data-service]").forEach((button) => {
    const active = button.dataset.service === selected;
    button.setAttribute("aria-selected", String(active));
    button.tabIndex = active ? 0 : -1;
    button.querySelector(".mini-dot").style.background = serviceData[
      button.dataset.service
    ].running
      ? "#5c8d6d"
      : "#abb3a2";
  });
  $("demo-detail").setAttribute("aria-labelledby", "tab-" + selected);
  $("demo-icon").textContent = service.icon;
  $("demo-title").textContent = service.name;
  $("demo-kind").textContent = t(selected + "Kind");
  $("demo-url").textContent =
    "127.0.0.1:" + service.port + (selected === "mcp" ? "/mcp" : "");
  $("demo-port").textContent = service.port;
  $("demo-environment").textContent =
    service.environment === "existing" ? t("existing") : service.environment;
  const state = t(service.running ? "running" : "stopped");
  $("demo-state").classList.toggle("stopped", !service.running);
  $("demo-state").querySelector("span").textContent = state;
  $("demo-health").textContent = t(
    service.running
      ? selected === "mcp"
        ? "mcpHealthy"
        : "healthy"
      : "stoppedHealth",
  );
  $("demo-action").textContent = t(
    service.running ? "stopService" : "startService",
  );
  $("service-toggle").setAttribute("aria-checked", String(service.running));
  $("tray-service").textContent = service.short;
  $("tray-state").textContent = state;
  $("tray-url").textContent = $("demo-url").textContent;
  if (announce)
    $("demo-announcement").textContent = service.name + " — " + state;
}
function setLanguage(language) {
  locale = language;
  document.documentElement.lang = locale;
  document.title =
    locale === "zh-CN"
      ? "Flint — 本地服务启动器"
      : "Flint — A home for your local services";
  nodes.forEach((node) => {
    const key = node.dataset.i18n || node.dataset.i18nHtml;
    if (node.dataset.i18nHtml) node.innerHTML = t(key);
    else node.textContent = t(key);
  });
  $("language").textContent = locale === "en" ? "中文" : "EN";
  $("language").setAttribute(
    "aria-label",
    locale === "en" ? "Switch to Simplified Chinese" : "Switch to English",
  );
  $("tray-toggle").setAttribute(
    "aria-label",
    locale === "en" ? "Show menu bar demo" : "查看菜单栏演示",
  );
  document
    .querySelector(".service-tabs")
    .setAttribute("aria-label", locale === "en" ? "Demo services" : "演示服务");
  document
    .querySelector(".demo-window")
    .setAttribute(
      "aria-label",
      locale === "en" ? "Interactive Flint product demo" : "Flint 交互演示",
    );
  $("user-guide").href =
    "https://github.com/puzihao2018/Flint/blob/main/docs/" +
    (locale === "en" ? "USER-GUIDE.md" : "USER-GUIDE_CN.md");
  renderService();
  try {
    localStorage.setItem("flint-site-language", locale);
  } catch {}
}
$("language").addEventListener("click", () =>
  setLanguage(locale === "en" ? "zh-CN" : "en"),
);
document.querySelectorAll("[data-service]").forEach((button) => {
  button.addEventListener("click", () => {
    selected = button.dataset.service;
    renderService(true);
  });
  button.addEventListener("keydown", (event) => {
    const tabs = [...document.querySelectorAll("[data-service]")];
    const index = tabs.indexOf(button);
    let next;
    if (event.key === "ArrowDown") next = (index + 1) % tabs.length;
    if (event.key === "ArrowUp") next = (index + tabs.length - 1) % tabs.length;
    if (event.key === "Home") next = 0;
    if (event.key === "End") next = tabs.length - 1;
    if (next !== undefined) {
      event.preventDefault();
      tabs[next].click();
      tabs[next].focus();
    }
  });
});
$("service-toggle").addEventListener("click", () => {
  serviceData[selected].running = !serviceData[selected].running;
  renderService(true);
});
function setTray(open) {
  $("tray-preview").hidden = !open;
  $("tray-toggle").setAttribute("aria-expanded", String(open));
}
$("tray-toggle").addEventListener("click", () =>
  setTray($("tray-preview").hidden),
);
$("show-tray").addEventListener("click", () => {
  setTray(true);
  $("tray-toggle").scrollIntoView({
    block: "center",
    behavior: matchMedia("(prefers-reduced-motion: reduce)").matches
      ? "instant"
      : "smooth",
  });
  $("tray-toggle").focus({ preventScroll: true });
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") setTray(false);
});
document.addEventListener("click", (event) => {
  if (!event.target.closest("#tray-preview, #tray-toggle, #show-tray"))
    setTray(false);
});
$("copy-manifest").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText($("manifest-code").textContent);
    $("copy-manifest").textContent = t("copied");
    $("copy-status").textContent = t("copied");
  } catch {
    $("copy-status").textContent = t("copyFailed");
    $("copy-manifest").textContent = t("copyFailed");
  }
});
setLanguage(locale);

const zh = {
  skip: "跳到正文",
  navDevelopers: "开发者",
  alpha: "macOS 测试版 · Apple Silicon",
  heroTitle: "启动和管理<br><em>本地服务。</em>",
  heroDescription:
    "从 Mac 菜单栏启停 Web App、HTTP API 和 MCP 服务，在 Flint 中设置端口、查看日志。",
  getFlint: "获取 Mac 版 Flint",
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
  developerTitle: "接入你的项目",
  developerBody:
    "填写项目目录和启动命令，或用 launcher.yaml 分享安装步骤、端口和健康检查设置。",
  aiGuide: "给 AI 的接入指南",
  examples: "查看示例项目",
  copy: "复制",
  copied: "已复制",
  copyFailed: "复制失败，请选中文本复制",
  downloadTitle: "安装 Flint",
  downloadBuild: "下载 Mac 版",
  downloadNote:
    "打开 DMG，把 Flint 拖入「应用程序」。",
  installationGuide: "使用指南",
  issues: "问题反馈",
  changelog: "更新记录",
  navInstall: "安装",
  exampleTitle: "Python 示例 · launcher.yaml",
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
      : "Flint — Local service launcher";
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
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") setTray(false);
});
document.addEventListener("click", (event) => {
  if (!event.target.closest("#tray-preview, #tray-toggle")) setTray(false);
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

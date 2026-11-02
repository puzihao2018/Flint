# 项目网站维护

[English](../EN/WEBSITE.md)

网站源码位于 website/，通过 GitHub Pages 部署到[项目官网](https://zihaopu.cn/Flint/?lang=zh-CN)。页面使用静态 HTML、CSS 和 JavaScript，无需后端、远程字体或运行时依赖。

## 本地预览

```sh
npm run site:build
python3 -m http.server 4173 --bind 127.0.0.1 --directory dist-site
```

打开 http://127.0.0.1:4173。构建脚本检查本地资源、页内锚点、重复 ID 和中文翻译覆盖，仅将网站文件复制到 Git 忽略的 dist-site/。页面示例会导出为 launcher-example.yaml，可从仓库根目录运行 `npm run manifest:check -- "$PWD/dist-site/launcher-example.yaml"` 校验。

## 编辑与部署

- index.html 包含英文文案、页面结构、链接和交互演示的无障碍标记。site.js 管理中文文案、语言偏好、切换服务、菜单栏预览和复制按钮。style.css 定义响应式布局、焦点和减少动画模式。assets/ 使用 Flint 自有图形。
- 演示只模拟操作，不连接用户本机服务。下载按钮直达对应 GitHub Release 的 DMG。版本变更时同时更新官网、两份 README 和两份使用指南。
- 在仓库 Settings → Pages 将 Source 设为 GitHub Actions。pages.yml 在 PR 检查网站，推送到 main 后部署 dist-site/。设置仓库首页为 Pages 地址。网站修改无需提升桌面应用版本。
- 发布前检查中英文及各自的文档目标、手机窄屏、键盘操作、服务切换、菜单预览、复制和静态资源加载。

页面参考了 [Quay](https://abhi.am/quay) 的本地服务演示和 [Pinokio](https://desktop.pinokio.co/) 的下载与开发者入口。布局、文案、代码和品牌图形由 Flint 项目提供。

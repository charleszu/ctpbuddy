# CTPBuddy OINK 文档站

独立的 Hugo 文档站目录，使用官方 `pgsty/oink-starter` 作为起点，并将主题源码固定在 `themes/oink/`，版本为 OINK `v1.1.0`。旧的 `docs-site/` 静态草稿已移除。

## 来源与边界

- OINK 官网：<https://oink.pgsty.com>；它是 OINK 1.1 Hugo-only 框架官网，不是 CTPBuddy 的授权域名。
- Starter：<https://github.com/pgsty/oink-starter>。
- 主题源码：`themes/oink/`，来自 `github.com/pgsty/oink` v1.1.0，Apache-2.0；源码随仓库提供，clone 后不依赖嵌套 submodule。
- `go.mod` / `go.sum` 保留 OINK 的固定版本元数据；当前构建优先使用内置主题源码，便于离线复现。
- 当前站点只启用简体中文；没有复制 `docs/notes/assets/compiled_html/` 下的官方 HTML 大文件。
- `content-starter-reference/` 保留官方 starter 示例作为升级/结构参考，但不参与 Hugo 内容构建。

## 工具链

官方 starter 的测试工具链为 Go `1.27.0` 与 Hugo Extended `0.165.0`。本项目已在 Hugo Extended `0.167.0` 上构建通过；配置要求 Hugo Extended `>=0.165.0`。Go 不是当前内置主题源码构建的运行时依赖；不要使用普通版 Hugo，也不要用未固定的 `@latest`。

## 本地构建

在本目录执行：

```bash
hugo server --bind 127.0.0.1
hugo --cleanDestinationDir --gc --minify --environment production \
  --printPathWarnings --panicOnWarning
```

当前构建使用随仓库提供的 `themes/oink/`，不要求运行时下载模块；执行 `hugo --gc --minify` 即可离线复现。`public/`、`resources/`、`.hugo_cache/` 均不应提交。

本目录仅做本地构建验证，不包含发布动作，不配置生产部署。发布前必须先取得实际域名授权、确认 `baseURL`、检查链接和审阅 OINK Apache-2.0 归属要求。

## 内容维护

内容采用本地摘要与原仓库链接：

- 首页、项目、架构、开发指南、语义知识库、API 原文索引、路线图由 `content/` 维护。
- 根项目的 `README.md`、`DESIGN.md` 和 `docs/` 是事实来源；不要在本站复制整份官方 API HTML。
- 若以后需要完整章节，应优先按章节导入并保留来源说明，而不是批量拷贝生成物。

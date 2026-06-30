# 项目规范

FM Tester：Tauri 2.0 + Vue 3 桌面 API 测试工具（类 Postman）。前端纯 CSS（无 UI 框架），Rust 后端遵循 DDD 分层。

## 构建与开发命令

- 安装依赖：`npm install`
- 完整开发：`npm run tauri dev`（**勿单独跑 `npm run dev`**，那只是 Vite，缺 Rust 后端）
- 前端构建：`npm run build`
- 格式化：`npm run format`（prettier，提交前跑）
- Rust 编译检查：`cargo check`（在 `src-tauri/` 下）
- 打包安装包：`cargo tauri build`

> 前端**无单元测试**（`npm run test` 的 vitest 当前无用例），测试以 Rust 后端 `cargo test` 为准。

> AI 禁止自动启动应用 / 服务进程。`npm run tauri dev` 等启动命令由用户手动执行。

## 提交前验证

改动后至少跑相关项再视为完成：

- 前端：`npm run lint`（eslint src）、`npm run build`
- 后端（均在 `src-tauri/` 下）：`cargo fmt --check` → `cargo clippy --all-targets -- -D warnings`（零警告）→ `cargo test`
- 版本一致性：`node scripts/check-versions.mjs`

## 架构

后端分层（`src-tauri/src/`，依赖方向严格自上而下，勿反向引用）：

- `domain/` — 核心业务，不含基础设施依赖：`models/`、`repositories/`（trait 接口）、`services/`（压测、脚本执行等纯逻辑）
- `application/services/` — 协调领域服务 + 仓储
- `infrastructure/` — `sqlite/` 仓储实现、`http_client.rs`(reqwest)、`encryption.rs`(AES-GCM)、`js_runtime.rs`(rquickjs)、`scheduler.rs` 等
- `interface/commands/` — `#[tauri::command]` 薄封装

前端入口：`src/App.vue` → `src/App.js` 的 `useAppSetup()`。状态逻辑集中在 `src/composables/`（`useRequest.js`、`useWorkspace.js` 等）。

**前端约定**：

- **组件拆分**：组件由 `index.vue`（模板 + `<script setup>` 入口）+ `index.js`（导出 `useXxxSetup(props, emit)` 组合式函数）组成，状态/事件逻辑放 hook 里，不在 `<script setup>` 内联。
- **i18n 双语同步**：新增界面文案必须同时改 `src/locales/zh-CN.json` 与 `src/locales/en.json`，调用 `t("key.path")`；遗漏任一会漏翻译。

数据持久化（以 `src-tauri/src/infrastructure/data_dir.rs` 为准）：

- `<exe_dir>/data/config.json` — 全局配置（工作区列表 + 设置）
- `<exe_dir>/data/<workspace_id>/data.db` — 各工作区独立 SQLite（WAL 模式）
- `<exe_dir>/data/.encryption_key` — AES-GCM 密钥（凭据等敏感数据）

## 添加功能的固定套路

新增 Tauri 命令：

1. `src-tauri/src/interface/commands/<域>.rs` 定义 `#[tauri::command]`
2. `src-tauri/src/lib.rs` 的 `invoke_handler![]` 列表注册（遗漏会导致前端调用 404）
3. 前端 `invoke('<command>', { ... })`

新增领域仓储：

1. trait → `domain/repositories/<name>_repository.rs`
2. 实现 → `infrastructure/sqlite/sqlite_<name>_repository.rs`
3. 工厂方法 → `infrastructure/repository_factory.rs`
4. 各 `mod.rs` 导出

## 版本号管理

版本号分布在 5 处，升级时必须同步：

- `package.json` — `version`
- `src-tauri/Cargo.toml` — `version`
- `src-tauri/tauri.conf.json` — `version`
- `src/components/StatusBar/index.vue` — `version` 常量（带 `v` 前缀）
- `src/components/SettingsCenter/index.vue` — `version` 常量（设置页「关于」面板，带 `v` 前缀）

校验：`node scripts/check-versions.mjs`

## Git 与发布

- 不要主动 `commit` / `push`，仅在用户明确要求时执行
- 活跃开发分支 `dev`，默认分支 `master`
- 发布：推送 `v*` tag 触发 `.github/workflows/release.yml`，矩阵构建 windows/macos/linux（macOS 出 universal 二进制）

## 关键约束与陷阱

- **git2 / OpenSSL**：`git2` 启用 `vendored-openssl`（让 `openssl-sys` 从源码编译）与 `libgit2-sys` 的 `vendored` 特性（libgit2 用 cc 构建），CI（macOS/Linux）无需预装系统 OpenSSL。libgit2 的 HTTPS：Windows 走原生 WinHTTP/Schannel、macOS 走 Secure Transport、Linux 走 OpenSSL。HTTP 客户端走 reqwest + rustls，不依赖 OpenSSL。
- **勿改 Monaco 相关配置**：`vite.config.js` 的 `cacheDir: false` 与 `tauri.conf.json` 的 `csp: null` 都是刻意的——重新开启缓存或收紧 CSP 会导致 Tauri 打包后 Monaco Worker 加载失败、语法高亮丢失。
- **固定端口**：Vite 开发端口锁定 `1420`（`strictPort`），Tauri 依赖此端口，不可随意改。
- **变量语法**：URL/Headers/Body 用 `{{变量名}}` 引用环境/集合变量。
- **脚本执行**：经 rquickjs(QuickJS) 在后端运行；前置脚本顺序为 工作区 → 环境 → 父集合 → 子集合 → 接口，后置脚本反向。

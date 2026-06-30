# FM Tester

一个基于 Tauri + Vue 3 开发的 API 测试工具，类似于 Postman 的桌面应用。

## 功能特性

### 🚀 核心功能
- **接口测试**：支持 GET、POST、PUT、PATCH、DELETE、HEAD、OPTIONS 等常用 HTTP 方法
- **请求配置**：
  - URL 参数（Query Parameters）
  - 请求头（Headers）
  - 请求体（Body）：支持 none、form-data、x-www-form-urlencoded、raw、binary 多种格式
  - Raw 编辑器：支持 JSON、Text、JavaScript、HTML、XML，带语法高亮和格式化
- **响应查看**：实时显示响应状态码、响应头、响应体（JSON/XML/HTML 格式化显示）
- **请求历史**：按日期存储请求记录，方便对比和回顾

### 📁 工作区管理
- **多工作区**：支持创建多个独立工作区，隔离不同项目的数据
- **快速切换**：快速在不同工作区之间切换
- **工作区设置**：每个工作区独立的集合、环境变量、脚本等配置

### 📂 集合管理
- **层级结构**：支持多层集合嵌套（最多 3 层），方便组织接口
- **拖拽排序**：支持同级拖拽排序和跨层级移动
- **集合变量**：集合级别的变量继承，支持 `{{variableName}}` 引用
- **公共请求头**：集合级别的公共请求头，自动应用到所有接口
- **集合设置**：为集合配置前置/后置脚本、变量、公共请求头

### 🔧 环境管理
- **多环境支持**：支持多环境配置（开发、测试、生产等）
- **环境变量**：URL/Headers/Body 支持 `{{变量名}}` 替换
- **环境切换**：快速切换不同环境，变量自动替换
- **环境脚本**：环境级别的前置/后置脚本

### 🤖 AI 助手
- **智能对话**：集成 AI 聊天功能，支持 OpenAI 协议 API
- **模型选择**：自动获取可用模型列表，或手动输入模型名称
- **对话历史**：保存聊天会话，支持历史查看和管理
- **多 API 支持**：兼容 OpenAI、Azure、本地部署等多种 API 端点
- **AI 辅助**：AI 生成 API 文档、优化脚本代码

### 📥 导入导出
- **OpenAPI 导入**：导入前预览 OpenAPI/Swagger 文档结构，一键导入所有接口
- **Postman 导入**：支持 Postman Collection 2.1 格式导入
- **Postman 导出**：导出集合为 Postman Collection 2.1 格式
- **curl 导入**：解析 curl 命令并创建接口
- **curl 导出**：将接口导出为 curl 命令，复制到剪贴板

### ⚡ 脚本引擎
- **前置脚本**：请求发送前执行，可修改请求参数
- **后置脚本**：响应返回后执行，可处理响应数据
- **控制台日志**：脚本执行日志实时显示，方便调试
- **fm API**：
  - `fm.environment.get/set/remove` - 环境变量操作
  - `fm.collection.get/set/remove` - 集合变量操作
  - `fm.request.getUrl/setUrl` - URL 操作
  - `fm.request.getBaseUrl/setBaseUrl` - baseUrl 操作
  - `fm.request.getPath/setPath` - 路径操作
  - `fm.request.getMethod/setMethod` - 方法操作
  - `fm.request.getHeader/setHeader/removeHeader` - 请求头操作
  - `fm.request.getParam/setParam/removeParam/getParams` - Query 参数操作
  - `fm.request.getBody/setBody` - 请求体操作
  - `fm.response.getStatus/getStatusText/getHeader/getBody/getJson/getTime/getSize` - 响应数据访问
  - `fm.log/assert/sleep` - 工具方法
- **执行顺序**：前置脚本（工作区 → 环境 → 父集合 → 子集合 → 接口），后置脚本反向执行

### 🍪 Cookie 管理
- **自动管理**：自动保存响应中的 Cookie
- **手动编辑**：支持添加、编辑、删除 Cookie
- **请求携带**：发送请求时自动携带匹配的 Cookie

### ⚡ 压力测试
- **并发测试**：配置并发数、总请求数或持续时间
- **实时进度**：显示实时 QPS、成功率、响应时间分布
- **统计分析**：P50/P90/P95/P99 响应时间、状态码分布
- **断言验证**：配置响应状态码、响应时间、响应体断言
- **结果保存**：保存压力测试结果，方便对比分析

### 🔀 编排管理
- **API 编排**：串联多个 API 依次执行
- **步骤管理**：配置每个步骤的请求参数、变量传递
- **执行记录**：保存每次编排的执行结果，方便追溯

### 💾 数据保存
- **响应快照**：保存完整的请求/响应快照，方便对比
- **请求历史**：按日期分目录存储，便于追溯

### 🌐 国际化
- **多语言支持**：支持中文（简体）、英文
- **语言切换**：设置面板切换语言，自动保存

### ✨ 用户体验
- **变量高亮**：输入框中自动高亮显示变量引用
- **Toast 提示**：友好的操作提示和错误信息
- **自动格式化**：JSON/XML/HTML 自动格式化显示
- **Monaco Editor**：专业的代码编辑器体验
- **JSON5 支持**：编辑支持注释和尾逗号，发送时转换为标准 JSON

## 安装

从 [GitHub Releases](https://github.com/forestcnz/fm-tester/releases) 下载对应平台的安装包：

### Windows
- 下载 `.msi` 或 `.exe` 安装包
- 双击安装

### macOS
- 下载 `.dmg` 文件
- 双击打开，拖拽到 Applications

### Linux
- 下载 `.deb` 或 `.AppImage` 文件
- `.deb`: `sudo dpkg -i fm-tester_*.deb`
- `.AppImage`: `chmod +x fm-tester_*.AppImage && ./fm-tester_*.AppImage`

## 开发

### 环境要求
- Node.js 20+
- Rust stable
- pnpm 或 npm

### 本地开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 验证编译

```bash
cd src-tauri
cargo check
```

### 构建发布

```bash
cargo tauri build
```

GitHub Actions（`.github/workflows/release.yml`）在推送 `v*` 开头的 tag（如 `v0.6.8`）时触发，矩阵构建 Windows / macOS（universal 二进制）/ Linux 安装包并自动发布到 Releases。

## 技术栈

- **前端框架**：Vue 3 + Composition API
- **桌面框架**：Tauri 2.0
- **UI 组件**：自定义组件 + Monaco Editor（纯原生 CSS，无 UI 框架）
- **国际化**：vue-i18n
- **后端语言**：Rust
- **HTTP 客户端**：reqwest
- **数据库**：rusqlite（SQLite，WAL 模式）
- **脚本引擎**：rquickjs（QuickJS）
- **构建工具**：Vite 6
- **包管理**：npm

## 项目结构

```
fm-tester/
├── src/                          # Vue 前端代码
│   ├── assets/                   # 静态资源
│   ├── components/               # Vue 组件
│   │   ├── AISettingsPanel/     # AI 设置面板
│   │   ├── ChatPanel/           # AI 聊天面板
│   │   ├── CollectionSettingsPanel/  # 集合设置面板
│   │   ├── ConsolePanel/        # 控制台面板（脚本日志）
│   │   ├── CookiePanel/         # Cookie 管理面板
│   │   ├── CurlImportDialog/    # curl 导入对话框
│   │   ├── DocPanel/            # 文档面板
│   │   ├── EnvironmentPanel/    # 环境变量面板
│   │   ├── HeaderAutocomplete/  # 请求头自动补全
│   │   ├── HistoryDetailPanel/  # 历史详情面板
│   │   ├── Icon/                # 图标组件
│   │   ├── ImportDialog/        # OpenAPI/Postman 导入对话框
│   │   ├── MenuBar/             # 菜单栏
│   │   ├── OrchestrationEditor/ # 编排编辑器
│   │   ├── RequestPanel/        # 请求配置面板
│   │   ├── ResponsePanel/       # 响应查看面板
│   │   ├── SavedResponseDetail/ # 保存的响应详情
│   │   ├── SavedResponseDocPanel/ # 保存响应的 MD 文档预览
│   │   ├── SaveResponseDialog/  # 保存响应对话框
│   │   ├── ScriptPanel/         # 脚本编辑面板
│   │   ├── SettingsPanel/       # 全局设置面板
│   │   ├── Sidebar/             # 侧边栏（集合、环境、历史、WebSocket、编排等）
│   │   ├── StatusBar/           # 状态栏
│   │   ├── StressTestPanel/     # 压力测试面板
│   │   ├── TabsBar/             # 标签页栏
│   │   ├── TitleBar/            # 标题栏（自定义窗口装饰）
│   │   ├── Toast/               # Toast 提示组件
│   │   ├── VariableHighlight/   # 变量高亮组件
│   │   ├── WebSocketDetailPanel/# WebSocket 详情面板
│   │   ├── WebSocketPanel/      # WebSocket 配置面板
│   │   ├── WorkspaceDialog/     # 工作区对话框
│   │   ├── WorkspaceImportDialog/ # 工作区导入对话框
│   │   └── WorkspaceSettingsPanel/  # 工作区设置面板
│   ├── composables/             # Vue Composition API hooks
│   │   ├── useDialogStack.js    # 对话框栈管理
│   │   ├── useEnvironment.js    # 环境变量管理
│   │   ├── useI18n.js           # 国际化设置
│   │   ├── useKeyboardShortcuts.js # 键盘快捷键
│   │   ├── useMonacoTheme.js    # Monaco 主题注册
│   │   ├── useOrchestrationExecution.js # 编排执行（全局定时调用）
│   │   ├── useOrchestrationSchedule.js  # 编排定时任务
│   │   ├── useRequest.js        # 请求管理
│   │   ├── useResponse.js       # 响应处理
│   │   ├── useSettings.js       # 全局设置
│   │   ├── useTabs.js           # 标签页管理
│   │   ├── useTheme.js          # 主题切换
│   │   ├── useToast.js          # Toast 提示
│   │   ├── useWebSocket.js      # WebSocket 状态
│   │   ├── useWorkspace.js      # 工作区管理
│   │   └── useWorkspaceIO.js    # 工作区导入导出
│   ├── i18n/                    # 国际化配置
│   ├── locales/                 # 语言包（zh-CN.json, en.json）
│   └── utils/                   # 工具函数
│       ├── markdown.js          # Markdown 渲染（含 DOMPurify 消毒）
│       ├── scriptEngine.js      # 脚本执行引擎
│       └── syntax-highlight.js  # 语法高亮
├── src-tauri/                    # Rust 后端代码
│   ├── src/
│   │   ├── lib.rs              # 入口：注册所有 Tauri 命令
│   │   ├── main.rs             # 程序入口
│   │   ├── error_macro.rs      # 错误宏（将 anyhow/String 转为前端可读字符串）
│   │   ├── domain/             # 领域层（核心业务逻辑，无基础设施依赖）
│   │   │   ├── models/         # 数据结构定义
│   │   │   ├── repositories/   # 仓储接口（trait）
│   │   │   └── services/       # 领域服务（纯业务逻辑，含压测、脚本执行等）
│   │   ├── application/        # 应用层（协调业务）
│   │   │   └── services/       # 应用服务（协调领域服务 + 仓储）
│   │   ├── infrastructure/     # 基础设施层（持久化与外部集成）
│   │   │   ├── sqlite/          # SQLite 仓储实现（WAL 模式）
│   │   │   │   ├── connection.rs    # 连接池管理
│   │   │   │   ├── schema.rs        # 数据库 schema
│   │   │   │   └── sqlite_*_repository.rs # 各领域仓储实现
│   │   │   ├── repository_factory.rs # 仓储工厂
│   │   │   ├── json_app_config.rs   # 全局配置（config.json）
│   │   │   ├── data_dir.rs          # 数据目录管理
│   │   │   ├── encryption.rs        # AES-GCM 加密（凭据等）
│   │   │   ├── http_client.rs       # reqwest HTTP 客户端
│   │   │   ├── ai_http_client.rs    # AI HTTP 客户端（SSE 流式）
│   │   │   ├── js_runtime.rs        # rquickjs 脚本运行时
│   │   │   ├── scheduler.rs         # 定时任务调度（croner）
│   │   │   ├── sse_client.rs        # SSE 客户端
│   │   │   └── ws_client.rs         # WebSocket 客户端
│   │   └── interface/          # 接口层（Tauri 命令）
│   │       └── commands/       # #[tauri::command] 函数
│   ├── capabilities/             # Tauri 权限配置
│   └── Cargo.toml               # Rust 依赖配置
├── scripts/                       # 辅助脚本（生成图标等）
├── .github/workflows/            # GitHub Actions 发布流程（tag 触发多平台构建）
└── package.json                  # Node.js 项目配置
```

## 数据持久化

应用数据存储在 `<exe_dir>/data/` 目录下：

- `config.json` — 全局配置（工作区列表、设置等）
- `.encryption_key` — 加密密钥（用于凭据等敏感数据的 AES-GCM 加密）
- `<workspace_id>/data.db` — 各工作区独立的 SQLite 数据库（WAL 模式）

每个工作区数据完全隔离，包含：集合、接口、环境、历史、脚本、压测结果、编排、WebSocket 配置等。


## License

MIT

## 作者

forestcnz
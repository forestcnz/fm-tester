# FM Tester 插件系统设计稿（通用扩展平台 / UI 原型 / Manifest / 协议 / 架构）

> 版本: 2.0 (草案) | 日期: 2026-06-29 | 关联：`docs/plugin-system-design.md`
>
> 方向：插件 = **通用应用扩展平台**，中心原语是**命令(Commands)** 与 **视图(Views)**；UI 自主度 = **声明式(JSON-UI/Markdown/HTML) + 嵌入式 Webview**。既有 API 测试扩展点降为**可选能力组**。
>
> 含：ASCII UI 原型、`plugin.json` 完整示例、FMPP 协议时序（含 Webview）、架构图、前端组件树、Tauri 命令清单、典型插件（含非 API 测试场景）。

---

## 一、UI 原型

### 1.1 设置中心 —「插件」分类（管理面板）

```
┌─ 设置 ────────────────────────────────────────────────────────────────┐
│  插件                                                  [×]            │
│  管理 CLI 插件，扩展 FM Tester（命令 / 视图 / Webview）              │
│                                                                       │
│ ┌─────────────┐ ┌───────────────────────────────────────────────────┐ │
│ │ 通用        │ │ ┌──────────────────────────────────────────────┐  │ │
│ │ AI          │ │ │ + 安装插件        ▾  本地 / URL / 市场        │  │ │
│ │ Git         │ │ └──────────────────────────────────────────────┘  │ │
│ │ 外观        │ │ 已安装 (3)        [全部启用] [全部禁用]            │ │
│ │ 快捷键      │ │                                                   │ │
│ │▶插件        │ │ ┌──────────────────────────────────────────────┐  │ │
│ │ 关于        │ │ │ ▣ JSON Explorer              ▁ 运行中 ●      │  │ │
│ └─────────────┘ │ │   com.example.json-explorer · v1.0.0          │  │ │
│                 │ │   交互式 JSON 树浏览（Webview 视图）           │  │ │
│                 │ │   提供: 视图·命令    权限: 上下文读·控制台      │  │ │
│                 │ │                              [配置 ▾]          │  │ │
│                 │ └──────────────────────────────────────────────┘  │ │
│                 │ ┌──────────────────────────────────────────────┐  │ │
│                 │ │ ▣ Local Dashboard            ▁ 运行中 ●      │  │ │
│                 │ │   com.acme.dashboard · v0.3.0                │  │ │
│                 │ │   内部服务实时看板（侧边栏 + 标签页）          │  │ │
│                 │ │   提供: 视图·命令·AI工具  权限: 网络·变量读写  │  │ │
│                 │ └──────────────────────────────────────────────┘  │ │
│                 │ ┌──────────────────────────────────────────────┐  │ │
│                 │ │ ▢ Internal Signer            ▁ 已禁用 ○      │  │ │
│                 │ │   com.acme.req-signer · v0.4.1               │  │ │
│                 │ │   为内部网关注入签名头（可选请求钩子）         │  │ │
│                 │ │   提供: 钩子          权限: 变量读写  [配置 ▾] │  │ │
│                 │ └──────────────────────────────────────────────┘  │ │
│                 └───────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
```

卡片：启用勾选 · 状态点(●运行/○禁用/⚠错误) · id·版本 · 描述 · 提供(命令/视图/钩子…) · 权限摘要 · `[配置 ▾]`。

### 1.2 「配置 ▾」下拉

```
┌──────────────────────┐
│ 查看详情 (manifest)   │
│ 权限管理…             │
│ 审计日志…             │
│ 设置优先级…           │
│ 检查更新              │
│──────────────────────│
│ 卸载                  │
└──────────────────────┘
```

### 1.3 安装对话框（来源 → 解析 → 授权）

```
┌─ 安装插件 ────────────────────────────────────────┐
│  来源:  ( ) 本地  (•) URL  ( ) 市场               │
│  URL:  https://example.com/json-explorer-1.0.zip  │
│        ☑ 校验 checksum (sha256:…)                 │
│                              [取消]  [解析清单 ▸]  │
└───────────────────────────────────────────────────┘
        │ 解析成功
        ▼
┌─ 安装 JSON Explorer v1.0.0 ───────────────────────┐
│  来源: example.com   信任:  Trusted(校验一致)      │
│  申请权限：                                        │
│   ☑ 读取上下文     fm.context.read                │
│   ☑ 写控制台       fm.console.log                 │
│   ☑ 嵌入 Webview   webview.embed                  │
│   ☐ 网络请求       network.outbound               │
│   ☐ 读取凭据       fm.secret.read  [选 key…]      │
│   ☑ 安装后默认启用                                 │
│  ⚠ CLI 插件以独立进程运行，Webview 已沙箱隔离。    │
│                       [取消] [拒绝部分] [授权并安装]│
└────────────────────────────────────────────────────┘
```

### 1.4 权限管理面板

```
┌─ 权限管理 — com.acme.dashboard ────────────────────┐
│  [✓]网络  [✓]变量读  [✓]变量写  [ ]控制台  [ ]凭据 │
│   secret.read 可访问 key: (无)  [+ 添加]           │
│   fs.write 作用域: 仅自身目录 (.fm/)               │
│                              [保存]   [取消]       │
└────────────────────────────────────────────────────┘
```

### 1.5 插件视图的承载位置（核心）

**(a) 侧边栏：插件视图入口**

```
┌─ Sidebar ────────┐
│ 集合             │
│ 环境             │
│ 历史记录         │
│──────────────────│
│ ▣ JSON Explorer  │  ← 插件视图(com.example.json-explorer)
│ ▣ 看板           │  ← 插件视图(com.acme.dashboard)
└──────────────────┘
```

**(b) 主区标签页：PluginViewHost 承载（声明式 或 沙箱 Webview）**

```
┌─ Tabs ─────────────────────────────────────────────────────────────┐
│ [接口A] [接口B]  [▣ JSON Explorer ✕]   [▣ 看板 ✕]                  │
├────────────────────────────────────────────────────────────────────┤
│  PluginViewHost                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  (渲染模式二选一，由视图声明)                                  │ │
│  │                                                              │ │
│  │  ① 声明式 JSON-UI：                                          │ │
│  │     ## JSON 树                                               │ │
│  │     ▸ root                                                   │ │
│  │       ▸ users [3]                                            │ │
│  │     [展开全部]  [复制路径]   ← button → view/action 回调       │ │
│  │                                                              │ │
│  │  ② 嵌入式 Webview（沙箱 iframe）：                           │ │
│  │     <html> 插件自带页面                                       │ │
│  │       [搜索框] [树] [图表(echarts)] [表单]                    │ │
│  │        ↕ postMessage ↔ 主窗口桥 ↔ FMPP ↔ 插件 CLI            │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

### 1.6 命令面板：聚合所有插件命令

```
┌─ 命令面板 (Ctrl+K) ──────────────────────────────────┐
│ 🔍 搜索或运行命令…                                    │
│ ─ 插件 ─                                             │
│   ▣ 格式化 JSON        (JSON Explorer)               │
│   ▣ 看板: 刷新所有     (Local Dashboard)             │
│   ▣ 复制为内部 SDK     (Signer · 右键集合时可用)     │
│ ─ 内置 ─                                              │
│   发送请求 / 切换环境 / 打开设置 …                    │
└──────────────────────────────────────────────────────┘
```

### 1.7 控制台：插件日志（带来源标签 + 可过滤）

```
┌─ Console ─────────────────────────────────────────────────────────┐
│ [全部 ▾]  [插件 ▾]                                                 │
│ 12:01:03  ℹ [JSON Explorer]  loaded: 3 root keys                   │
│ 12:01:03  ℹ [dashboard]       refreshing metrics…                  │
│ 12:01:04  ⚠ [dashboard]       svc-b timeout (retries=2)            │
│ 12:01:04  ✖ [req-signer]      sign failed: key revoked [ABORT]     │
└────────────────────────────────────────────────────────────────────┘
```

---

## 二、`plugin.json` 完整示例

### 2.1 通用插件：JSON Explorer（命令 + Webview 视图，最全演示）

```jsonc
{
  "manifestVersion": 1,
  "id": "com.example.json-explorer",
  "name": "JSON Explorer",
  "version": "1.0.0",
  "author": "Example Labs <labs@example.com>",
  "description": "交互式 JSON 树浏览器，支持超大文档与路径复制。",
  "homepage": "https://github.com/example/json-explorer",
  "license": "MIT",
  "engines": { "fmTester": "^0.9.0" },

  "entry": {
    "command": { "windows": "${dir}/backend.exe", "macos": "${dir}/backend", "linux": "${dir}/backend" },
    "args": ["--stdio"],
    "transport": "stdio-ndjson",
    "idleTimeoutMs": 60000,
    "restartOnError": true,
    "maxRestarts": 5
  },

  "webview": {
    "entry": "webview/index.html",
    "csp": "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'none';"
  },

  "capabilities": {
    "commands": [
      {
        "id": "format",
        "title": { "zh-CN": "格式化 JSON", "en": "Format JSON" },
        "icon": "braces",
        "placement": ["commandPalette", "menu.tools"],
        "input": { "type": "object", "properties": { "text": { "type": "string" } } },
        "output": "markdown"
      }
    ],
    "views": [
      {
        "id": "explorer",
        "title": { "zh-CN": "JSON Explorer", "en": "JSON Explorer" },
        "icon": "tree",
        "placement": ["sidebar", "tab"],
        "render": "webview",                 // 用嵌入式 Webview 渲染
        "defaultOpen": "sidebar"
      }
    ]
  },

  "permissions": ["fm.console.log", "fm.context.read", "fm.clipboard.write", "webview.embed"],
  "priority": 100,
  "checksum": "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
}
```

### 2.2 声明式命令插件（cli-once，单脚本，无 Webview）

```jsonc
{
  "manifestVersion": 1,
  "id": "com.example.qrcode",
  "name": "QR Code",
  "version": "0.1.0",
  "author": "Example",
  "description": "把选中文本/链接生成二维码（声明式渲染）。",
  "engines": { "fmTester": "^0.9.0" },
  "entry": { "command": "${dir}/qr.py", "transport": "cli-once", "interpreter": "python" },

  "capabilities": {
    "commands": [
      {
        "id": "generate",
        "title": { "zh-CN": "生成二维码", "en": "Generate QR Code" },
        "placement": ["commandPalette", "menu.tools"],
        "input": { "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"] },
        "output": "json-ui"
      }
    ]
  },
  "permissions": ["fm.console.log"]
}
```

调用：`python qr.py command/invoke --input '{"commandId":"generate","params":{...}}'` → stdout 单行 JSON-UI：
```jsonc
{ "type": "image", "src": "data:image/png;base64,iVBOR..." }
```

### 2.3 可选能力组：Internal Signer（请求钩子，opt-in）

```jsonc
{
  "manifestVersion": 1,
  "id": "com.acme.req-signer",
  "name": "Internal Signer",
  "version": "0.4.1",
  "author": "ACME",
  "description": "为内部网关注入签名头（失败即中止请求）。",
  "engines": { "fmTester": "^0.9.0" },
  "entry": { "command": "${dir}/signer", "transport": "stdio-ndjson", "idleTimeoutMs": 120000 },

  "capabilities": {
    "hooks": [
      { "kind": "beforeRequest", "onError": "abort", "timeoutMs": 5000 }
    ],
    "commands": [
      { "id": "rotateKey", "title": { "zh-CN": "轮换签名密钥", "en": "Rotate Signing Key" }, "placement": ["commandPalette"] }
    ]
  },
  "permissions": ["fm.variable.read", "fm.variable.write", "fm.console.log", "fm.secret.read"],
  "secrets": ["acme-signing-key"]          // 声明可读的凭据 key
}
```

---

## 三、FMPP 协议时序

### 3.1 命令调用（声明式输出）

```
应用                              插件(JSON Explorer)
  │ command/invoke (req id=1)        │
  │ { commandId:"format",            │
  │   params:{ text:"{...}" },       │
  │   context:{ workspace, sel } }   │
  │─────────────────────────────────▶│
  │                                   │
  │ result (id=1)                     │
  │ { output:{                        │
  │     type:"markdown",              │
  │     content:"```json\n{...}\n```" │
  │   } }                             │
  │◀─────────────────────────────────│
```

### 3.2 嵌入式 Webview：加载 + 双向交互

```
主窗口 / iframe                  主窗口 Bridge              插件 CLI(JSON Explorer)
   │                               │                            │
   │ ① 用户点开「JSON Explorer」视图                              │
   │       应用指示加载插件资源                                   │
   │       Bridge ──加载 fm-plugin://json-explorer/webview/index.html──▶ iframe
   │                                                               │
   │ ② iframe 加载完成                                            │
   │ ◀── loaded ──                                                 │
   │       Bridge ──webview/ready(notify)────────────────────────▶│ CLI
   │                                                               │ CLI 可主动 webview/postMessage 初始化
   │ ◀──postMessage({type:"init", data})── Bridge ◀──webview/postMessage── │
   │                                                               │
   │ ③ 用户在插件页输入查询，点「搜索」                            │
   │ ──postMessage({type:"invoke", method:"search", q})──▶ Bridge  │
   │                              Bridge ──webview/message(req)──▶│ CLI
   │                                                               │ CLI 处理（可 fm/* 反向调宿主）
   │                              Bridge ◀──result────────────── │ CLI
   │ ◀──postMessage({type:"result", rows})── Bridge ◀──────────── │
   │                                                               │
   │ ④ 插件需要宿主能力（读变量）                                  │
   │ ──postMessage({type:"fm", method:"variable.get", key})─▶Bridge│
   │                              Bridge ──权限校验(fm.variable.read)──▶ fm/variable.get → 执行
   │ ◀──postMessage({type:"result", value})── Bridge ◀──────────── │
```

> Webview 永远不直连网络（CSP connect-src 'none'）；所有数据经 CLI 或宿主网关（权限校验后）。

### 3.3 可选钩子：失败(abort) 示例

```
应用                              req-signer(beforeRequest onError=abort)
  │ hook/beforeRequest (req id=5)       │
  │ { request:{...} }                   │
  │────────────────────────────────────▶│  …签名失败…
  │ error (id=5)                        │
  │ { code:-32000,                      │
  │   message:"key revoked",            │
  │   data:{ fatal:true } }             │
  │◀────────────────────────────────────│
  │ ▶ onError=abort → 终止请求          │
  │ ▶ 控制台 ✖[req-signer] + Toast      │
```

---

## 四、架构图

### 4.1 整体（含 Webview 桥）

```
┌──────────────────── 前端 (Vue 3) ────────────────────────────────────┐
│  SettingsCenter▸插件  PluginManagerPanel  usePlugins.js              │
│  命令面板 / MenuBar / Sidebar / TabsBar / 右键菜单 / ConsolePanel     │
│        │ invoke()                                          ▲ log/event│
│  PluginViewHost ──┬─ JsonUiRenderer(声明式树)                │        │
│                   └─ 沙箱 iframe(插件 Webview) ─postMessage──┤        │
└───────────────────┼──────────────────────────────────────────┼────────┘
                    │ Tauri 命令 + Webview asset 协议            │
┌───────────────────▼──── 后端 (Rust, DDD) ───────────────────┼────────┐
│  interface/commands/plugin.rs   (#[tauri::command])         │        │
│  application/services/plugin_service.rs ── PluginDispatcher ─┘        │
│  domain/services/plugin_domain.rs   (manifest/权限/CSP/编排)          │
│  domain/repositories/plugin_repository.rs (trait)                    │
│  domain/models/plugin.rs                                             │
│  infrastructure/                                                     │
│    plugin_runner.rs ─────── NDJSON/JSON-RPC ──────────────┐          │
│    plugin_webview_bridge.rs (asset协议 + iframe桥 + CSP)  │          │
│    plugin_registry_repository.rs / plugin_installer.rs    │          │
│  可选接线: http_service / import_service / ai_tool_service│          │
└───────────────────────────────────────────────────────────┼──────────┘
                                                            │ stdio
                                          ┌─────────────────▼──────────┐
                                          │  插件 CLI 进程 (任意语言)   │
                                          │   ┌─ webview/ (自带前端) ──┐│ ← 经 asset 协议加载进沙箱 iframe
                                          │   │  index.html / app.js   ││
                                          │   └────────────────────────┘│
                                          └────────────────────────────┘
```

### 4.2 `PluginRunner` + `WebviewBridge` 内部

```
PluginRunner (per enabled plugin)
├── ProcessHandle          spawn(entry) + env 注入
├── NdjsonCodec            行缓冲 ↔ JSON-RPC 帧
├── PendingCalls           id → oneshot (并发复用)
├── ReverseDispatcher      插件→应用 fm/* (权限校验后执行)
├── HealthSupervisor       崩溃计数/退避重启/连续失败自禁用
├── IdleTimer              idleTimeoutMs 回收
└── AuditSink              fm/* + Webview 宿主访问 → 审计日志

PluginWebviewBridge (per plugin webview view)
├── AssetProtocol          fm-plugin://<id>/... → 插件目录资源 + CSP 注入
├── SandboxPolicy          iframe sandbox(无 same-origin)、来源校验
├── MessageRouter          iframe postMessage ↔ webview/message 帧
└── HostCapabilityGate     Webview → fm/* 宿主能力的权限网关
```

---

## 五、前端组件树

```
components/
├── SettingsCenter/                (categories 新增 plugins)
├── PluginManagerPanel/            ← 新增 index.vue + index.js + style.css
├── PluginInstallDialog/           ← 新增（来源/解析/跳转授权）
├── PluginPermissionDialog/        ← 新增（权限审查/授权/降权）
├── PluginAuditLogPanel/           ← 新增（按插件查看审计）
├── PluginViewHost/                ← 新增：按 render 模式承载视图
│   ├── index.vue / index.js
│   └── (内含) 沙箱 iframe 容器
├── PluginSidebarEntry/            ← 新增：插件注入侧边栏入口
├── JsonUiRenderer/                ← 新增：JSON-UI 树 → 原生组件（含 chart via echarts）
└── (既有) 命令面板 / MenuBar / Sidebar / TabsBar / 右键菜单 / ConsolePanel
        └─ 聚合 list_plugin_commands / list_plugin_views 注入入口

composables/
└── usePlugins.js                  ← 新增：列表/启停/安装/权限/视图/日志
```

遵循约定：`index.vue`（模板 + `<script setup>` 入口）+ `index.js`（导出 `useXxxSetup`）；状态逻辑放 hook。

---

## 六、Tauri 命令清单（`interface/commands/plugin.rs`）

```text
// 仓库/列表
list_plugins()                                  → Vec<PluginDescriptor>      // 含启用态/能力/信任级别
get_plugin_manifest(id)                         → PluginManifest
// 安装/卸载/启停/升级
install_plugin(source, opts?)                   → ParsedManifest             // 仅解析，返回待授权清单
confirm_install(id, grantedPermissions, enable) → ()                         // 授权落盘后启用
uninstall_plugin(id, keepConfig?)               → ()
enable_plugin(id) / disable_plugin(id)          → ()
upgrade_plugin(id, source)                      → PluginDescriptor
// 通用能力（命令/视图）
list_plugin_commands(placement?)                → Vec<PluginCommand>         // 注入命令面板/菜单/右键
list_plugin_views(placement?)                   → Vec<PluginView>            // 注入侧边栏/标签页
invoke_plugin_command(id, commandId, params, ctx)→ CommandResult             // 输出: json-ui/markdown/html/open-webview
render_plugin_view(id, viewId, ctx)             → JsonUiTree                 // 声明式视图渲染
plugin_view_action(id, viewId, actionId, args)  → JsonUiTree | patch         // 声明式 button/input 回调
// Webview（资源加载与桥由前端 asset 协议 + postMessage；后端仅校验/审计）
register_plugin_webview(id, viewId)             → { assetBase, csp }         // 返回沙箱加载参数
// 权限与日志
update_plugin_permissions(id, granted[])        → ()
get_plugin_audit_log(id, opts?)                 → Vec<AuditEntry>
```

注册：`lib.rs` 的 `pub use interface::commands::plugin::*;` + `invoke_handler![]` 追加。

---

## 七、DDD 落地文件清单

| 层 | 文件（新增） | 职责 |
|----|-------------|------|
| domain/models | `domain/models/plugin.rs` | Manifest / Command / View / Capability / Permission / TrustLevel / JsonUiNode / WebviewConfig |
| domain/repositories | `domain/repositories/plugin_repository.rs` | trait：registry 读写 |
| domain/services | `domain/services/plugin_domain.rs` | manifest 校验、semver、权限策略、CSP 基线校验、顺序/失败策略 |
| application/services | `application/services/plugin_service.rs` | 协调 + `PluginDispatcher` |
| infrastructure | `infrastructure/plugin_runner.rs` | 子进程 + NDJSON/JSON-RPC + 重启 + 审计 |
| infrastructure | `infrastructure/plugin_webview_bridge.rs` | asset 协议 + iframe 消息桥 + CSP 注入 + 权限网关 |
| infrastructure | `infrastructure/plugin_registry_repository.rs` | registry.json 实现 |
| infrastructure | `infrastructure/plugin_installer.rs` | 解压/校验/落盘（复用 `safe_file_ops`） |
| interface/commands | `interface/commands/plugin.rs` | Tauri 命令薄封装 |
| 接线点（改动·可选） | `http_service.rs` / `import_service.rs` / `ai_tool_service.rs` | 插入 `PluginDispatcher` 调度（仅 P4 起需要） |

依赖方向严格自上而下；`domain` 不依赖基础设施。

---

## 八、典型插件创意（验证通用性 + 可选能力覆盖）

| 插件 | 中心能力 | 用到的可选组 | 价值 |
|------|----------|-------------|------|
| **JSON Explorer** | 命令 + Webview 视图 | — | 超大 JSON 交互浏览（与 API 测试无关） |
| **Local Dashboard** | 侧边栏/标签页视图 + 命令 | network.outbound | 内部服务实时看板 |
| **Snippet / Notes** | 侧边栏视图（Webview） | fs.write | 代码片段/笔记管理器 |
| **QR / Barcode** | 命令（声明式 json-ui） | — | 选中文本生成码图 |
| **Internal Signer** | 命令 | hooks(beforeRequest, abort) | 网关签名头注入 |
| **Protobuf/Thrift Viewer** | 命令 | responseRenderers + hooks | 私有协议解码 |
| **HAR / JMeter Importer** | 命令 | importers | 补齐导入格式 |
| **JSONPath Assertion** | 命令 | assertions | 增强断言 |
| **Secrets from Vault** | 命令 | secret.read + hooks | 拉取 token |
| **AI: 内部知识库** | 命令 | tools(MCP 风格) | AI 助手调用插件查内部系统 |

> 前三类（JSON Explorer / Dashboard / Snippet）与 API 测试**无直接关系**，体现「插件 = 通用应用扩展平台」。

---

> 本设计稿与 `docs/plugin-system-design.md` 配套评审；评审通过后按设计文档 §15 路线图（P0→P7）分阶段实施：P1 先落地「命令 + 声明式 UI」，P2 落地「嵌入式 Webview」，P3 落地「常驻视图」。

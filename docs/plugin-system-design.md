# FM Tester 插件系统设计方案（通用 CLI 插件平台）

> 版本: 2.0 (草案) | 日期: 2026-06-29 | 目标版本: 0.9.0+ | 状态: 评审中
>
> 关键演进（相对 v1）：插件**不再围绕 API 测试既有功能**，而是一个**通用应用扩展平台**——任意 CLI 程序可为应用增加新命令、新视图、新工具，与 API 测试无直接关系亦可。中心原语是**命令(Commands)** 与 **视图(Views)**；既有「请求钩子/导入器/渲染器/断言」**降级为可选能力组**，插件按需 opt-in。
>
> UI 自主度采用**最强模型**：声明式输出（JSON-UI / Markdown / HTML）+ **嵌入式 Webview**（插件自带 HTML 页，沙箱加载，经协议双向通信），类 VS Code / Obsidian。
>
> 关联文档：`docs/plugin-system-mockup.md`（UI 原型 / manifest / 协议时序 / 架构图）

---

## 一、目标与非目标

### 1.1 背景

FM Tester 已有强内置扩展（rquickjs 脚本、导入器、AI 助手、压测断言等），但所有扩展点**编译期固定**，第三方无法不改源码地增强。

本方案不再把插件绑定在「API 测试」功能上，而是建立一个**通用扩展平台**：一个插件可以是一个完全独立的工具（如 JSON 树浏览器、本地看板、内部运维小工具、笔记/代码片段管理器），它只是「住在 FM Tester 这个壳里」，借用宿主的窗口、标签页、侧边栏、变量、凭据、日志、AI 等基础设施。

i18n 中已预留 `menu.plugin = "插件"` 占位，本方案将其落地。

### 1.2 目标（Goals）

| # | 目标 | 说明 |
|---|------|------|
| G1 | **语言无关的 CLI 插件** | 插件是一个**可执行 CLI 程序**（任意语言），应用经子进程 + stdio 协议加载。 |
| G2 | **通用扩展，不绑定现有功能** | 中心原语是**命令**与**视图**；插件可提供与 API 测试无关的全新能力。 |
| G3 | **三种 UI 呈现模型** | ① 声明式 JSON-UI；② 富文本（Markdown/HTML）；③ **嵌入式 Webview**（插件自带 HTML 页，最强）。 |
| G4 | **声明式 manifest** | `plugin.json` 声明身份/版本/入口/视图/命令/能力组/权限。 |
| G5 | **沙箱隔离 + 权限模型** | CLI 进程 + Webview 均隔离；安装时显式授权；行为可审计。 |
| G6 | **零侵入核心** | 插件系统作为独立域（`plugin`）加入；核心流程仅在「调度点」插入分发。 |
| G7 | **可选复用既有扩展点** | 请求钩子/导入器/渲染器/断言/AI 工具作为**可选能力组**，插件按需声明。 |

### 1.3 非目标（首版不做）

- **OS 级硬沙箱**（seccomp/App Sandbox/容器）：以权限声明 + 授权 + 审计 + Webview 沙箱做软约束，硬沙箱列入路线图。
- **插件商店运营**：首版本地安装 + 简易索引；线上市场后续。
- **移动端**：仅桌面（Tauri desktop）。
- **热替换运行中的请求**：启用/禁用对新请求生效。

### 1.4 与脚本引擎的关系

| 维度 | rquickjs 脚本 | CLI 插件 |
|------|---------------|----------|
| 运行位置 | 应用进程内，QuickJS 沙箱 | 独立 OS 进程 + 可选沙箱 Webview |
| 语言 | 仅 JavaScript | 任意语言（前端可用任意 Web 技术） |
| 能力 | 受限 `fm.*` API | 声明任意权限 + 自带 UI |
| 定位 | 请求内轻逻辑 | 跨请求 / 强能力 / 可分发 / 可独立成「应用内小工具」 |

二者并存：脚本用于请求内轻逻辑，插件用于通用扩展。

---

## 二、核心概念

### 2.1 插件（Plugin）= 一个目录

```
plugins/
└── com.example.json-explorer/
    ├── plugin.json            ← 声明文件（必需）
    ├── backend(.exe)          ← CLI 入口（任意语言/二进制/脚本）
    ├── webview/               ← 可选：自定义 Webview 前端资源
    │   ├── index.html
    │   ├── app.js
    │   └── assets/
    ├── README.md
    └── .fm/                   ← 运行期私有数据（fs.write 默认作用域）
```

### 2.2 中心原语：命令（Command）与视图（View）

- **命令（Command）**：一次性动作，出现在**命令面板 / 菜单 / 右键菜单 / 侧边栏入口**。可带输入 schema、有输出（声明式/富文本/或打开一个 Webview）。无状态、即用即走（或经 `cli-once` 退化模式直接 spawn）。
- **视图（View）**：持久 UI 面，挂载到**侧边栏**或**主区标签页**。由插件渲染（声明式 JSON-UI 流，或嵌入式 Webview），可与服务端 CLI 双向交互、保持状态。

> 这是 VS Code「Command + Webview View」与 Raycast「Command」的结合：命令负责「做一件事」，视图负责「一个常驻的工作面」。

### 2.3 能力组（Capability Group）

插件按需声明它**提供**或**接入**的能力。能力组分为两类：

**A. 通用能力（中心）**

| 能力 | 说明 |
|------|------|
| `commands[]` | 一次性命令（含输入 schema、输出模式、出现位置） |
| `views[]` | 常驻视图（侧边栏 / 主区标签页，渲染模式） |
| `webview` | 自带 HTML 前端（嵌入式 Webview 入口 + CSP） |

**B. 可选：接入宿主既有功能（按需 opt-in）**

| 能力 | 说明 |
|------|------|
| `hooks[]` | 请求生命周期钩子（beforeRequest / afterResponse） |
| `importers[]` / `exporters[]` | 导入/导出器 |
| `responseRenderers[]` | 响应体渲染器（按 content-type） |
| `assertions[]` | 压测/接口断言器 |
| `tools[]` | AI 工具（MCP 风格，供 AI 助手调用） |

> 未声明的能力即使插件实现也不会被调用（manifest 与运行期 `initialize` 取交集）。

### 2.4 清单（Manifest）

完整字段见 `docs/plugin-system-mockup.md` §2。核心新增：`commands` / `views` / `webview`。

---

## 三、三种 UI 呈现模型

插件命令/视图的渲染分三档，**由插件按场景选择**：

### 3.1 声明式 JSON-UI（轻、安全）

插件返回一棵**组件树**（JSON），应用用**原生组件**渲染：

```jsonc
{ "type": "column", "children": [
    { "type": "heading", "level": 2, "text": "导入结果" },
    { "type": "stat", "label": "成功", "value": "12", "tone": "success" },
    { "type": "table", "columns": ["名称","状态"], "rows": [["a","ok"]] },
    { "type": "chart", "option": { /* echarts option */ } },          // 复用项目 echarts
    { "type": "button", "label": "重试", "action": "retry" }          // 点击 → action/invoke 回调插件
  ] }
```

- 组件白名单：`column/row/heading/text/stat/table/list/button/input/select/chart/code/markdown/tabs/spacer/...`。
- 交互（button/input）经 `action/invoke` 回到插件，插件返回新树 → 局部或整体刷新。
- 无任意 HTML 执行，**最安全**；适合工具/报告/简单表单。

### 3.2 富文本（Markdown / 受控 HTML）

插件返回 Markdown 或 HTML，应用用既有 `marked` + `dompurify` 渲染（项目已集成）。适合文档、报告、富结果展示。

### 3.3 嵌入式 Webview（强、自定义 UI）

插件自带 `webview/index.html`（可用任意 Web 技术：Vue/React/Svelte/纯 JS/echarts/三方库），加载进**沙箱 iframe**，经 FMPP 协议与其 CLI 后端双向通信。

```
插件 Webview (iframe, sandbox, 独立源)
        │  postMessage
        ▼
主窗口 Bridge (翻译为 FMPP 帧)
        │  stdio NDJSON / JSON-RPC
        ▼
插件 CLI 进程 (插件自己的"后端")
```

- 适合：复杂表单、可视化看板、交互式工具、图表丰富的面板，甚至与 API 测试完全无关的小应用。
- **沙箱化**：iframe 无 `allow-same-origin`（独立 opaque 源）、CSP 锁定、**无直接 Tauri API**；一切宿主能力（变量/凭据/网络/剪贴板/Toast）必须经协议、受权限校验（§7）。
- Webview 的「后端」就是插件 CLI 进程；CLI 通过 `fm/*` 反向能力访问宿主。

> 三种模式**共用同一套 FMPP 协议 + 安装/权限/审计**，区别仅在渲染层。简单插件用声明式，复杂插件上 Webview。

---

## 四、扩展点（通用 + 可选）

| 扩展点 | 能力组 | 触发 | 输入 → 输出 | 典型用途（含非 API 测试） |
|--------|--------|------|-------------|--------------------------|
| **命令** | `commands[]` | 命令面板/菜单/右键/侧边栏入口 | 上下文 → 输出(任意模式) | 任意一次性工具 |
| **侧边栏视图** | `views[]`(`sidebar`) | 常驻侧边栏 | 持久面 | 笔记/片段管理器、快捷面板 |
| **主区标签页视图** | `views[]`(`tab`) | 打开为标签页 | 持久面 | JSON 浏览器、本地看板、运维工具 |
| **请求前置钩子** *(可选)* | `hooks: beforeRequest` | 发送前 | request → 可改写 | 签名/鉴权头注入 |
| **请求后置钩子** *(可选)* | `hooks: afterResponse` | 响应后 | response → 处理 | 解密响应、提取变量 |
| **导入/导出** *(可选)* | `importers/exporters` | 导入/导出对话框 | 文本 ↔ 集合格式 | HAR/JMeter/.proto |
| **响应渲染器** *(可选)* | `responseRenderers` | 按 content-type | bytes → 渲染 | Protobuf/Thrift |
| **断言器** *(可选)* | `assertions` | 压测/测试 | 值+配置 → pass/fail | JSONPath/JMESPath |
| **AI 工具** *(可选)* | `tools` | AI 助手调用 | 入参 → 结果 | 让 AI 调用插件能力 |

---

## 五、嵌入式 Webview 架构

### 5.1 承载方式（Tauri 2）

- **面板内嵌入**（首选）：主窗口内一个 **沙箱 `<iframe>`**，`src` 指向插件 `webview/index.html`，通过**自定义 asset 协议**（Tauri `register_uri_scheme_protocol`，如 `fm-plugin://<id>/...`）加载插件资源。
- **完全分离窗口**（可选）：`WebviewWindow` 独立窗口，适合「插件即独立小窗」场景。

### 5.2 沙箱与隔离

- iframe `sandbox` **不含 `allow-same-origin`** → 插件页为 opaque 源，无法访问主窗口 DOM / Tauri 原生对象。
- CSP 由**应用钳制**（插件可声明更严，但不能放宽）：`default-src 'none'; script-src 'self'(插件域); connect-src 'none';` —— **禁止插件页直连网络**，数据一律走协议。
- 主窗口 ↔ iframe 仅靠 `window.parent.postMessage`，消息带 `target:'fm'` 前缀 + 来源校验。

### 5.3 消息流（Webview ↔ 宿主 ↔ CLI 后端）

```
① 用户在插件页点按钮
   iframe ──postMessage({type:'invoke', method:'analyze', params})──▶ 主窗口 Bridge
② Bridge 翻译为 FMPP，转发给该插件 CLI 进程
   Bridge ──webview/message(req)──▶ 插件 CLI
③ CLI 处理（可反向调用宿主能力 fm/*，受权限校验）
④ CLI 返回结果
   Bridge ◀──result──── 插件 CLI
⑤ Bridge 回传 iframe
   iframe ◀──postMessage({type:'result', ...})── Bridge

宿主能力（变量/凭据/网络/Toast/剪贴板）：
   iframe ──postMessage({type:'fm', method:'variable.get'})──▶ Bridge ──权限校验──▶ 执行或转发 ──▶ 回传
```

### 5.4 生命周期

- 视图首次显示 → 应用 `webview/loadUrl` 指示加载资源 → iframe 加载完成 → 应用发 `webview/ready` → 插件可主动 `view/render` 推送初始 JSON-UI 或初始化 Webview。
- 视图关闭/插件禁用 → iframe 卸载 + CLI 进程按空闲/禁用策略回收。

---

## 六、通信协议 FMPP（FM Plugin Protocol）

### 6.1 传输与信封

- **传输**：子进程 stdin/stdout，**NDJSON**（每行一 JSON 对象，UTF-8）。
- **信封**：**JSON-RPC 2.0**（请求带 `id`，通知无 `id`）。
- **stderr**：插件诊断日志（采集后写日志面板，不参与协议）。
- **生命周期**：长驻进程 + `idleTimeoutMs` 空闲回收 + `restartOnError` 重启。

### 6.2 握手

```
应用 ──initialize──▶ 插件        { fmTesterVersion, protocolVersion, client, locale }
应用 ◀──result────── 插件        { protocolVersion, serverInfo, capabilities(与manifest取交集) }
应用 ──initialized──▶ 插件
... 业务 ...
应用 ──shutdown─────▶ 插件
```

### 6.3 应用 → 插件

| method | 说明 |
|--------|------|
| `initialize`/`initialized`/`shutdown` | 生命周期 |
| `command/invoke` | 运行命令，params: `commandId` + 上下文；result: 输出（json-ui / markdown / html / open-webview） |
| `view/render` | 请求视图的声明式渲染树；result: JSON-UI 树 |
| `view/action` | 声明式 UI 上的 button/input 回调；result: 新树或 patch |
| `webview/ready` | 通知：插件 Webview iframe 已就绪 |
| `webview/message` | 透传 Webview → CLI 的消息 |
| *(可选)* `hook/beforeRequest` / `hook/afterResponse` | 请求钩子 |
| *(可选)* `import/parse` / `export/serialize` | 导入/导出 |
| *(可选)* `render/response` | 响应渲染 |
| *(可选)* `assertion/run` | 断言 |
| *(可选)* `tool/invoke` | AI 工具 |

### 6.4 插件 → 应用（反向，受权限）

| method | 说明 | 权限 |
|--------|------|------|
| `fm/log` | 写控制台（带插件标签） | `fm.console.log` |
| `fm/toast` | 弹 Toast | `fm.toast` |
| `fm/variable.get/set/remove` | 读/写变量 | `fm.variable.read` / `.write` |
| `fm/request.send` | 发请求（走全局代理/超时） | `network.outbound` |
| `fm/clipboard.write` | 写剪贴板 | `fm.clipboard.write` |
| `fm/secret.get` | 取加密凭据（细粒度 key） | `fm.secret.read` |
| `fm/context.get` | 取当前上下文（工作区/选中节点） | `fm.context.read` |
| `webview/postMessage` | 向插件自己的 Webview iframe 推消息 | （自有视图，无需额外权限） |
| `view/update` | 声明式视图局部刷新 | （自有视图） |

### 6.5 退化模式（一次性 CLI）

轻量命令可声明 `entry.transport: "cli-once"`：每次 `backend <method> --input <stdin-json>`，stdout 读单行 JSON result 后退出。简单无状态，便于分发单文件脚本。

### 6.6 超时与取消

- 各能力默认超时（command 30s、view/render 5s、hook 5s、tool 30s；manifest 可覆盖）。
- 超时 → 发 `$/cancel`（带原 `id`）+ 按 `restartOnError` 决定是否杀进程重启。
- 请求生命周期钩子不应阻塞主请求：超时即放弃，按失败策略继续（§8.5）。

---

## 七、调度点与集成（零侵入）

| 调度点 | 既有位置 | 改动 |
|--------|----------|------|
| 命令/视图 | 前端命令面板、`MenuBar`、`Sidebar`、`TabsBar`、右键菜单 | 汇总 `list_plugin_commands/views` 注入入口；新增 `PluginViewHost` 承载视图 |
| 请求前置/后置 *(可选)* | `http_service.rs` | `PluginDispatcher::before_request/after_response` |
| 导入 *(可选)* | `import_service.rs` | 命中扩展名则走协议 |
| 响应渲染 *(可选)* | `ResponsePanel` | content-type 命中则 `invoke` |
| 断言 *(可选)* | 压测/测试断言处 | 命中则走协议 |
| AI 工具 *(可选)* | `ai_tool_service.rs` | 合并插件 `tools[]` |

`PluginDispatcher`（应用层）：按 manifest 能力过滤 → 选插件 → 经 `PluginRunner`（基础设施层）收发协议 → 汇总结果。

---

## 八、生命周期与策略

### 8.1 安装

1. 来源：本地目录 / `.zip` / `.tar.gz` / URL（可选 checksum）。
2. 解压 → 校验 `plugin.json`：字段完整、`id` 格式、semver、`engines.fmTester` 兼容、权限白名单、入口存在 + 可执行位、Webview CSP 合规（不得放宽基线）。
3. **权限审查**（§11.4 mockup）：逐项授权。
4. 落盘 `<exe_dir>/data/plugins/<id>/<version>/`，登记 `registry.json`。
5. 默认启用（可配为安装后默认禁用）。

### 8.2 启停/卸载/升级

- 启停：改 `enabled`；禁用时优雅 `shutdown` 进程并卸载 Webview。
- 卸载：`shutdown` → 删目录 → 移除 registry（可选保留配置）。
- 升级：新版本并存、切换指针；semver 拦截不兼容；新增权限触发差量重新授权。

### 8.3 进程管理（`PluginRunner`）

- 每启用插件最多 1 长驻进程；按需懒启动。
- 空闲回收；崩溃计数 + 指数退避重启；连续失败阈值 → 自动禁用并通知。

### 8.4 执行顺序（多插件 + 脚本，仅可选钩子链）

```
前置链: 工作区脚本 → 环境脚本 → 父集合 → 子集合 → 接口脚本
        → 插件 beforeRequest（按 priority 升序，默认 id 字母序）
           [网络发送]
后置链: 插件 afterResponse（反向） → 接口脚本 → ... → 工作区脚本（反向）
```

只读能力（render/assert/command）可并行；可改写请求/响应的钩子默认串行。

### 8.5 失败策略（钩子）

- `continue`（默认）：出错/超时记日志，继续。
- `abort`：失败终止请求并在控制台报错（强校验插件，如签名必选）。
- `warn`：仅告警。

---

## 九、安全与权限

### 9.1 权限白名单（首版）

| 权限 | 含义 |
|------|------|
| `fm.console.log` / `fm.toast` | 控制台 / Toast |
| `fm.variable.read` / `.write` | 变量读写 |
| `fm.context.read` | 读当前上下文（工作区/选中节点） |
| `fm.clipboard.write` | 写剪贴板 |
| `network.outbound` | 经应用网关发请求 |
| `fm.secret.read` | 读加密凭据（细粒度 key） |
| `fs.read` / `fs.write` | 读写**插件沙箱目录**（默认自身；越界路径单独授权） |
| `webview.embed` | 启用嵌入式 Webview（自带前端） |

### 9.2 Webview 安全基线（强制）

- iframe 沙箱无 `allow-same-origin`；CSP `connect-src 'none'`（禁直连网络）；`script-src` 限插件域。
- 无直接 Tauri 原生 API；所有宿主访问走协议 + 权限校验。
- 插件页第三方资源需随包分发（离线），不得外链 CDN（除非显式授权 `webview.remote`，并降信任级别）。

### 9.3 完整性与信任

- `checksum`（sha256）可选；URL 安装强制校验。
- 可选**签名**（minisign/cosign），应用内置/可配置公钥校验。

| 信任级别 | 判定 | 表现 |
|----------|------|------|
| Verified | 签名通过 + checksum 一致 | 正常安装 |
| Trusted | checksum 一致但未签名（本地/已知来源） | 提示后安装 |
| Unverified | 来源未知/校验缺失 | 醒目警告，默认禁用运行；Webview 默认关闭 |

### 9.4 授权与降权

- 权限安装时授予，存 `registry.json`；升级新增权限触发差量授权。
- 设置面板可随时回收单项或整体禁用；敏感操作（secret/fs 越界）支持「每次询问」。

### 9.5 审计

- 插件所有 `fm/*` 反向调用 + Webview 宿主访问 → 写**插件审计日志**（控制台带标签 + 独立文件）。

> **诚实声明**：CLI 子进程在 OS 层不受权限硬约束（恶意插件理论上可直访文件/网络）。本模型实现「声明可见 + 安装授权 + 应用能力网关化 + Webview 沙箱 + 行为可观测」；硬隔离见路线图 P5。

---

## 十、存储与发现

```
<exe_dir>/data/plugins/
├── registry.json                ← 索引：启用态/权限/版本指针/优先级/信任级别
└── <plugin_id>/<version>/        ← 解压目录（plugin.json + backend + webview/ + 资源）
    └── .fm/                      ← 插件私有数据 + audit.log
```

启动时扫描 `registry.json` + 校验一致性，重建「已启用插件 → 能力」内存索引，供调度点 O(1) 查询。

`registry.json`：

```jsonc
{
  "version": 1,
  "plugins": [
    { "id": "com.example.json-explorer", "version": "1.0.0", "enabled": true,
      "priority": 100, "trust": "trusted",
      "grantedPermissions": ["fm.console.log", "fm.context.read", "webview.embed"],
      "installedAt": "2026-06-29T10:00:00Z", "source": "local" }
  ]
}
```

---

## 十一、后端架构（DDD 分层）

```
interface/commands/plugin.rs            ← #[tauri::command] 薄封装（注册进 lib.rs invoke_handler!）
        │
application/services/plugin_service.rs  ← 协调：安装/启停/卸载/升级/命令/视图 + PluginDispatcher
        │
domain/services/plugin_domain.rs        ← 纯逻辑：manifest 校验、semver、权限策略、CSP 基线校验、顺序/失败策略
domain/repositories/plugin_repository.rs← trait：registry 读写
domain/models/plugin.rs                 ← Manifest / Command / View / Capability / Permission / TrustLevel / JsonUiNode
        │
infrastructure/plugin_runner.rs         ← 子进程 + NDJSON/JSON-RPC + 超时/重启 + 审计
infrastructure/plugin_webview_bridge.rs ← Webview asset 协议 + iframe 消息桥 + CSP 注入
infrastructure/plugin_registry_repository.rs ← registry.json 实现
infrastructure/plugin_installer.rs      ← 解压/校验/落盘（复用 safe_file_ops）
```

**注册**：`lib.rs` 的 `pub use interface::commands::plugin::*;` + `invoke_handler![]` 追加命令。
**可选接线**：`http_service.rs` / `import_service.rs` / `ai_tool_service.rs` 各加 `PluginDispatcher` 调用。

---

## 十二、前端架构

- **设置中心新增「插件」分类**（i18n `menu.plugin` 已存在）：`categories` 增 `plugins`；新增 `components/PluginManagerPanel/`。
- **新增**：
  - `composables/usePlugins.js`：列表/启停/安装/权限/日志。
  - `components/PluginViewHost/`：按渲染模式承载视图（声明式 JSON-UI 渲染器 / Markdown / 沙箱 iframe）。
  - `components/PluginSidebarEntry/`：插件注入的侧边栏入口。
  - `components/JsonUiRenderer/`：JSON-UI 组件树 → 原生组件（复用 echarts 等）。
- **既有组件注入**：命令面板 / `MenuBar` / `Sidebar` / `TabsBar` / 右键菜单 / `ConsolePanel` 聚合插件命令与视图。
- **i18n**：`zh-CN.json` 与 `en.json` 同步新增 `plugin.*`（双语必改）。

组件树 / Tauri 命令 / UI 原型见 `docs/plugin-system-mockup.md`。

---

## 十三、i18n 键规划

```
plugin.title/subtitle/install/installFromFile/installFromUrl/enable/disable/uninstall
plugin.permissions/grant/trust.verified|trusted|unverified
plugin.noPlugins/running/disabled/error/auditLog/newPermissions
plugin.command/view/sidebar/tab/webview
plugin.permission.<key>   各权限中英文名
```

---

## 十四、版本兼容

- **宿主→插件**：`engines.fmTester` semver 校验。
- **协议版本**：`initialize` 交换 `protocolVersion`（如 `2.0`），不兼容拒绝加载。
- **manifest schema**：`manifestVersion` 字段供演进。
- **能力演进**：新增能力向后兼容；废弃先 `deprecated` 一版再移除。

---

## 十五、分阶段路线图

| 阶段 | 范围 | 交付 |
|------|------|------|
| **P0 骨架** | 协议/manifest/安装/启停 + 设置面板 + 审计 | 能装、能启停、能看日志 |
| **P1 命令 + 声明式 UI** | `commands`（含 `cli-once`）+ JSON-UI/Markdown/HTML 渲染 | 命令面板/菜单/右键触发；返回声明式结果 |
| **P2 嵌入式 Webview** | `webview` + asset 协议 + 沙箱 iframe + 消息桥 | 插件自带 HTML 页，沙箱内交互 |
| **P3 常驻视图** | `views`（侧边栏 / 主区标签页）+ `PluginViewHost` | 插件常驻工作面 |
| **P4 可选能力组** | hooks / importers / exporters / renderers / assertions | 复用 API 测试扩展点 |
| **P5 AI 工具** | `tools`（MCP 风格）接入 `ai_tool_service` | 插件能力供 AI 调用 |
| **P6 安全增强** | 签名校验、硬沙箱调研 | 提升信任与隔离 |
| **P7 市场（可选）** | 在线索引 + 一键安装 | 分发 |

---

## 十六、风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| 恶意 CLI / Webview 损害宿主 | 安全 | 权限网关 + Webview 沙箱 + 签名 + 审计 + 崩溃自禁用；硬沙箱列入 P6 |
| Webview 越权（同源/外链） | 安全 | iframe 无 same-origin + CSP connect-src 'none' + 无原生 API + 资源离线随包 |
| 子进程阻塞 | 性能 | 超时 + 失败策略 + 长驻复用 + 只读并行 |
| 跨平台可执行分发 | 体验 | 脚本型入口（node/python）+ cli-once + 按平台分发 |
| 协议演进破坏兼容 | 生态 | semver + protocolVersion + manifestVersion |
| Webview 与主窗口样式割裂 | 体验 | 提供主题/CSS 变量桥，供插件跟随宿主外观（可选） |

---

## 十七、验收标准（P0–P3 摘要）

- [ ] `plugin.json` 安装校验：非法 id/版本/权限/入口/CSP → 明确报错。
- [ ] 长驻进程：握手→调用→空闲回收→崩溃重启→连续失败自禁用。
- [ ] 设置面板：列表/启停/卸载/权限回收/审计日志。
- [ ] 命令在命令面板/菜单/右键正确出现；返回 JSON-UI/Markdown 正确渲染，button 回调生效。
- [ ] 嵌入式 Webview：沙箱 iframe 加载插件页、CSP 锁定、postMessage 双向通信、宿主能力经权限校验。
- [ ] `views` 可挂载侧边栏 / 主区标签页，常驻且可交互。
- [ ] zh-CN / en 双语完整；`npm run lint` + `cargo clippy -D warnings` + `cargo test` 通过。

---

> 本方案为草案，评审通过后按 §15 路线图（P0→P7）分阶段实现。UI 原型、manifest 完整示例、协议时序与架构图见 **`docs/plugin-system-mockup.md`**。

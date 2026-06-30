// ============================================================================
// 版本号一致性校验脚本
//
// 背景：项目版本号分散在 5 个文件（见 AGENTS.md），手动同步易遗漏，导致发布
// 版本错位。本脚本统一读取并比对，任一不一致即以非零码退出，供 CI 与本地
// `npm version` / 发版流程兜底。
//
// 用法：node scripts/check-versions.mjs
// 退出码：0 = 一致；1 = 不一致（CI 会标红）。
// ============================================================================

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

// 五处版本来源及各自的提取方式。
// StatusBar / SettingsCenter 的常量均带 "v" 前缀（如 v0.6.3），统一去掉前缀后再比对。
const extractVueVersion = (label) => (text) => {
  const m = text.match(/version\s*=\s*"?v?([0-9][^"]*)"?/);
  if (!m) throw new Error(`${label} 未找到 version 常量`);
  return m[1];
};

const sources = [
  {
    name: "package.json",
    path: join(root, "package.json"),
    extract: (text) => JSON.parse(text).version,
  },
  {
    name: "src-tauri/Cargo.toml",
    path: join(root, "src-tauri", "Cargo.toml"),
    // 仅匹配「行首」的顶层 version 字段，避免误匹配依赖里的 version = "..."
    extract: (text) => {
      const m = text.match(/^\s*version\s*=\s*"([^"]+)"/m);
      if (!m) throw new Error("Cargo.toml 未找到顶层 version 字段");
      return m[1];
    },
  },
  {
    name: "src-tauri/tauri.conf.json",
    path: join(root, "src-tauri", "tauri.conf.json"),
    extract: (text) => JSON.parse(text).version,
  },
  {
    name: "src/components/StatusBar/index.vue",
    path: join(root, "src", "components", "StatusBar", "index.vue"),
    // 形如：const version = "v0.6.3";  —— 去掉可选的 v 前缀后参与比对
    extract: extractVueVersion("StatusBar/index.vue"),
  },
  {
    name: "src/components/SettingsCenter/index.vue",
    path: join(root, "src", "components", "SettingsCenter", "index.vue"),
    // 设置页「关于」面板展示的版本号，格式与 StatusBar 一致
    extract: extractVueVersion("SettingsCenter/index.vue"),
  },
];

const results = sources.map((s) => {
  const text = readFileSync(s.path, "utf8");
  return { name: s.name, version: s.extract(text) };
});

console.log("版本号校验：");
for (const r of results) {
  console.log(`  ${r.name.padEnd(42)} ${r.version}`);
}

const unique = new Set(results.map((r) => r.version));
if (unique.size === 1) {
  console.log(`\n✅ 一致：${results[0].version}`);
  process.exit(0);
}

console.error("\n❌ 版本号不一致，请同步以下文件后重试：");
for (const r of results) {
  console.error(`  ${r.name.padEnd(42)} ${r.version}`);
}
process.exit(1);

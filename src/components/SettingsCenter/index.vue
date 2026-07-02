<script setup>
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsCenter } from "./index.js";
import { THEMES as allThemes } from "../../composables/useTheme.js";
import appIcon from "../../assets/app-icon.png";

const props = defineProps({
  visible: { type: Boolean, default: false },
  initialCategory: { type: String, default: "general" },
});

const emit = defineEmits(["close"]);

const { t } = useI18n();
const c = useSettingsCenter(props, emit);

const categories = computed(() => [
  { key: "general", label: t("settings.general") },
  { key: "ai", label: t("settings.ai") },
  { key: "git", label: t("settings.git") },
  { key: "appearance", label: t("settings.appearance") },
  { key: "about", label: t("settings.about") },
]);

const monoFontOptions = ["IBM Plex Mono", "JetBrains Mono", "Fira Code", "SFMono-Regular", "Consolas"];
const version = "v0.1.1";
</script>

<template>
  <Teleport to="body">
    <div v-if="props.visible" class="sc-overlay" @click.self="c.close()">
      <div class="sc-modal" :class="{ loading: c.loading.value }">
        <!-- 头部 -->
        <header class="sc-header">
          <div class="sh-info">
            <span class="sh-ico">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
            </span>
            <div>
              <h2 class="sh-title">{{ t("settings.title") }}</h2>
              <p class="sh-desc">{{ t("settings.subtitle") }}</p>
            </div>
          </div>
          <div class="sh-actions">
            <button class="sh-x" @click="c.close()">×</button>
          </div>
        </header>

        <!-- 主体 -->
        <div class="sc-body">
          <!-- 左导航 -->
          <aside class="sc-nav">
            <button
              v-for="cat in categories"
              :key="cat.key"
              class="sc-nav-item"
              :class="{ on: c.activeCategory.value === cat.key }"
              @click="c.setCategory(cat.key)"
            >
              {{ cat.label }}
            </button>
          </aside>

          <!-- 右主区 -->
          <main class="sc-main">
            <!-- ============ 通用 ============ -->
            <section v-show="c.activeCategory.value === 'general'" class="sc-page">
              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.request") }}</h3>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.timeout") }}</label>
                  <div class="field s-field">
                    <input
                      v-model.number="c.requestTimeout.value"
                      type="number"
                      min="1"
                    />
                    <span class="s-suffix">{{ t("settings.seconds") }}</span>
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.language") }}</label>
                  <select v-model="c.language.value" class="sel">
                    <option value="zh-CN">简体中文</option>
                    <option value="en">English</option>
                  </select>
                </div>
              </div>

              <div class="s-grp">
                <h3 class="s-grp-title">
                  {{ t("settings.behavior") }}
                </h3>
                <div class="s-row s-row-switch">
                  <div class="s-row-t">
                    <span class="s-row-name">{{ t("settings.restoreWs") }}</span>
                    <span class="s-row-desc">{{ t("settings.restoreWsDesc") }}</span>
                  </div>
                  <button
                    class="switch"
                    :class="{ on: c.restoreWorkspace.value }"
                    @click="c.restoreWorkspace.value = !c.restoreWorkspace.value"
                  ></button>
                </div>
                <div class="s-row s-row-switch">
                  <div class="s-row-t">
                    <span class="s-row-name">
                      {{ t("settings.autoSave") }}
                    </span>
                    <span class="s-row-desc">{{ t("settings.autoSaveDesc") }}</span>
                  </div>
                  <button
                    class="switch"
                    :class="{ on: c.autoSaveOnSend.value }"
                    @click="c.autoSaveOnSend.value = !c.autoSaveOnSend.value"
                  ></button>
                </div>
              </div>
            </section>

            <!-- ============ AI 助手 ============ -->
            <section v-show="c.activeCategory.value === 'ai'" class="sc-page">
              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.aiConfig") }}</h3>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.aiEndpoint") }}</label>
                  <div class="field s-field">
                    <input v-model="c.aiEndpoint.value" placeholder="https://api.openai.com/v1" />
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">API Key</label>
                  <div class="field s-field">
                    <input
                      v-model="c.aiApiKey.value"
                      type="password"
                      :placeholder="c.apiKeyPlaceholder.value"
                    />
                    <span
                      class="key-status"
                      :class="c.hasApiKey.value ? 'set' : 'unset'"
                    >
                      <span class="d"></span>
                      {{ c.hasApiKey.value ? t("settings.keySet") : t("settings.keyUnset") }}
                    </span>
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.model") }}</label>
                  <div class="field s-field model-input-container">
                    <input v-model="c.aiModel.value" :placeholder="t('settings.modelPh')" />
                    <button class="model-dd-btn" @click="c.toggleModelDropdown()">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6" /></svg>
                    </button>
                    <div v-if="c.showModelDropdown.value" class="model-menu">
                      <div v-if="c.loadingModels.value" class="model-opt muted">
                        {{ t("settings.loading") }}
                      </div>
                      <template v-else>
                        <div
                          v-for="m in c.aiModels.value"
                          :key="m"
                          class="model-opt"
                          @click="c.selectModel(m)"
                        >
                          {{ m }}
                        </div>
                        <div v-if="!c.aiModels.value.length" class="model-opt muted">
                          {{ t("settings.noModels") }}
                        </div>
                      </template>
                    </div>
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.timeout") }}</label>
                  <div class="field s-field">
                    <input v-model.number="c.aiTimeout.value" type="number" min="1" />
                    <span class="s-suffix">{{ t("settings.seconds") }}</span>
                  </div>
                </div>
              </div>

              <div class="s-grp">
                <div class="s-grp-head">
                  <h3 class="s-grp-title">{{ t("settings.customHeaders") }}</h3>
                  <button class="ahd-add" @click="c.addHeader()">+ {{ t("settings.addHeader") }}</button>
                </div>
                <div class="ahd-table">
                  <div class="ahd-h">
                    <span class="ahd-cb"></span>
                    <span class="ahd-k">{{ t("settings.headerKey") }}</span>
                    <span class="ahd-v">{{ t("settings.headerValue") }}</span>
                    <span class="ahd-d">{{ t("settings.headerDesc") }}</span>
                    <span class="ahd-x"></span>
                  </div>
                  <div v-for="(h, i) in c.customHeaders.value" :key="i" class="ahd-r">
                    <span class="ahd-cb">
                      <input v-model="h.enabled" type="checkbox" />
                    </span>
                    <input v-model="h.key" class="ahd-k" :placeholder="t('settings.headerKey')" />
                    <input v-model="h.value" class="ahd-v" :placeholder="t('settings.headerValue')" />
                    <input v-model="h.description" class="ahd-d" :placeholder="t('settings.headerDesc')" />
                    <button class="ahd-del" @click="c.removeHeader(i)">×</button>
                  </div>
                  <div v-if="!c.customHeaders.value.length" class="ahd-empty">
                    {{ t("settings.noHeaders") }}
                  </div>
                </div>
              </div>
            </section>

            <!-- ============ Git 备份 ============ -->
            <section v-show="c.activeCategory.value === 'git'" class="sc-page">
              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.gitConfig") }}</h3>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.repoUrl") }}</label>
                  <div class="field s-field">
                    <input v-model="c.repoUrl.value" placeholder="https://github.com/user/repo.git" />
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.branch") }}</label>
                  <div class="field s-field">
                    <input v-model="c.branch.value" placeholder="master" />
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.username") }}</label>
                  <div class="field s-field">
                    <input v-model="c.username.value" />
                  </div>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.password") }}</label>
                  <div class="field s-field">
                    <input
                      v-model="c.password.value"
                      type="password"
                      :placeholder="c.hasPassword.value ? t('gitBackup.passwordKeepPlaceholder') : t('git.passwordPlaceholder')"
                      @blur="c.savePassword()"
                    />
                  </div>
                </div>
                <div class="s-row-actions">
                  <button class="btn sm" :disabled="c.gitTesting.value" @click="c.testGitConnection()">
                    {{ c.gitTesting.value ? t("settings.testing") : t("settings.testConn") }}
                  </button>
                </div>
              </div>

              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.autoBackup") }}</h3>
                <div class="s-row s-row-switch">
                  <div class="s-row-t">
                    <span class="s-row-name">{{ t("settings.autoBackup") }}</span>
                    <span class="s-row-desc">{{
                      c.repoUrl.value.trim()
                        ? t("settings.autoBackupDesc")
                        : t("settings.autoBackupRequireGit")
                    }}</span>
                  </div>
                  <button
                    class="switch"
                    :class="{ on: c.autoBackupEnabled.value }"
                    :disabled="!c.repoUrl.value.trim()"
                    @click="c.autoBackupEnabled.value = !c.autoBackupEnabled.value"
                  ></button>
                </div>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.autoBackupTime") }}</label>
                  <div class="field s-field">
                    <input
                      type="time"
                      v-model="c.autoBackupTime.value"
                      :disabled="!c.autoBackupEnabled.value"
                    />
                  </div>
                </div>
                <div class="s-row s-row-top">
                  <label
                    class="s-lab"
                    :title="t('settings.autoBackupWorkspacesDesc')"
                  >{{ t("settings.autoBackupWorkspaces") }}</label>
                  <div class="ab-ws-dd">
                    <button
                      type="button"
                      class="ab-dd-trigger"
                      :class="{
                        open: c.showBackupWsDropdown.value,
                        disabled: !c.autoBackupEnabled.value,
                      }"
                      :disabled="!c.autoBackupEnabled.value"
                      :title="c.backupWsSummary.value"
                      @click="c.toggleBackupWsDropdown()"
                    >
                      <span
                        class="ab-dd-text"
                        :class="{
                          'has-value': c.autoBackupWorkspaceIds.value.length > 0,
                        }"
                      >{{ c.backupWsSummary.value }}</span>
                      <span class="ab-dd-caret"></span>
                    </button>
                    <div v-if="c.showBackupWsDropdown.value" class="ab-dd-panel">
                      <div class="ab-dd-toolbar">
                        <button
                          type="button"
                          class="ab-dd-op"
                          :disabled="!c.autoBackupWorkspaceIds.value.length"
                          @click="c.clearBackupWorkspaces()"
                        >{{ t("common.clear") }}</button>
                      </div>
                      <div class="ab-dd-list">
                        <div
                          v-for="ws in c.workspaces.value"
                          :key="ws.id"
                          class="ab-dd-item"
                          :class="{
                            selected: c.isBackupWorkspaceSelected(ws.id),
                          }"
                          @click="c.toggleBackupWorkspace(ws.id)"
                        >
                          <span class="ab-dd-name">{{ ws.name }}</span>
                          <svg
                            v-if="c.isBackupWorkspaceSelected(ws.id)"
                            class="ab-dd-check"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="3"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                          ><path d="M20 6 9 17 4 12" /></svg>
                        </div>
                        <div
                          v-if="!c.workspaces.value.length"
                          class="ab-dd-empty"
                        >{{ t("settings.noWorkspaces") }}</div>
                      </div>
                    </div>
                    <div
                      v-if="
                        c.autoBackupEnabled.value &&
                        c.workspaces.value.length &&
                        !c.autoBackupWorkspaceIds.value.length
                      "
                      class="ab-ws-warn"
                    >{{ t("settings.autoBackupNoneSelected") }}</div>
                  </div>
                </div>
              </div>

              <div class="danger-zone">
                <h4 class="dz-title">{{ t("settings.dangerZone") }}</h4>
                <p class="dz-desc">{{ t("settings.clearCredsDesc") }}</p>
                <button class="btn danger sm" @click="c.password.value = ''">
                  {{ t("settings.clearCreds") }}
                </button>
              </div>
            </section>

            <!-- ============ 外观 ============ -->
            <section v-show="c.activeCategory.value === 'appearance'" class="sc-page">
              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.theme") }}</h3>
                <div class="theme-cards">
                  <div
                    v-for="theme in allThemes"
                    :key="theme.id"
                    class="theme-card"
                    :class="{ on: c.themeId.value === theme.id }"
                    @click="c.previewTheme(theme.id)"
                  >
                    <div class="tc-preview" :class="'tc-' + theme.id">
                      <div class="tc-bar"><i></i><i></i><i></i></div>
                      <div class="tc-dot"></div>
                      <div class="tc-line"></div>
                      <div class="tc-line"></div>
                    </div>
                    <div class="tc-foot">
                      <span>{{ theme.label }}</span>
                      <span class="tick">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12l5 5 9-9" /></svg>
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="s-grp">
                <h3 class="s-grp-title">{{ t("settings.typography") }}</h3>
                <div class="s-row">
                  <label class="s-lab">{{ t("settings.monoFont") }}</label>
                  <select v-model="c.monoFont.value" class="sel">
                    <option v-for="f in monoFontOptions" :key="f" :value="f">{{ f }}</option>
                  </select>
                </div>
                <div class="s-row s-row-switch">
                  <div class="s-row-t">
                    <span class="s-row-name">{{ t("settings.animations") }}</span>
                    <span class="s-row-desc">{{ t("settings.animationsDesc") }}</span>
                  </div>
                  <button
                    class="switch"
                    :class="{ on: c.animations.value }"
                    @click="c.animations.value = !c.animations.value"
                  ></button>
                </div>
              </div>
            </section>

            <!-- ============ 关于 ============ -->
            <section v-show="c.activeCategory.value === 'about'" class="sc-page">
              <div class="about-hero">
                <span class="about-mark">
                  <img :src="appIcon" alt="FM Tester" />
                </span>
                <div>
                  <h2 class="about-name">FM <em>Tester</em></h2>
                  <span class="about-ver">{{ version }}</span>
                </div>
              </div>
              <div class="info-rows">
                <div class="info-row">
                  <span class="info-k">{{ t("settings.version") }}</span>
                  <span class="info-v">{{ version }}</span>
                </div>
                <div class="info-row">
                  <span class="info-k">{{ t("settings.author") }}</span>
                  <span class="info-v">forestcnz</span>
                </div>
                <div class="info-row">
                  <span class="info-k">{{ t("settings.techStack") }}</span>
                  <span class="info-v">Tauri · Vue 3 · Rust</span>
                </div>
                <div class="info-row">
                  <span class="info-k">{{ t("settings.repo") }}</span>
                  <a class="info-v link" href="https://github.com/forestcnz/fm-tester" target="_blank">github.com/forestcnz/fm-tester</a>
                </div>
              </div>
              <div class="license-box">
                <div class="license-head">
                  <span>MIT License</span>
                  <span class="license-badge">© 2026</span>
                </div>
                <p class="license-body">{{ t("settings.licenseNote") }}</p>
              </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped src="./style.css"></style>

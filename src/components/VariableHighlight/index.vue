<script setup>
import { computed, ref, onMounted, onUnmounted } from "vue";
import {
  useVariableHighlight,
  useUrlOverlay,
  useVariableAutocomplete,
} from "./index.js";

const props = defineProps({
  // 文本内容
  text: {
    type: String,
    default: "",
  },
  // 模式: 'text' 纯文本高亮, 'input' 输入框覆盖层
  mode: {
    type: String,
    default: "text",
    validator: (val) => ["text", "input"].includes(val),
  },
  // 输入框模式时的值 (v-model)
  modelValue: {
    type: String,
    default: "",
  },
  // 占位符
  placeholder: {
    type: String,
    default: "",
  },
  // 可用变量列表 [{ key, value }]
  variables: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits([
  "update:modelValue",
  "input",
  "scroll",
  "focus",
  "blur",
]);

const { splitByVariables, highlightVariables } = useVariableHighlight();
const { syncScroll } = useUrlOverlay();
const {
  showAutocomplete,
  autocompletePosition,
  filteredVariables,
  handleKeyDown,
  handleInputAutocomplete,
  selectVariable,
  closeAutocomplete,
} = useVariableAutocomplete(props, emit);

// 文本模式：计算高亮片段
const segments = computed(() => {
  return splitByVariables(props.text, props.variables);
});

// 输入框模式
const inputRef = ref(null);
const overlayRef = ref(null);
const wrapperRef = ref(null);

// 输入框值
const inputValue = computed({
  get: () => props.modelValue || props.text,
  set: (val) => {
    emit("update:modelValue", val);
    emit("input", val);
  },
});

// 高亮后的 HTML
const highlightedHtml = computed(() => {
  return highlightVariables(inputValue.value, props.variables);
});

// 处理输入事件
const handleInput = (e) => {
  inputValue.value = e.target.value;
  // 检测 {{ 触发自动补全
  handleInputAutocomplete(e.target.value, e.target.selectionStart, e.target);
};

// 处理滚动事件
const handleScroll = () => {
  syncScroll(inputRef.value, overlayRef.value);
  emit("scroll");
};

// 处理键盘事件
const onKeyDown = (e) => {
  handleKeyDown(e, inputRef.value);
};

// 点击外部关闭下拉菜单
const handleClickOutside = (e) => {
  if (wrapperRef.value && !wrapperRef.value.contains(e.target)) {
    closeAutocomplete();
  }
};

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});

// 暴露方法给父组件
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  getInputRef: () => inputRef.value,
});
</script>

<template>
  <!-- 文本模式：纯文本高亮显示 -->
  <span v-if="mode === 'text'" class="var-highlight-text">
    <template v-for="(seg, index) in segments" :key="index">
      <span
        v-if="seg.isVariable"
        :class="['var-ref', { 'var-undefined': seg.isUndefined }]"
        :title="seg.isUndefined ? `未定义变量: ${seg.varName}` : ''"
        >{{ seg.text }}</span
      >
      <span v-else>{{ seg.text }}</span>
    </template>
    <span v-if="!text" class="var-placeholder">{{ placeholder }}</span>
  </span>

  <!-- 输入框模式：覆盖层方案 -->
  <div
    v-else-if="mode === 'input'"
    class="var-highlight-input-wrapper"
    ref="wrapperRef"
  >
    <!-- 高亮覆盖层 -->
    <div ref="overlayRef" class="var-highlight-overlay">
      <!-- 有内容时显示高亮文本 -->
      <!-- eslint-disable-next-line vue/no-v-html -->
      <span v-if="inputValue" v-html="highlightedHtml"></span>
      <!-- 无内容时显示占位符 -->
      <span v-else class="var-placeholder">{{ placeholder }}</span>
    </div>

    <!-- 透明输入框 -->
    <input
      ref="inputRef"
      type="text"
      :value="inputValue"
      :placeholder="placeholder"
      class="var-highlight-input"
      @input="handleInput"
      @scroll="handleScroll"
      @keydown="onKeyDown"
      @focus="emit('focus', $event)"
      @blur="emit('blur', $event)"
    />

    <!-- 变量自动补全下拉菜单 -->
    <div
      v-if="showAutocomplete && filteredVariables.length > 0"
      class="var-autocomplete"
      :style="{
        top: autocompletePosition.top + 'px',
        left: autocompletePosition.left + 'px',
      }"
    >
      <div
        v-for="item in filteredVariables"
        :key="item.key"
        class="var-autocomplete-item"
        @click="selectVariable(item.key, inputRef)"
      >
        <span class="var-autocomplete-key">{{ item.key }}</span>
        <span class="var-autocomplete-value">{{ item.value }}</span>
      </div>
    </div>
  </div>
</template>

<style src="./style.css"></style>

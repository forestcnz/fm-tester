import { computed, ref } from "vue";

/**
 * 变量高亮 composable
 * 用于检测和高亮显示 {{变量名}} 格式的环境变量引用
 */
export function useVariableHighlight() {
  // 匹配 {{变量名}} 的正则表达式
  const variablePattern = /\{\{([^}]+)\}\}/g;

  /**
   * 检测文本中是否包含变量引用
   * @param {string} text - 要检测的文本
   * @returns {boolean} 是否包含变量引用
   */
  const hasVariables = (text) => {
    if (!text) return false;
    return variablePattern.test(text);
  };

  /**
   * 将文本分割为高亮片段
   * @param {string} text - 要处理的文本
   * @param {Array<{key: string}>} variables - 可用变量列表，用于校验变量是否存在
   * @returns {Array<{text: string, isVariable: boolean, isUndefined: boolean}>} 文本片段数组
   */
  const splitByVariables = (text, variables = []) => {
    if (!text) return [];

    // 创建变量名集合，用于快速查找
    const variableKeys = new Set(variables.map((v) => v.key?.trim()));

    const result = [];
    let lastIndex = 0;
    let match;

    // 重置正则
    variablePattern.lastIndex = 0;

    while ((match = variablePattern.exec(text)) !== null) {
      // 添加变量前的普通文本
      if (match.index > lastIndex) {
        result.push({
          text: text.slice(lastIndex, match.index),
          isVariable: false,
          isUndefined: false,
        });
      }
      // 添加变量文本，检查是否未定义
      const varName = match[1].trim(); // 提取变量名（去掉 {{ 和 }}）
      const isUndefined = !variableKeys.has(varName);
      result.push({
        text: match[0],
        isVariable: true,
        isUndefined,
        varName,
      });
      lastIndex = match.index + match[0].length;
    }

    // 添加剩余的普通文本
    if (lastIndex < text.length) {
      result.push({
        text: text.slice(lastIndex),
        isVariable: false,
        isUndefined: false,
      });
    }

    return result;
  };

  /**
   * 生成高亮后的 HTML 字符串
   * @param {string} text - 要处理的文本
   * @param {Array<{key: string}>} variables - 可用变量列表，用于校验变量是否存在
   * @returns {string} 高亮后的 HTML
   */
  const highlightVariables = (text, variables = []) => {
    if (!text) return "";

    const segments = splitByVariables(text, variables);
    return segments
      .map((seg) => {
        if (seg.isVariable) {
          if (seg.isUndefined) {
            // 未定义变量：红色错误样式
            return `<span class="var-ref var-undefined" title="未定义变量: ${seg.varName}">${escapeHtml(seg.text)}</span>`;
          }
          // 已定义变量：蓝色高亮样式
          return `<span class="var-ref">${escapeHtml(seg.text)}</span>`;
        }
        return escapeHtml(seg.text);
      })
      .join("");
  };

  /**
   * HTML 转义
   * @param {string} text - 要转义的文本
   * @returns {string} 转义后的文本
   */
  const escapeHtml = (text) => {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  };

  /**
   * 检查文本中是否有未定义的变量
   * @param {string} text - 要检查的文本
   * @param {Array<{key: string}>} variables - 可用变量列表
   * @returns {boolean} 是否有未定义变量
   */
  const hasUndefinedVariables = (text, variables = []) => {
    if (!text) return false;
    const segments = splitByVariables(text, variables);
    return segments.some((seg) => seg.isVariable && seg.isUndefined);
  };

  return {
    variablePattern,
    hasVariables,
    splitByVariables,
    highlightVariables,
    hasUndefinedVariables,
  };
}

/**
 * URL 输入框覆盖层 composable
 * 用于同步输入框和覆盖层的滚动、光标位置
 */
export function useUrlOverlay() {
  /**
   * 同步滚动位置
   * @param {HTMLInputElement} inputEl - 输入框元素
   * @param {HTMLElement} overlayEl - 覆盖层元素
   */
  const syncScroll = (inputEl, overlayEl) => {
    if (!inputEl || !overlayEl) return;
    overlayEl.scrollLeft = inputEl.scrollLeft;
  };

  /**
   * 获取输入框的样式对象（用于覆盖层同步）
   * @param {HTMLInputElement} inputEl - 输入框元素
   * @returns {Object} 样式对象
   */
  const getInputStyles = (inputEl) => {
    if (!inputEl) return {};

    const styles = window.getComputedStyle(inputEl);
    return {
      fontSize: styles.fontSize,
      fontFamily: styles.fontFamily,
      lineHeight: styles.lineHeight,
      padding: styles.padding,
      border: styles.border,
      borderRadius: styles.borderRadius,
    };
  };

  return {
    syncScroll,
    getInputStyles,
  };
}

/**
 * 计算文本在输入框中的宽度
 * @param {string} text - 要测量的文本
 * @param {HTMLInputElement} inputEl - 输入框元素
 * @returns {number} 文本宽度（像素）
 */
const measureTextWidth = (text, inputEl) => {
  if (!inputEl) return 0;

  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");

  const computedStyle = window.getComputedStyle(inputEl);
  ctx.font = `${computedStyle.fontSize} ${computedStyle.fontFamily}`;

  return ctx.measureText(text).width;
};

/**
 * 变量自动补全 composable
 * 用于检测 {{ 输入并显示变量选择下拉菜单
 */
export function useVariableAutocomplete(props, emit) {
  // 下拉菜单状态
  const showAutocomplete = ref(false);
  const autocompletePosition = ref({ top: 0, left: 0 });
  const selectedIndex = ref(0);
  const triggerPosition = ref(0); // {{ 的位置
  const filterKeyword = ref(""); // 过滤关键字
  const inputElement = ref(null); // 保存输入框引用

  // 过滤后的变量列表
  const filteredVariables = computed(() => {
    const vars = props.variables || [];
    if (!filterKeyword.value) {
      return vars;
    }
    const keyword = filterKeyword.value.toLowerCase();
    return vars.filter((v) => v.key?.toLowerCase().includes(keyword));
  });

  /**
   * 检测输入是否包含 {{ 并触发自动补全
   * @param {string} value - 输入值
   * @param {number} cursorPos - 光标位置
   * @param {HTMLInputElement} inputEl - 输入框元素
   */
  const handleInputAutocomplete = (value, cursorPos, inputEl) => {
    // 更新输入框引用
    if (inputEl) {
      inputElement.value = inputEl;
    }

    // 查找光标前最近的 {{
    const beforeCursor = value.slice(0, cursorPos);
    const lastBraceIndex = beforeCursor.lastIndexOf("{{");

    if (lastBraceIndex >= 0) {
      // 检查 {{ 后面是否已经有 }}（已完成的变量）
      const afterBrace = value.slice(lastBraceIndex);
      if (
        afterBrace.includes("}}") &&
        afterBrace.indexOf("}}") < cursorPos - lastBraceIndex
      ) {
        // 已完成的变量，不触发
        showAutocomplete.value = false;
        return;
      }

      // 检查 {{ 是否在最近的 }} 之后（避免嵌套）
      const lastCloseBrace = beforeCursor.lastIndexOf("}}");
      if (lastCloseBrace > lastBraceIndex) {
        // {{ 在 }} 之后，不触发
        showAutocomplete.value = false;
        return;
      }

      // 提取 {{ 后面已输入的内容作为过滤关键字
      const keywordStart = lastBraceIndex + 2;
      filterKeyword.value = beforeCursor.slice(keywordStart).trim();

      // 触发自动补全
      triggerPosition.value = lastBraceIndex;
      showAutocomplete.value = true;
      selectedIndex.value = 0;

      // 计算 {{ 前面文本的宽度，确定下拉菜单水平位置
      const textBeforeBrace = value.slice(0, lastBraceIndex);
      const textWidth = measureTextWidth(textBeforeBrace, inputElement.value);

      // 获取输入框的 padding 和 border
      const computedStyle = inputElement.value
        ? window.getComputedStyle(inputElement.value)
        : null;
      const paddingLeft = computedStyle
        ? parseFloat(computedStyle.paddingLeft) || 0
        : 0;
      const borderLeft = computedStyle
        ? parseFloat(computedStyle.borderLeftWidth) || 0
        : 0;

      // 考虑滚动偏移
      const scrollLeft = inputElement.value ? inputElement.value.scrollLeft : 0;

      autocompletePosition.value = {
        top: 32,
        left: Math.max(0, paddingLeft + borderLeft + textWidth - scrollLeft),
      };
    } else {
      showAutocomplete.value = false;
      filterKeyword.value = "";
    }
  };

  /**
   * 选择变量并插入
   * @param {string} varKey - 变量名
   * @param {HTMLInputElement} inputEl - 输入框元素
   */
  const selectVariable = (varKey, inputEl) => {
    if (!inputEl) return;

    const value = inputEl.value;
    const cursorPos = inputEl.selectionStart;

    // 找到 {{ 的位置
    const beforeCursor = value.slice(0, cursorPos);
    const braceIndex = beforeCursor.lastIndexOf("{{");

    if (braceIndex >= 0) {
      // 替换 {{ 为 {{变量名}}
      const newValue =
        value.slice(0, braceIndex) +
        "{{" +
        varKey +
        "}}" +
        value.slice(cursorPos);

      // 更新值
      emit("update:modelValue", newValue);
      emit("input", newValue);

      // 设置光标位置到 }} 后面
      const newCursorPos = braceIndex + varKey.length + 4;
      setTimeout(() => {
        inputEl.setSelectionRange(newCursorPos, newCursorPos);
        inputEl.focus();
      }, 0);
    }

    closeAutocomplete();
  };

  /**
   * 处理键盘事件
   * @param {KeyboardEvent} e - 键盘事件
   * @param {HTMLInputElement} inputEl - 输入框元素
   */
  const handleKeyDown = (e, inputEl) => {
    if (!showAutocomplete.value) return;

    if (e.key === "Escape") {
      closeAutocomplete();
      e.preventDefault();
    } else if (e.key === "ArrowDown") {
      selectedIndex.value = Math.min(
        selectedIndex.value + 1,
        filteredVariables.value.length - 1,
      );
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
      e.preventDefault();
    } else if (e.key === "Enter") {
      if (filteredVariables.value[selectedIndex.value]) {
        selectVariable(
          filteredVariables.value[selectedIndex.value].key,
          inputEl,
        );
      }
      e.preventDefault();
    }
  };

  /**
   * 关闭自动补全
   */
  const closeAutocomplete = () => {
    showAutocomplete.value = false;
    selectedIndex.value = 0;
  };

  return {
    showAutocomplete,
    autocompletePosition,
    filteredVariables,
    selectedIndex,
    handleInputAutocomplete,
    selectVariable,
    handleKeyDown,
    closeAutocomplete,
  };
}

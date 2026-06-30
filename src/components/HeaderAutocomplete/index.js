import { ref, computed } from "vue";

/**
 * 常见 HTTP 请求头定义
 * key: 请求头名称
 * values: 常见值列表
 * description: 描述
 */
export const COMMON_HEADERS = [
  {
    key: "Content-Type",
    values: [
      "application/json",
      "application/x-www-form-urlencoded",
      "multipart/form-data",
      "text/plain",
      "text/html",
      "application/xml",
      "application/octet-stream",
    ],
    description: "请求体的媒体类型",
  },
  {
    key: "Authorization",
    values: ["Bearer {{token}}", "Basic {{credentials}}", "Digest {{digest}}"],
    description: "认证信息",
  },
  {
    key: "Accept",
    values: [
      "application/json",
      "text/html",
      "*/*",
      "application/xml",
      "text/plain",
      "application/octet-stream",
    ],
    description: "可接受的响应类型",
  },
  {
    key: "Accept-Encoding",
    values: ["gzip, deflate, br", "gzip, deflate", "br", "identity"],
    description: "可接受的内容编码",
  },
  {
    key: "Accept-Language",
    values: ["zh-CN,zh;q=0.9,en;q=0.8", "en-US,en;q=0.9", "zh-CN", "en-US"],
    description: "可接受的语言",
  },
  {
    key: "Cache-Control",
    values: [
      "no-cache",
      "no-store",
      "max-age=3600",
      "public",
      "private",
      "must-revalidate",
    ],
    description: "缓存控制",
  },
  {
    key: "Connection",
    values: ["keep-alive", "close"],
    description: "连接控制",
  },
  {
    key: "User-Agent",
    values: [
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
      "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
    ],
    description: "用户代理",
  },
  {
    key: "Host",
    values: [],
    description: "目标主机",
  },
  {
    key: "Origin",
    values: [],
    description: "请求来源",
  },
  {
    key: "Referer",
    values: [],
    description: "引用页",
  },
  {
    key: "Cookie",
    values: [],
    description: "Cookie 数据",
  },
  {
    key: "X-Requested-With",
    values: ["XMLHttpRequest"],
    description: "AJAX 请求标识",
  },
  {
    key: "X-API-Key",
    values: [],
    description: "API 密钥",
  },
  {
    key: "X-Auth-Token",
    values: [],
    description: "认证令牌",
  },
  {
    key: "X-Forwarded-For",
    values: [],
    description: "客户端真实 IP",
  },
  {
    key: "X-Forwarded-Proto",
    values: ["https", "http"],
    description: "原始协议",
  },
  {
    key: "If-Modified-Since",
    values: [],
    description: "条件请求：修改时间",
  },
  {
    key: "If-None-Match",
    values: [],
    description: "条件请求：ETag",
  },
  {
    key: "Content-Length",
    values: [],
    description: "请求体长度",
  },
  {
    key: "Content-Encoding",
    values: ["gzip", "deflate", "br"],
    description: "内容编码",
  },
  {
    key: "Transfer-Encoding",
    values: ["chunked"],
    description: "传输编码",
  },
  {
    key: "TE",
    values: ["trailers", "gzip", "deflate"],
    description: "传输编码偏好",
  },
  {
    key: "Expect",
    values: ["100-continue"],
    description: "期望行为",
  },
  {
    key: "Range",
    values: ["bytes=0-499", "bytes=500-999", "bytes=-500", "bytes=500-"],
    description: "范围请求",
  },
  {
    key: "Access-Control-Request-Method",
    values: ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"],
    description: "CORS 预检：请求方法",
  },
  {
    key: "Access-Control-Request-Headers",
    values: ["Content-Type", "Authorization", "X-Requested-With"],
    description: "CORS 预检：请求头",
  },
];

/**
 * 请求头自动补全 composable
 */
export function useHeaderAutocomplete() {
  const showKeyAutocomplete = ref(false);
  const showValueAutocomplete = ref(false);
  const keyAutocompletePosition = ref({ top: 0, left: 0 });
  const valueAutocompletePosition = ref({ top: 0, left: 0 });
  const selectedIndex = ref(0);
  const currentKeyInput = ref("");
  const currentValueInput = ref("");
  const activeHeaderIndex = ref(-1);

  const filteredHeaders = computed(() => {
    const input = currentKeyInput.value.toLowerCase();
    if (!input) {
      return COMMON_HEADERS.slice(0, 10);
    }
    return COMMON_HEADERS.filter((h) =>
      h.key.toLowerCase().includes(input),
    ).slice(0, 10);
  });

  const filteredValues = computed(() => {
    const header = COMMON_HEADERS.find(
      (h) => h.key.toLowerCase() === currentKeyInput.value.toLowerCase(),
    );
    if (!header || !header.values.length) return [];

    const input = currentValueInput.value.toLowerCase();
    if (!input) return header.values;

    return header.values.filter((v) => v.toLowerCase().includes(input));
  });

  const showKeySuggestions = (input, index, rect) => {
    currentKeyInput.value = input;
    activeHeaderIndex.value = index;
    selectedIndex.value = 0;
    showKeyAutocomplete.value = true;
    showValueAutocomplete.value = false;

    if (rect) {
      keyAutocompletePosition.value = {
        top: rect.bottom + window.scrollY,
        left: rect.left + window.scrollX,
      };
    }
  };

  const showValueSuggestions = (keyInput, valueInput, index, rect) => {
    currentKeyInput.value = keyInput;
    currentValueInput.value = valueInput;
    activeHeaderIndex.value = index;
    selectedIndex.value = 0;
    showValueAutocomplete.value = true;
    showKeyAutocomplete.value = false;

    if (rect) {
      valueAutocompletePosition.value = {
        top: rect.bottom + window.scrollY,
        left: rect.left + window.scrollX,
      };
    }
  };

  const hideAutocomplete = () => {
    showKeyAutocomplete.value = false;
    showValueAutocomplete.value = false;
    selectedIndex.value = 0;
  };

  const handleKeyNavigation = (e) => {
    const list = showKeyAutocomplete.value
      ? filteredHeaders.value
      : showValueAutocomplete.value
        ? filteredValues.value
        : [];

    if (!list.length) return false;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, list.length - 1);
      return true;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
      return true;
    } else if (e.key === "Enter") {
      e.preventDefault();
      return true;
    } else if (e.key === "Escape") {
      hideAutocomplete();
      return true;
    }
    return false;
  };

  const selectKey = (key) => {
    hideAutocomplete();
    return key;
  };

  const selectValue = (value) => {
    hideAutocomplete();
    return value;
  };

  return {
    showKeyAutocomplete,
    showValueAutocomplete,
    keyAutocompletePosition,
    valueAutocompletePosition,
    selectedIndex,
    filteredHeaders,
    filteredValues,
    activeHeaderIndex,
    showKeySuggestions,
    showValueSuggestions,
    hideAutocomplete,
    handleKeyNavigation,
    selectKey,
    selectValue,
  };
}

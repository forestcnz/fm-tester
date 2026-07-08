<script setup>
defineProps({
  name: String,
  size: {
    type: Number,
    default: 16,
  },
});
</script>

<template>
  <span class="icon">
    <svg
      v-if="name === 'collection'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="7" y="3" width="13" height="15" rx="2" stroke-opacity="0.42" />
      <rect x="3" y="6" width="13" height="15" rx="2" />
      <circle cx="6.5" cy="11" r="1" fill="currentColor" stroke="none" />
      <line x1="9.2" y1="11" x2="13" y2="11" stroke-opacity="0.5" />
      <circle cx="6.5" cy="15.5" r="1" fill="currentColor" stroke="none" />
      <line x1="9.2" y1="15.5" x2="13" y2="15.5" stroke-opacity="0.5" />
    </svg>

    <svg
      v-if="name === 'websocket'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="3" y="8.5" width="5.5" height="7" rx="1.5" />
      <rect
        x="15.5"
        y="8.5"
        width="5.5"
        height="7"
        rx="1.5"
        fill="currentColor"
        stroke="none"
      />
      <path d="M9 11 H 14.3" />
      <path d="M13 9.5 L 14.7 11 L 13 12.5" />
      <path d="M15 13 H 9.7" />
      <path d="M11 11.5 L 9.3 13 L 11 14.5" />
    </svg>

    <svg
      v-if="name === 'orchestration'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="5" cy="6" r="2" />
      <path d="M7 6.6 C 10 7.2, 12 9.8, 12 12 C 12 14.2, 10 16.8, 7 17.4" />
      <path d="M17 12 H 12" />
      <circle cx="5" cy="18" r="2" fill="currentColor" stroke="none" />
      <circle cx="19" cy="12" r="2" />
    </svg>

    <svg
      v-if="name === 'environment'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M9 4 C 6.5 4 6.5 7 6.5 9 C 6.5 11 5 12 4 12 C 5 12 6.5 13 6.5 15 C 6.5 17 6.5 20 9 20"
      />
      <path
        d="M15 4 C 17.5 4 17.5 7 17.5 9 C 17.5 11 19 12 20 12 C 19 12 17.5 13 17.5 15 C 17.5 17 17.5 20 15 20"
      />
      <circle cx="10.3" cy="10.3" r="1.1" fill="currentColor" stroke="none" />
      <circle cx="13.7" cy="13.7" r="1.1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'history'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="8" />
      <path d="M12 7.5 V 12 L 15.5 14" />
      <circle cx="12" cy="4.4" r="1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'workspace'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="3" y="3" width="18" height="18" rx="2.5" />
      <line x1="3" y1="9" x2="21" y2="9" />
      <line x1="9" y1="9" x2="9" y2="21" />
      <rect
        x="4.3"
        y="4.3"
        width="3.4"
        height="3.4"
        rx="1"
        fill="currentColor"
        stroke="none"
      />
    </svg>

    <svg
      v-if="name === 'chat'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M4 5 H 18 a2 2 0 0 1 2 2 V 14 a2 2 0 0 1-2 2 H 11 L 6.5 20.2 V 16 H 4 a2 2 0 0 1-2-2 V 7 a2 2 0 0 1 2-2 Z"
      />
      <circle cx="8" cy="10.5" r="1.1" fill="currentColor" stroke="none" />
      <circle cx="12" cy="10.5" r="1.1" fill="currentColor" stroke="none" />
      <circle cx="16" cy="10.5" r="1.1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'folder'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M3 7.5 V 18 a1.5 1.5 0 0 0 1.5 1.5 H 19.5 a1.5 1.5 0 0 0 1.5-1.5 V 9.5 a1.5 1.5 0 0 0-1.5-1.5 H 11 L 9 6 H 4.5 A1.5 1.5 0 0 0 3 7.5 Z"
      />
      <circle cx="17.5" cy="13" r="1.2" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'folder-open'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M3 8 V 18.5 A1.5 1.5 0 0 0 4.5 20 H 17 A1.5 1.5 0 0 0 18.5 19 L 21 9.5 A1 1 0 0 0 20 8 H 6 L 4.5 6 H 3"
      />
      <circle cx="15" cy="14" r="1.2" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'doc'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M5 3 H 14 L 19 8 V 20 A1 1 0 0 1 18 21 H 6 A1 1 0 0 1 5 20 V 4 A1 1 0 0 1 6 3 Z"
      />
      <path d="M14 3 V 7 A1 1 0 0 0 15 8 H 19" />
      <line x1="8" y1="12" x2="15" y2="12" stroke-opacity="0.5" />
      <line x1="8" y1="15" x2="15" y2="15" stroke-opacity="0.5" />
      <line x1="8" y1="18" x2="12" y2="18" stroke-opacity="0.5" />
    </svg>

    <svg
      v-if="name === 'file'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M6 3 H 14 L 19 8 V 20 A1 1 0 0 1 18 21 H 6 A1 1 0 0 1 5 20 V 4 A1 1 0 0 1 6 3 Z"
      />
      <path d="M14 3 V 7 A1 1 0 0 0 15 8 H 19" />
    </svg>

    <svg
      v-if="name === 'api'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <line x1="12" y1="3" x2="12" y2="7.5" stroke-opacity="0.5" />
      <line x1="12" y1="16.5" x2="12" y2="21" stroke-opacity="0.5" />
      <line x1="3" y1="12" x2="7.5" y2="12" stroke-opacity="0.5" />
      <line x1="16.5" y1="12" x2="21" y2="12" stroke-opacity="0.5" />
      <circle cx="12" cy="12" r="1.2" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'ws'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M3 7 V 18 a1.5 1.5 0 0 0 1.5 1.5 H 19.5 a1.5 1.5 0 0 0 1.5-1.5 V 9 a1.5 1.5 0 0 0-1.5-1.5 H 11 L 9 6 H 4.5 A1.5 1.5 0 0 0 3 7.5 Z"
      />
      <rect
        x="9"
        y="11"
        width="2.6"
        height="2.6"
        rx="0.5"
        fill="currentColor"
        stroke="none"
      />
    </svg>

    <svg
      v-if="name === 'send'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M21 4 L 3 11 L 10 14 L 13 21 L 21 4 Z" />
      <path d="M21 4 L 10 14" />
    </svg>

    <svg
      v-if="name === 'save'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M5 3 H 16 L 21 8 V 19 A2 2 0 0 1 19 21 H 5 A2 2 0 0 1 3 19 V 5 A2 2 0 0 1 5 3 Z"
      />
      <rect x="8" y="13" width="8" height="6" rx="0.5" />
      <circle cx="12" cy="16" r="1.1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'delete'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M4 7 H 20" />
      <path d="M9 7 V 5 A1 1 0 0 1 10 4 H 14 A1 1 0 0 1 15 5 V 7" />
      <path d="M6 7 V 19 A2 2 0 0 0 8 21 H 16 A2 2 0 0 0 18 19 V 7" />
      <line x1="10" y1="11" x2="10" y2="17" stroke-opacity="0.5" />
      <line x1="14" y1="11" x2="14" y2="17" stroke-opacity="0.5" />
    </svg>

    <svg
      v-if="name === 'edit'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M14 5 L 19 10 L 8 21 H 3 V 16 L 14 5 Z" />
      <line x1="13" y1="6" x2="18" y2="11" />
      <circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'copy'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="8" y="8" width="13" height="13" rx="2" />
      <path d="M3 16 V 5 A2 2 0 0 1 5 3 H 16" />
    </svg>

    <svg
      v-if="name === 'add'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="9" />
      <line x1="12" y1="7.5" x2="12" y2="16.5" />
      <line x1="7.5" y1="12" x2="16.5" y2="12" />
    </svg>

    <svg
      v-if="name === 'plus'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>

    <svg
      v-if="name === 'stop'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="6" y="6" width="12" height="12" rx="2.5" />
      <rect
        x="9.5"
        y="9.5"
        width="5"
        height="5"
        rx="1"
        fill="currentColor"
        stroke="none"
      />
    </svg>

    <svg
      v-if="name === 'play'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M8 5 L 19 12 L 8 19 Z" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'update'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M12 3 V 14" />
      <path d="M8 10 L 12 14 L 16 10" />
      <path d="M5 17 V 19 A2 2 0 0 0 7 21 H 17 A2 2 0 0 0 19 19 V 17" />
    </svg>

    <svg
      v-if="name === 'export'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="4" y="13" width="16" height="8" rx="2" />
      <path d="M12 13 V 4" />
      <path d="M8 7 L 12 3 L 16 7" />
    </svg>

    <svg
      v-if="name === 'import'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="4" y="13" width="16" height="8" rx="2" />
      <path d="M12 4 V 13" />
      <path d="M8 9 L 12 13 L 16 9" />
    </svg>

    <svg
      v-if="name === 'sync'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M20 13 A8 8 0 0 1 7.5 17" />
      <path d="M4 11 A8 8 0 0 1 16.5 7" />
      <path d="M16.5 4 L 16.5 7 L 19.5 7" />
      <path d="M7.5 20 L 7.5 17 L 4.5 17" />
    </svg>

    <svg
      v-if="name === 'push'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M12 20 V 7" />
      <path d="M7 12 L 12 7 L 17 12" />
      <path d="M5 21 H 19" />
      <circle cx="12" cy="7" r="1.4" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'pull'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M12 4 V 16" />
      <path d="M7 11 L 12 16 L 17 11" />
      <path d="M5 3 H 19" />
      <circle cx="12" cy="16" r="1.4" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'commit'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="3" y1="12" x2="9" y2="12" />
      <line x1="15" y1="12" x2="21" y2="12" />
      <circle cx="12" cy="12" r="3.5" />
      <circle cx="12" cy="12" r="1.3" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'git'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="6" cy="6" r="2" />
      <circle cx="6" cy="18" r="2" />
      <circle cx="18" cy="9" r="2" fill="currentColor" stroke="none" />
      <path d="M6 8 V 16" />
      <path d="M6 12 C 6 10, 8 9, 16 9" />
    </svg>

    <svg
      v-if="name === 'key'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="5" y="10" width="14" height="10" rx="2" />
      <path d="M8 10 V 8 A4 4 0 0 1 16 8 V 10" />
      <circle cx="12" cy="15" r="1.2" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'settings'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3.5" />
      <path
        d="M12 2 V 5 M 12 19 V 22 M 2 12 H 5 M 19 12 H 22 M 5 5 L 7 7 M 17 17 L 19 19 M 19 5 L 17 7 M 7 17 L 5 19"
      />
      <circle cx="12" cy="12" r="1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'performance'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M3 20 H 21" />
      <path d="M3 16 L 8 11 L 12 14 L 16 7 L 21 11" />
      <circle cx="16" cy="7" r="1.3" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'function'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M9 8 L 5 12 L 9 16" />
      <path d="M15 8 L 19 12 L 15 16" />
      <line x1="13" y1="6" x2="11" y2="18" />
    </svg>

    <svg
      v-if="name === 'eye'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M2 12 C 4 7, 8 5, 12 5 C 16 5, 20 7, 22 12 C 20 17, 16 19, 12 19 C 8 19, 4 17, 2 12 Z"
      />
      <circle cx="12" cy="12" r="3" />
      <circle cx="12" cy="12" r="1.1" fill="currentColor" stroke="none" />
    </svg>

    <svg
      v-if="name === 'sparkles'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M11 4 L 12.5 8.5 L 17 10 L 12.5 11.5 L 11 16 L 9.5 11.5 L 5 10 L 9.5 8.5 Z"
      />
      <path
        d="M17.5 14 L 18 15.5 L 19.5 16 L 18 16.5 L 17.5 18 L 17 16.5 L 15.5 16 L 17 15.5 Z"
        fill="currentColor"
        stroke="none"
      />
    </svg>

    <svg
      v-if="name === 'arrow-right'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="4" y1="12" x2="19" y2="12" />
      <polyline points="13 6 19 12 13 18" />
    </svg>

    <svg
      v-if="name === 'arrow-left'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="5" y1="12" x2="20" y2="12" />
      <polyline points="11 6 5 12 11 18" />
    </svg>

    <svg
      v-if="name === 'arrow-down'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="12" y1="4" x2="12" y2="19" />
      <polyline points="6 13 12 19 18 13" />
    </svg>

    <svg
      v-if="name === 'close'"
      :width="size"
      :height="size"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="6" y1="6" x2="18" y2="18" />
      <line x1="18" y1="6" x2="6" y2="18" />
    </svg>
  </span>
</template>

<style scoped src="./style.css"></style>

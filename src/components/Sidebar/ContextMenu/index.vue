<template>
  <div
    v-if="visible"
    class="context-menu"
    :style="{ left: x + 'px', top: y + 'px' }"
    @click.stop
  >
    <template v-for="(item, index) in items" :key="index">
      <!-- 分隔线 -->
      <div v-if="item.divider" class="menu-divider"></div>

      <!-- 菜单项 -->
      <div
        v-else
        class="menu-item"
        :class="{ delete: item.danger }"
        @click="handleClick(item)"
      >
        <span v-if="item.icon" class="menu-icon">
          <Icon :name="item.icon" :size="14" />
        </span>
        <span>{{ item.label }}</span>
      </div>
    </template>
  </div>
</template>

<script setup>
import Icon from "../../Icon/index.vue";

defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  x: {
    type: Number,
    default: 0,
  },
  y: {
    type: Number,
    default: 0,
  },
  items: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(["action", "close"]);

/**
 * 处理菜单项点击
 * @param {Object} item - 菜单项配置
 */
const handleClick = (item) => {
  emit("action", item.action);
  emit("close");
};
</script>

<style src="./style.css" scoped></style>

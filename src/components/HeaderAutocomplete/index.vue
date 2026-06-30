<script setup>

defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  type: {
    type: String,
    default: "key",
    validator: (val) => ["key", "value"].includes(val),
  },
  position: {
    type: Object,
    default: () => ({ top: 0, left: 0 }),
  },
  items: {
    type: Array,
    default: () => [],
  },
  selectedIndex: {
    type: Number,
    default: 0,
  },
  width: {
    type: String,
    default: "280px",
  },
});

const emit = defineEmits(["select", "close"]);

const handleSelect = (item, index) => {
  emit("select", item, index);
};
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible && items.length > 0"
      class="header-autocomplete"
      :style="{
        top: position.top + 'px',
        left: position.left + 'px',
        width: width,
      }"
    >
      <div
        v-for="(item, index) in items"
        :key="type === 'key' ? item.key : item"
        class="autocomplete-item"
        :class="{ selected: index === selectedIndex }"
        @click="handleSelect(item, index)"
        @mousedown.prevent
      >
        <template v-if="type === 'key'">
          <span class="item-key">{{ item.key }}</span>
          <span class="item-desc">{{ item.description }}</span>
        </template>
        <template v-else>
          <span class="item-value">{{ item }}</span>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style src="./style.css"></style>

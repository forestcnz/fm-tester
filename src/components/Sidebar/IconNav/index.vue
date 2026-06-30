<script setup>
import { useIconNavSetup } from "./index.js";
import Icon from "../../Icon/index.vue";

const props = defineProps({
  activeKey: {
    type: String,
    default: "collection",
  },
});

const emit = defineEmits(["navChange"]);

const { navItems, selectNav, getNavName } = useIconNavSetup(props, emit);

// Rail 分组（组间渲染分隔线），顺序按墨砚设计：
// 主功能[集合·WebSocket·编排] | 数据[环境·历史] | 协作[工作区·对话]
const railGroups = [
  { keys: ["collection", "websocket", "orchestration"] },
  { keys: ["environment", "history"] },
  { keys: ["workspace", "chat"] },
];

const itemByKey = (key) => navItems.find((n) => n.key === key);
</script>

<template>
  <nav class="rail">
    <div v-for="(group, gi) in railGroups" :key="gi" class="rail-grp">
      <button
        v-for="key in group.keys"
        :key="key"
        v-show="itemByKey(key)"
        class="rail-item"
        :class="{ on: props.activeKey === key }"
        :aria-label="getNavName(itemByKey(key))"
        @click="selectNav(key)"
      >
        <span class="rail-ico"><Icon :name="itemByKey(key).icon" /></span>
        <span class="tip">{{ getNavName(itemByKey(key)) }}</span>
      </button>
      <span v-if="gi < railGroups.length - 1" class="rail-sep"></span>
    </div>
  </nav>
</template>

<style src="./style.css"></style>

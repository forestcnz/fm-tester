<script setup>
import { useToast } from "../../composables/useToast";

const { toasts, removeToast } = useToast();
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="toast.type"
        @click="removeToast(toast.id)"
      >
        <span class="toast-icon">
          <template v-if="toast.type === 'success'">✓</template>
          <template v-else-if="toast.type === 'error'">✗</template>
          <template v-else-if="toast.type === 'warning'">⚠</template>
          <template v-else>ℹ</template>
        </span>
        <span class="toast-message">{{ toast.message }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  background: #fff;
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  min-width: 200px;
  max-width: 400px;
  animation: slideIn 0.3s ease;
}

.toast-item:hover {
  opacity: 0.9;
}

.toast-icon {
  margin-right: 8px;
  font-size: 14px;
  font-weight: bold;
}

.toast-message {
  font-size: 14px;
  color: #262626;
}

/* 类型样式 */
.toast-item.success {
  background: #f6ffed;
  border: 1px solid #b7eb8f;
}

.toast-item.success .toast-icon {
  color: #52c41a;
}

.toast-item.error {
  background: #fff2f0;
  border: 1px solid #ffccc7;
}

.toast-item.error .toast-icon {
  color: #ff4d4f;
}

.toast-item.warning {
  background: #fffbe6;
  border: 1px solid #ffe58f;
}

.toast-item.warning .toast-icon {
  color: #faad14;
}

.toast-item.info {
  background: #e6f7ff;
  border: 1px solid #91d5ff;
}

.toast-item.info .toast-icon {
  color: #1890ff;
}

/* 动画 */
@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

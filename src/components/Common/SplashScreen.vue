<template>
  <div class="splash-screen">
    <div class="splash-content">
      <img src="/icon.png" class="splash-logo" alt="BeGit Icon" />
      <h1 class="splash-title">BeGit</h1>
      <p class="splash-subtitle">Loading your projects...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--app-bg, var(--bg-main, #1e1e1e));
  z-index: 9999;
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  animation: fade-in 0.5s ease-out;
}

.splash-logo {
  width: 96px;
  height: 96px;
  border-radius: 20px;
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  margin-bottom: 24px;
}

.splash-title {
  font-size: 36px;
  font-weight: 800;
  color: var(--text-main, #ffffff);
  margin: 0 0 8px 0;
  letter-spacing: -1px;
}

.splash-subtitle {
  font-size: 14px;
  color: var(--text-muted, #9ca3af);
  margin: 0;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: .6;
    transform: scale(0.92);
  }
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>

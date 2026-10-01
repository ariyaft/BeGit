<template>
  <div class="author-avatar" :title="authorName" :style="{ width: size + 'px', height: size + 'px', fontSize: (size * 0.5) + 'px', background: backgroundGradient }">
    {{ initials }}
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  authorName?: string;
  size?: number;
}>(), {
  authorName: '?',
  size: 20
});

const initials = computed(() => {
  if (!props.authorName) return '?';
  return props.authorName.charAt(0).toUpperCase();
});

function stringToColors(str: string): [string, string] {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  
  const h1 = Math.abs(hash) % 360;
  const s1 = 65 + (Math.abs(hash) % 20);
  const l1 = 55 + (Math.abs(hash) % 10);
  
  const h2 = (h1 + 40 + (Math.abs(hash >> 8) % 40)) % 360;
  const s2 = 70 + (Math.abs(hash >> 16) % 20);
  const l2 = 45 + (Math.abs(hash >> 24) % 10);
  
  return [`hsl(${h1}, ${s1}%, ${l1}%)`, `hsl(${h2}, ${s2}%, ${l2}%)`];
}

const backgroundGradient = computed(() => {
  const [c1, c2] = stringToColors(props.authorName || '?');
  return `linear-gradient(135deg, ${c1}, ${c2})`;
});
</script>

<style scoped>
.author-avatar {
  border-radius: 50%;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  flex-shrink: 0;
  user-select: none;
}
</style>

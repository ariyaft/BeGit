<script setup lang="ts">
defineProps<{
  filePath: string;
  oldImage: string | null;
  newImage: string | null;
  isCommitComparison: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();
</script>

<template>
  <div class="image-diff-viewer">
    <div class="image-diff-header">
      <span>{{ filePath }}</span>
      <button class="close-image-diff" type="button" aria-label="Close image preview" @click="emit('close')">×</button>
    </div>
    <div class="image-comparison">
      <section class="image-version">
        <span class="image-version-label">{{ isCommitComparison ? 'Older revision' : 'Before' }}</span>
        <img v-if="oldImage" :src="oldImage" alt="Image before the selected change" />
        <p v-else class="missing-image">Image did not exist</p>
      </section>
      <section class="image-version">
        <span class="image-version-label">{{ isCommitComparison ? 'Newer revision' : 'After' }}</span>
        <img v-if="newImage" :src="newImage" alt="Image at the selected revision" />
        <p v-else class="missing-image">Image was deleted</p>
      </section>
    </div>
  </div>
</template>

<style scoped>
.image-diff-viewer {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  overflow: hidden;
}

.image-diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
}

.close-image-diff {
  border: 0;
  padding: 2px 7px;
  border-radius: 4px;
  color: var(--text-muted);
  background: transparent;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
}

.close-image-diff:hover {
  color: var(--text-main);
  background: var(--hover-bg);
}

.image-comparison {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  flex: 1;
  min-height: 0;
  gap: 16px;
  padding: 16px;
  overflow: auto;
}

.image-version {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 8px;
}

.image-version-label {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: .04em;
  text-transform: uppercase;
}

.image-version img {
  width: 100%;
  min-height: 120px;
  max-height: 70vh;
  object-fit: contain;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--panel-bg);
  background-image: linear-gradient(45deg, rgba(128, 128, 128, .12) 25%, transparent 25%), linear-gradient(-45deg, rgba(128, 128, 128, .12) 25%, transparent 25%), linear-gradient(45deg, transparent 75%, rgba(128, 128, 128, .12) 75%), linear-gradient(-45deg, transparent 75%, rgba(128, 128, 128, .12) 75%);
  background-position: 0 0, 0 8px, 8px -8px, -8px 0;
  background-size: 16px 16px;
}

.missing-image {
  display: grid;
  min-height: 120px;
  margin: 0;
  place-items: center;
  color: var(--text-muted);
  font-size: 13px;
  border: 1px dashed var(--border);
  border-radius: 6px;
}

@media (max-width: 760px) {
  .image-comparison {
    grid-template-columns: 1fr;
  }
}
</style>

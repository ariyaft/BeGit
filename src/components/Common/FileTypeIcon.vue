<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  fileName: string;
}>();


type FileType = { label: string; kind: string };


const fileTypes: Record<string, FileType> = {
  ts: { label: 'TS', kind: 'typescript' },
  tsx: { label: 'TS', kind: 'typescript' },
  js: { label: 'JS', kind: 'javascript' },
  jsx: { label: 'JS', kind: 'javascript' },
  vue: { label: 'V', kind: 'vue' },
  py: { label: 'PY', kind: 'python' },
  rs: { label: 'RS', kind: 'rust' },
  go: { label: 'GO', kind: 'go' },
  java: { label: 'J', kind: 'java' },
  c: { label: 'C', kind: 'c' },
  'cc|cpp|cxx|h|hpp': { label: 'C+', kind: 'cpp' },
  cs: { label: 'C#', kind: 'csharp' },
  rb: { label: 'RB', kind: 'ruby' },
  php: { label: 'PHP', kind: 'php' },
  swift: { label: 'SW', kind: 'swift' },
  kt: { label: 'KT', kind: 'kotlin' },
  html: { label: 'H', kind: 'html' },
  htm: { label: 'H', kind: 'html' },
  css: { label: 'CSS', kind: 'css' },
  scss: { label: 'SC', kind: 'scss' },
  sass: { label: 'SA', kind: 'scss' },
  less: { label: 'LE', kind: 'less' },
  json: { label: '{}', kind: 'json' },
  yaml: { label: 'YML', kind: 'yaml' },
  yml: { label: 'YML', kind: 'yaml' },
  toml: { label: 'T', kind: 'toml' },
  xml: { label: '</>', kind: 'xml' },
  md: { label: 'MD', kind: 'markdown' },
  mdx: { label: 'MD', kind: 'markdown' },
  txt: { label: 'TXT', kind: 'text' },
  pdf: { label: 'PDF', kind: 'pdf' },
  png: { label: 'IMG', kind: 'image' },
  jpg: { label: 'IMG', kind: 'image' },
  jpeg: { label: 'IMG', kind: 'image' },
  gif: { label: 'IMG', kind: 'image' },
  webp: { label: 'IMG', kind: 'image' },
  avif: { label: 'IMG', kind: 'image' },
  bmp: { label: 'IMG', kind: 'image' },
  ico: { label: 'IMG', kind: 'image' },
  svg: { label: 'SVG', kind: 'svg' },
  zip: { label: 'ZIP', kind: 'archive' },
  tar: { label: 'TAR', kind: 'archive' },
  gz: { label: 'GZ', kind: 'archive' },
  rar: { label: 'RAR', kind: 'archive' },
  lock: { label: 'LK', kind: 'lock' },
};

const specialFiles: Record<string, FileType> = {
  '.gitignore': { label: 'GIT', kind: 'git' },
  '.gitattributes': { label: 'GIT', kind: 'git' },
  '.gitmodules': { label: 'GIT', kind: 'git' },
  '.env': { label: 'ENV', kind: 'env' },
  dockerfile: { label: 'DKR', kind: 'docker' },
  makefile: { label: 'MK', kind: 'make' },
  readme: { label: 'MD', kind: 'markdown' },
};

const fileType = computed<FileType>(() => {
  const name = props.fileName.toLocaleLowerCase();
  if (specialFiles[name]) return specialFiles[name];
  if (name.startsWith('.env.')) return specialFiles['.env'];

  const extension = name.split('.').pop() ?? '';
  const entry = Object.entries(fileTypes).find(([extensions]) => extensions.split('|').includes(extension));
  return entry?.[1] ?? { label: extension.slice(0, 3).toUpperCase() || '•', kind: 'default' };
});
</script>

<template>
  <span
    class="file-type-icon"
    :class="`file-type-${fileType.kind}`"
    :title="fileType.label"
    aria-hidden="true"
  >{{ fileType.label }}</span>
</template>

<style scoped>
.file-type-icon {
  display: inline-grid;
  width: 17px;
  height: 17px;
  flex: 0 0 17px;
  place-items: center;
  border-radius: 3px;
  color: #fff;
  background: #8b95a7;
  box-shadow: inset 0 0 0 1px rgb(255 255 255 / 16%);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 7px;
  font-weight: 800;
  letter-spacing: -.35px;
  line-height: 1;
}

.file-type-typescript { background: #3178c6; }
.file-type-javascript { color: #3b3500; background: #f0db4f; }
.file-type-vue { background: #42b883; }
.file-type-python { background: #3776ab; }
.file-type-rust { background: #b05d22; }
.file-type-go { background: #00add8; }
.file-type-java { background: #e76f00; }
.file-type-c { background: #5c6bc0; }
.file-type-cpp { background: #4a90d9; }
.file-type-csharp { background: #7b4fbb; }
.file-type-ruby { background: #cc342d; }
.file-type-php { background: #777bb4; }
.file-type-swift { background: #f05138; }
.file-type-kotlin { background: #7f52ff; }
.file-type-html { background: #e44d26; }
.file-type-css { background: #264de4; }
.file-type-scss { background: #cd6799; }
.file-type-less { background: #1d365d; }
.file-type-json { background: #8d6e63; }
.file-type-yaml { background: #cb3837; }
.file-type-toml { background: #9c4221; }
.file-type-xml { background: #f57c00; }
.file-type-markdown { background: #536d7a; }
.file-type-text { background: #78909c; }
.file-type-pdf { background: #e53935; }
.file-type-image { background: #8e5ac7; }
.file-type-svg { background: #f9a825; }
.file-type-archive { background: #8d6e63; }
.file-type-lock { background: #607d8b; }
.file-type-git { background: #f05033; }
.file-type-env { background: #7cb342; }
.file-type-docker { background: #2496ed; }
.file-type-make { background: #6d7b93; }
</style>

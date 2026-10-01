import hljs from 'highlight.js';

export interface LanguageInfo {
  id: string;
  name: string;
  hljsLang: string;
}

const EXTENSION_MAP: Record<string, LanguageInfo> = {
  // Go
  go: { id: 'go', name: 'Go', hljsLang: 'go' },

  // Rust
  rs: { id: 'rust', name: 'Rust', hljsLang: 'rust' },

  // TypeScript & JavaScript
  ts: { id: 'typescript', name: 'TypeScript', hljsLang: 'typescript' },
  tsx: { id: 'tsx', name: 'TypeScript React', hljsLang: 'typescript' },
  js: { id: 'javascript', name: 'JavaScript', hljsLang: 'javascript' },
  jsx: { id: 'jsx', name: 'JavaScript React', hljsLang: 'javascript' },
  mjs: { id: 'javascript', name: 'JavaScript', hljsLang: 'javascript' },
  cjs: { id: 'javascript', name: 'JavaScript', hljsLang: 'javascript' },
  vue: { id: 'vue', name: 'Vue', hljsLang: 'xml' },
  svelte: { id: 'svelte', name: 'Svelte', hljsLang: 'xml' },
  astro: { id: 'astro', name: 'Astro', hljsLang: 'xml' },

  // Systems & Native
  c: { id: 'c', name: 'C', hljsLang: 'c' },
  h: { id: 'c', name: 'C Header', hljsLang: 'c' },
  cpp: { id: 'cpp', name: 'C++', hljsLang: 'cpp' },
  cc: { id: 'cpp', name: 'C++', hljsLang: 'cpp' },
  cxx: { id: 'cpp', name: 'C++', hljsLang: 'cpp' },
  hpp: { id: 'cpp', name: 'C++ Header', hljsLang: 'cpp' },
  cs: { id: 'csharp', name: 'C#', hljsLang: 'csharp' },
  java: { id: 'java', name: 'Java', hljsLang: 'java' },
  kt: { id: 'kotlin', name: 'Kotlin', hljsLang: 'kotlin' },
  kts: { id: 'kotlin', name: 'Kotlin Script', hljsLang: 'kotlin' },
  swift: { id: 'swift', name: 'Swift', hljsLang: 'swift' },
  py: { id: 'python', name: 'Python', hljsLang: 'python' },
  pyw: { id: 'python', name: 'Python', hljsLang: 'python' },
  rb: { id: 'ruby', name: 'Ruby', hljsLang: 'ruby' },
  php: { id: 'php', name: 'PHP', hljsLang: 'php' },

  // Markup & Web styles
  html: { id: 'html', name: 'HTML', hljsLang: 'xml' },
  htm: { id: 'html', name: 'HTML', hljsLang: 'xml' },
  xml: { id: 'xml', name: 'XML', hljsLang: 'xml' },
  svg: { id: 'svg', name: 'SVG', hljsLang: 'xml' },
  css: { id: 'css', name: 'CSS', hljsLang: 'css' },
  scss: { id: 'scss', name: 'SCSS', hljsLang: 'scss' },
  sass: { id: 'scss', name: 'Sass', hljsLang: 'scss' },
  less: { id: 'less', name: 'LESS', hljsLang: 'less' },

  // Config & Data
  json: { id: 'json', name: 'JSON', hljsLang: 'json' },
  jsonc: { id: 'json', name: 'JSON with Comments', hljsLang: 'json' },
  yaml: { id: 'yaml', name: 'YAML', hljsLang: 'yaml' },
  yml: { id: 'yaml', name: 'YAML', hljsLang: 'yaml' },
  toml: { id: 'toml', name: 'TOML', hljsLang: 'ini' },
  ini: { id: 'ini', name: 'INI', hljsLang: 'ini' },
  env: { id: 'bash', name: 'Environment', hljsLang: 'bash' },
  conf: { id: 'ini', name: 'Config', hljsLang: 'ini' },
  sql: { id: 'sql', name: 'SQL', hljsLang: 'sql' },

  // Shell & DevOps
  sh: { id: 'bash', name: 'Shell', hljsLang: 'bash' },
  bash: { id: 'bash', name: 'Bash', hljsLang: 'bash' },
  zsh: { id: 'bash', name: 'Zsh', hljsLang: 'bash' },
  dockerfile: { id: 'docker', name: 'Dockerfile', hljsLang: 'dockerfile' },
  md: { id: 'markdown', name: 'Markdown', hljsLang: 'markdown' },
  markdown: { id: 'markdown', name: 'Markdown', hljsLang: 'markdown' }
};

const FILENAME_MAP: Record<string, LanguageInfo> = {
  dockerfile: { id: 'docker', name: 'Dockerfile', hljsLang: 'dockerfile' },
  makefile: { id: 'bash', name: 'Makefile', hljsLang: 'bash' },
  gemfile: { id: 'ruby', name: 'Ruby (Gemfile)', hljsLang: 'ruby' },
  'cargo.toml': { id: 'toml', name: 'Cargo TOML', hljsLang: 'ini' },
  'cargo.lock': { id: 'toml', name: 'Cargo Lock', hljsLang: 'ini' },
  'package.json': { id: 'json', name: 'Package JSON', hljsLang: 'json' },
  'tsconfig.json': { id: 'json', name: 'TypeScript Config', hljsLang: 'json' },
  '.gitignore': { id: 'bash', name: 'Git Ignore', hljsLang: 'bash' },
  '.env': { id: 'bash', name: 'Env', hljsLang: 'bash' }
};

/**
 * Detect language from file path or file name.
 */
export function detectLanguage(filePath: string): LanguageInfo {
  if (!filePath) {
    return { id: 'plaintext', name: 'Plain Text', hljsLang: 'plaintext' };
  }

  const parts = filePath.replace(/\\/g, '/').split('/');
  const fileName = (parts[parts.length - 1] || '').toLowerCase();

  // Check exact filename match
  if (FILENAME_MAP[fileName]) {
    return FILENAME_MAP[fileName];
  }

  // Check extension match
  const dotIndex = fileName.lastIndexOf('.');
  if (dotIndex !== -1) {
    const ext = fileName.substring(dotIndex + 1);
    if (EXTENSION_MAP[ext]) {
      return EXTENSION_MAP[ext];
    }
  }

  return { id: 'plaintext', name: 'Plain Text', hljsLang: 'plaintext' };
}

/**
 * Escape raw HTML to prevent injection
 */
export function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

/**
 * Highlight a single line of code with Highlight.js.
 */
export function highlightCodeLine(codeLine: string, languageInfo: LanguageInfo): string {
  if (!codeLine) return '';

  if (!languageInfo.hljsLang || languageInfo.hljsLang === 'plaintext' || !hljs.getLanguage(languageInfo.hljsLang)) {
    return escapeHtml(codeLine);
  }

  try {
    const res = hljs.highlight(codeLine, { language: languageInfo.hljsLang, ignoreIllegals: true });
    return res.value || escapeHtml(codeLine);
  } catch (e) {
    return escapeHtml(codeLine);
  }
}

/**
 * Render indentation guide markers for beautifying code structure.
 */
export function formatIndentation(line: string, tabSize: number = 2): { indentHtml: string; restHtml: string } {
  const match = line.match(/^([ \t]+)/);
  if (!match) {
    return { indentHtml: '', restHtml: line };
  }

  const indentStr = match[1];
  let spacesCount = 0;
  for (let i = 0; i < indentStr.length; i++) {
    if (indentStr[i] === '\t') {
      spacesCount += tabSize;
    } else {
      spacesCount += 1;
    }
  }

  const levels = Math.floor(spacesCount / tabSize);
  const remainder = spacesCount % tabSize;

  let indentHtml = '';
  for (let i = 0; i < levels; i++) {
    indentHtml += `<span class="indent-guide" style="width: ${tabSize}ch;"><span class="indent-line"></span></span>`;
  }
  if (remainder > 0) {
    indentHtml += `<span class="indent-guide indent-sub" style="width: ${remainder}ch;"></span>`;
  }

  return {
    indentHtml,
    restHtml: line.substring(indentStr.length)
  };
}

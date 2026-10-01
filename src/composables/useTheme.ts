import { ref } from 'vue';

const isDarkTheme = ref(true);
let initialized = false;

function applyTheme(dark: boolean) {
  document.body.classList.toggle('light-theme', !dark);
  document.documentElement.style.colorScheme = dark ? 'dark' : 'light';
}

export function useTheme() {
  if (!initialized) {
    const saved = localStorage.getItem('git-tree-theme');
    if (saved === 'light') {
      isDarkTheme.value = false;
    } else {
      isDarkTheme.value = true;
    }
    applyTheme(isDarkTheme.value);
    initialized = true;
  }

  function toggleTheme() {
    isDarkTheme.value = !isDarkTheme.value;
    applyTheme(isDarkTheme.value);
    localStorage.setItem('git-tree-theme', isDarkTheme.value ? 'dark' : 'light');
  }

  return {
    isDarkTheme,
    toggleTheme
  };
}

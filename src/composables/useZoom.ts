import { ref } from 'vue';

const MIN_ZOOM = 60;
const MAX_ZOOM = 180;
const ZOOM_STEP = 10;
const DEFAULT_ZOOM = 100;

const zoomLevel = ref(DEFAULT_ZOOM);
let initialized = false;

function applyZoom(level: number) {
  const clamped = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, level));
  zoomLevel.value = clamped;
  (document.documentElement.style as any).zoom = `${clamped}%`;
  document.documentElement.style.setProperty('--app-zoom', `${clamped / 100}`);
  localStorage.setItem('begit-zoom-level', clamped.toString());
}

export function useZoom() {
  if (!initialized && typeof window !== 'undefined') {
    const saved = localStorage.getItem('begit-zoom-level');
    if (saved && !isNaN(Number(saved))) {
      const parsed = Number(saved);
      applyZoom(parsed);
    } else {
      applyZoom(DEFAULT_ZOOM);
    }

    // Global keyboard shortcuts for zoom
    window.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        if (e.key === '=' || e.key === '+') {
          e.preventDefault();
          zoomIn();
        } else if (e.key === '-' || e.key === '_') {
          e.preventDefault();
          zoomOut();
        } else if (e.key === '0') {
          e.preventDefault();
          resetZoom();
        }
      }
    });

    initialized = true;
  }

  function zoomIn() {
    applyZoom(zoomLevel.value + ZOOM_STEP);
  }

  function zoomOut() {
    applyZoom(zoomLevel.value - ZOOM_STEP);
  }

  function resetZoom() {
    applyZoom(DEFAULT_ZOOM);
  }

  function setZoom(val: number) {
    applyZoom(val);
  }

  return {
    zoomLevel,
    zoomIn,
    zoomOut,
    resetZoom,
    setZoom,
    minZoom: MIN_ZOOM,
    maxZoom: MAX_ZOOM
  };
}

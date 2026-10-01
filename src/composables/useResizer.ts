import { ref } from 'vue';

export function useResizer(
  initialLeftWidth: number = 240, 
  initialRightWidth: number = 320
) {
  const savedLeft = localStorage.getItem('gitTreeLeftWidth');
  const savedRight = localStorage.getItem('gitTreeRightWidth');
  
  const leftSidebarWidth = ref(savedLeft ? parseInt(savedLeft, 10) : initialLeftWidth);
  const rightSidebarWidth = ref(savedRight ? parseInt(savedRight, 10) : initialRightWidth);
  const isDraggingLeft = ref(false);
  const isDraggingRight = ref(false);

  const prevLeftSidebarWidth = ref(leftSidebarWidth.value || 240);
  const prevRightSidebarWidth = ref(rightSidebarWidth.value || 320);

  let startX = 0;
  let startWidth = 0;

  function getZoomFactor(): number {
    const rootZoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom'));
    if (!isNaN(rootZoom) && rootZoom > 0) return rootZoom;
    const docZoom = parseFloat((document.documentElement.style as any).zoom);
    if (!isNaN(docZoom) && docZoom > 0) return docZoom / 100;
    return 1;
  }

  function toggleLeftSidebar() {
    if (leftSidebarWidth.value > 0) {
      prevLeftSidebarWidth.value = leftSidebarWidth.value;
      leftSidebarWidth.value = 0;
    } else {
      leftSidebarWidth.value = prevLeftSidebarWidth.value > 0 ? prevLeftSidebarWidth.value : 240;
    }
    localStorage.setItem('gitTreeLeftWidth', leftSidebarWidth.value.toString());
  }

  function toggleRightSidebar() {
    if (rightSidebarWidth.value > 0) {
      prevRightSidebarWidth.value = rightSidebarWidth.value;
      rightSidebarWidth.value = 0;
    } else {
      rightSidebarWidth.value = prevRightSidebarWidth.value > 0 ? prevRightSidebarWidth.value : 320;
    }
    localStorage.setItem('gitTreeRightWidth', rightSidebarWidth.value.toString());
  }

  function startDragLeft(e: MouseEvent) {
    e?.preventDefault?.();
    e?.stopPropagation?.();
    window.getSelection()?.removeAllRanges();
    isDraggingLeft.value = true;
    startX = e.clientX;
    startWidth = leftSidebarWidth.value;
    document.addEventListener('mousemove', onDragLeft);
    document.addEventListener('mouseup', stopDrag);
    document.body.classList.add('is-resizing');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    document.documentElement.style.userSelect = 'none';
  }

  function onDragLeft(e: MouseEvent) {
    if (!isDraggingLeft.value) return;
    const zoom = getZoomFactor();
    const deltaX = (e.clientX - startX) / zoom;
    let newWidth = Math.round(startWidth + deltaX);
    if (newWidth < 150) newWidth = 150;
    const maxAllowed = Math.max(150, (window.innerWidth / zoom) - rightSidebarWidth.value - 220);
    if (newWidth > maxAllowed) newWidth = maxAllowed;
    if (newWidth > 600) newWidth = 600;
    leftSidebarWidth.value = newWidth;
  }

  function startDragRight(e: MouseEvent) {
    e?.preventDefault?.();
    e?.stopPropagation?.();
    window.getSelection()?.removeAllRanges();
    isDraggingRight.value = true;
    startX = e.clientX;
    startWidth = rightSidebarWidth.value;
    document.addEventListener('mousemove', onDragRight);
    document.addEventListener('mouseup', stopDrag);
    document.body.classList.add('is-resizing');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    document.documentElement.style.userSelect = 'none';
  }

  function onDragRight(e: MouseEvent) {
    if (!isDraggingRight.value) return;
    const zoom = getZoomFactor();
    const deltaX = (e.clientX - startX) / zoom;
    let newWidth = Math.round(startWidth - deltaX);
    if (newWidth < 200) newWidth = 200;
    const maxAllowed = Math.max(200, (window.innerWidth / zoom) - leftSidebarWidth.value - 220);
    if (newWidth > maxAllowed) newWidth = maxAllowed;
    if (newWidth > 800) newWidth = 800;
    rightSidebarWidth.value = newWidth;
  }

  function stopDrag() {
    isDraggingLeft.value = false;
    isDraggingRight.value = false;
    document.removeEventListener('mousemove', onDragLeft);
    document.removeEventListener('mousemove', onDragRight);
    document.removeEventListener('mouseup', stopDrag);
    document.body.classList.remove('is-resizing');
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    document.documentElement.style.userSelect = '';
    window.getSelection()?.removeAllRanges();
    
    localStorage.setItem('gitTreeLeftWidth', leftSidebarWidth.value.toString());
    localStorage.setItem('gitTreeRightWidth', rightSidebarWidth.value.toString());
  }

  return {
    leftSidebarWidth,
    rightSidebarWidth,
    isDraggingLeft,
    isDraggingRight,
    startDragLeft,
    startDragRight,
    stopDrag,
    toggleLeftSidebar,
    toggleRightSidebar
  };
}

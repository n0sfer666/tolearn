export function grab(element: HTMLElement): void {
  queueMicrotask(() => element.focus());
}

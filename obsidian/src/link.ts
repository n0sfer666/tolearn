export function deepLink(roadmap: string, topic: string): string {
  const where = new URLSearchParams({ roadmap, topic });
  return `tolearn://topic?${where.toString()}`;
}

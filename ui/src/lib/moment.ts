const pad = (value: number) => String(value).padStart(2, "0");

export function moment(seconds: number): string {
  const at = new Date(seconds * 1000);
  return `${pad(at.getDate())}-${pad(at.getMonth() + 1)} ${pad(at.getHours())}:${pad(at.getMinutes())}`;
}

const DAY = 86_400_000;

type Step = readonly [number, Intl.RelativeTimeFormatUnit];

const STEPS: readonly Step[] = [
  [365, "year"],
  [30, "month"],
  [7, "week"],
];
const DAYS: Step = [1, "day"];

export function ago(iso: string, locale: string, today: Date): string {
  const now = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  const days = Math.round((now - Date.parse(`${iso}T00:00:00Z`)) / DAY);
  const [size, unit] = STEPS.find(([size]) => days >= size) ?? DAYS;
  return new Intl.RelativeTimeFormat(locale, { numeric: "auto" }).format(-Math.floor(days / size), unit);
}

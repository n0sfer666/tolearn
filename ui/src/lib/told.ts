import { coded, explain } from "./toast";

export function told(failure: unknown, known: ReadonlyMap<string, string>): string {
  return known.get(coded(failure)) ?? explain(failure);
}

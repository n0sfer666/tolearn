import type { VariantView } from "../ipc";

export interface Ranked {
  variant: VariantView;
  choice: number;
}

export function ranked(variants: VariantView[]): Ranked[] {
  return variants
    .map((variant, choice) => ({ variant, choice }))
    .sort((one, two) => Number(two.variant.recommended) - Number(one.variant.recommended));
}

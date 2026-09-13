function place(program: string, node: string): URLSearchParams {
  const params = new URLSearchParams({ program });
  if (node !== "" && node !== program) params.set("node", node);
  return params;
}

function staged(page: string, locale: string, program: string, node: string, stage: string): string {
  const params = place(program, node);
  params.set("stage", stage);
  return `/${locale}/${page}/?${params.toString()}`;
}

export function nodeHref(locale: string, program: string, node = ""): string {
  return `/${locale}/program/?${place(program, node).toString()}`;
}

export function stageHref(locale: string, program: string, node: string, stage: string): string {
  return staged("stage", locale, program, node, stage);
}

export function nextHref(locale: string, program: string, node: string, stage: string): string {
  return staged("next", locale, program, node, stage);
}

export function go(href: string): void {
  location.assign(href);
}

export const KEY = "tolearn.theme";

function inline(html) {
  return [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(([, code]) => code);
}

function only(codes, where) {
  if (codes.length !== 1) throw new Error(`ожидался один скрипт ${where}, найдено ${codes.length}`);
  return codes[0];
}

export function early(html) {
  const head = html.slice(0, html.indexOf("<body"));
  return only(inline(head).filter((code) => code.includes(KEY)), "темы в <head>");
}


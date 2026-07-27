import { plural, strings } from "../src/i18n/index.ts";

export const pluralize = plural;

export function sources(locale) {
  return strings(locale);
}

function isPlural(value) {
  return typeof value === "object" && typeof value.other === "string";
}

export function keys(dictionary, prefix = "") {
  return Object.entries(dictionary).flatMap(([key, value]) =>
    typeof value === "string" || isPlural(value) ? [`${prefix}${key}`] : keys(value, `${prefix}${key}.`),
  );
}

export function plurals(dictionary, prefix = "") {
  return Object.entries(dictionary).flatMap(([key, value]) => {
    if (typeof value === "string") return [];
    if (isPlural(value)) return [[`${prefix}${key}`, Object.keys(value)]];
    return plurals(value, `${prefix}${key}.`);
  });
}

export function categories(locale) {
  return new Intl.PluralRules(locale).resolvedOptions().pluralCategories;
}

export function missing(expected, actual) {
  return expected.filter((key) => !actual.includes(key));
}

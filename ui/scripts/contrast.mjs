const LIGHT_DARK = /^light-dark\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)$/;

export function themes(value) {
  const paired = LIGHT_DARK.exec(value.trim());
  return paired ? { light: paired[1], dark: paired[2] } : { light: value.trim(), dark: value.trim() };
}

function channels(hex) {
  const digits = hex.replace("#", "");
  const full = digits.length === 3 ? [...digits].map((digit) => digit + digit).join("") : digits;
  return [0, 2, 4].map((at) => Number.parseInt(full.slice(at, at + 2), 16) / 255);
}

export function luminance(hex) {
  const [red, green, blue] = channels(hex).map((channel) =>
    channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4,
  );
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

export function ratio(one, other) {
  const first = luminance(one);
  const second = luminance(other);
  return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
}

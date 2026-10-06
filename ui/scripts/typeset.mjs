const TYPE = ["font-size", "font-family", "line-height"];
const MOTION = /^(?:transition|animation)(?:-|$)/;
const TIME = /(?<![\w.-])\d*\.?\d+m?s\b/;
const CALM = /prefers-reduced-motion:\s*no-preference/;

export function typeset(found, media, where, file) {
  const problems = [];

  for (const { property, value } of found) {
    if (TYPE.includes(property) && value !== "inherit" && !new RegExp(`^var\\(--${property}-[a-z]+\\)$`).test(value)) {
      problems.push({ file, message: `${property} «${value}» в ${where} — значение берётся из --${property}-*` });
    }
    if (property === "font") {
      problems.push({ file, message: `font «${value}» в ${where} — шорткат прячет размер и семейство, пиши их токенами` });
    }
    if (!MOTION.test(property) || value === "none") continue;
    if (TIME.test(value)) {
      problems.push({ file, message: `время в «${property}: ${value}» в ${where} — длительность берётся из --duration` });
    }
    if (!CALM.test(media ?? "")) {
      problems.push({
        file,
        message: `${property} в ${where} вне @media (prefers-reduced-motion: no-preference)`,
      });
    }
  }

  return problems;
}

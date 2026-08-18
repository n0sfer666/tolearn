import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";

let rich;
let spans;

before(async () => {
  ({ rich } = await island("rich", "src/lib", "ts"));
  ({ spans } = await island("spans", "src/lib", "ts"));
}, { timeout: 300_000 });

const kinds = (chunks) => chunks.map((chunk) => chunk.kind);
const plain = (chunk) => chunk.spans.map((span) => span.text).join("");

test("пустой текст не даёт ни одного блока", () => {
  assert.deepEqual(rich(""), []);
  assert.deepEqual(rich("   \n\n  "), []);
});

test("абзацы режутся по пустой строке, перенос внутри абзаца склеивается", () => {
  const chunks = rich("Первый абзац\nпродолжение\n\nВторой абзац");

  assert.deepEqual(kinds(chunks), ["para", "para"]);
  assert.equal(plain(chunks[0]), "Первый абзац продолжение");
  assert.equal(plain(chunks[1]), "Второй абзац");
});

test("нумерованный список внутри сплошного абзаца становится пунктами", () => {
  const chunks = rich("Порядок обязателен, шаг 1 делается до запусков. 1. Запиши предсказание. 2. Прогони модель. 3. Сохрани лог.");

  assert.deepEqual(kinds(chunks), ["para", "item", "item", "item"]);
  assert.equal(plain(chunks[0]), "Порядок обязателен, шаг 1 делается до запусков.");
  assert.equal(plain(chunks[1]), "Запиши предсказание.");
  assert.equal(plain(chunks[3]), "Сохрани лог.");
  assert.ok(chunks[1].ordered);
});

test("одинокий номер не считается списком", () => {
  const chunks = rich("Модель весит 4.5 ГБ. 1. Это единственный пункт без продолжения.");

  assert.deepEqual(kinds(chunks), ["para"]);
});

test("номера вне порядка не режут абзац", () => {
  const chunks = rich("Смотри пункт 3. И ещё 7. Оба не начинают список.");

  assert.deepEqual(kinds(chunks), ["para"]);
});

test("список со своих строк разбирается по маркеру", () => {
  const chunks = rich("- веса\n- KV-кэш\n\n1. поднять\n2. замерить");

  assert.deepEqual(kinds(chunks), ["item", "item", "item", "item"]);
  assert.equal(chunks[0].ordered, false);
  assert.equal(plain(chunks[0]), "веса");
  assert.equal(chunks[2].ordered, true);
  assert.equal(plain(chunks[2]), "поднять");
});

test("продолжение пункта с отступом остаётся внутри пункта", () => {
  const chunks = rich(
    "1. Запиши предсказание,\n   три числа с обоснованием.\n2. Прогони модель\n   и сохрани вывод.",
  );

  assert.deepEqual(kinds(chunks), ["item", "item"]);
  assert.equal(plain(chunks[0]), "Запиши предсказание, три числа с обоснованием.");
  assert.equal(plain(chunks[1]), "Прогони модель и сохрани вывод.");
});

test("пояснение под буллетом не рвёт список", () => {
  const chunks = rich("- `ollama ps`\n  показывает занятую память.\n- `nvidia-smi`\n  показывает GPU.");

  assert.deepEqual(kinds(chunks), ["item", "item"]);
  assert.equal(chunks[0].ordered, false);
  assert.equal(plain(chunks[0]), "ollama ps показывает занятую память.");
});

test("пустая строка закрывает пункт, следующий текст — абзац", () => {
  const chunks = rich("1. поднять\n2. замерить\n\nЧто предъявить: каталог с логами.");

  assert.deepEqual(kinds(chunks), ["item", "item", "para"]);
  assert.equal(plain(chunks[2]), "Что предъявить: каталог с логами.");
});

test("заголовок обрывает абзац и не склеивается со следующей строкой", () => {
  const chunks = rich("вступление\n## Практика\nпервый шаг");

  assert.deepEqual(kinds(chunks), ["para", "head", "para"]);
  assert.equal(plain(chunks[2]), "первый шаг");
});

test("заголовок и цитата — свои блоки", () => {
  const chunks = rich("## Практика\n\n> так делать нельзя\n\nобычный текст");

  assert.deepEqual(kinds(chunks), ["head", "quote", "para"]);
  assert.equal(chunks[0].level, 2);
  assert.equal(plain(chunks[0]), "Практика");
  assert.equal(plain(chunks[1]), "так делать нельзя");
});

test("огороженный блок кода сохраняется дословно", () => {
  const chunks = rich("Запусти:\n\n```sh\nollama ps\nollama create m\n```\n\nи сравни");

  assert.deepEqual(kinds(chunks), ["para", "code", "para"]);
  assert.equal(chunks[1].text, "ollama ps\nollama create m");
});

test("незакрытый блок кода не съедает остаток молча", () => {
  const chunks = rich("Вот команда:\n\n```\nollama ps");

  assert.deepEqual(kinds(chunks), ["para", "code"]);
  assert.equal(chunks[1].text, "ollama ps");
});

test("разметка внутри блока кода остаётся текстом", () => {
  const chunks = rich("```\n**не жирный** и `не код`\n```");

  assert.equal(chunks[0].text, "**не жирный** и `не код`");
});

test("инлайн-разметка разбирается по видам", () => {
  const parts = spans("вывод `ollama ps` в **файл** и *потом* [док](https://ollama.com)");

  assert.deepEqual(parts, [
    { kind: "plain", text: "вывод " },
    { kind: "code", text: "ollama ps" },
    { kind: "plain", text: " в " },
    { kind: "strong", text: "файл" },
    { kind: "plain", text: " и " },
    { kind: "em", text: "потом" },
    { kind: "plain", text: " " },
    { kind: "link", text: "док", href: "https://ollama.com" },
  ]);
});

test("текст без разметки остаётся одним куском", () => {
  assert.deepEqual(spans("просто текст"), [{ kind: "plain", text: "просто текст" }]);
});

test("ссылка на посторонней схеме рисуется текстом, а не ссылкой", () => {
  const parts = spans("нажми [сюда](javascript:alert(1))");

  assert.deepEqual(new Set(kinds(parts)), new Set(["plain"]));
  assert.equal(parts.map((part) => part.text).join(""), "нажми [сюда](javascript:alert(1))");
});

test("одинокая обратная кавычка не ломает разбор", () => {
  assert.deepEqual(spans("цена ` за токен"), [{ kind: "plain", text: "цена ` за токен" }]);
});

test("подчёркивание внутри имени не делает курсив", () => {
  const parts = spans("файл ollama_default_log.txt");

  assert.deepEqual(kinds(parts), ["plain"]);
});

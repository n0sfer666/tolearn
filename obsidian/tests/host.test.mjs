import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { deepLink } from "../src/link.ts";
import { text, world } from "../src/host.ts";
import { shown } from "../src/view.ts";

const NOTE = `---
tolearn:
  roadmap: llm-agents-base
  topic: local-runtime
---

Конспект.
`;

function yard() {
  const root = mkdtempSync(join(tmpdir(), "tolearn-obsidian-"));
  const bundle = join(root, "bundle");
  mkdirSync(bundle, { recursive: true });
  writeFileSync(
    join(root, "registry.yaml"),
    `schema: tolearn/registry/v1\nprograms:\n  - id: "llm-agents-base"\n    title: "Агенты на LLM"\n    path: "${bundle}"\n    opened_at: "2026-07-29T10:00:00Z"\n`,
  );
  writeFileSync(
    join(bundle, "progress.yaml"),
    `schema: learning-roadmap/progress/v1\nroadmap_id: llm-agents-base\ntopics:\n  local-runtime:\n    status: in_progress\n    next_review_at: null\n`,
  );
  return { root, bundle };
}

test("статус берётся из каталога данных на диске", () => {
  const { root } = yard();
  try {
    const found = shown(NOTE, world(root, "2026-07-29"));

    assert.equal(found.kind, "topic");
    assert.equal(found.title, "Агенты на LLM");
    assert.equal(found.status, "in_progress");
    assert.equal(found.overdue, false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("наличие ключа рядом с конспектами читается как шифрование", () => {
  const { root } = yard();
  try {
    mkdirSync(join(root, "notes"), { recursive: true });
    writeFileSync(join(root, "notes", "identity.age"), "запертый ключ");

    assert.equal(world(root, "2026-07-29").sealed, true);
    assert.equal(shown(NOTE, world(root, "2026-07-29")).kind, "sealed");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("пустой каталог данных не роняет плагин", () => {
  const root = mkdtempSync(join(tmpdir(), "tolearn-obsidian-"));
  try {
    const state = world(root, "2026-07-29");

    assert.deepEqual(state.programs, []);
    assert.equal(state.sealed, false);
    assert.equal(shown(NOTE, state).kind, "no-program");
    assert.equal(text(join(root, "нет-файла")), null);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("ссылка в приложение экранирует значения", () => {
  assert.equal(
    deepLink("llm-agents-base", "local-runtime"),
    "tolearn://topic?roadmap=llm-agents-base&topic=local-runtime",
  );
  assert.equal(deepLink("a b", "c&d"), "tolearn://topic?roadmap=a+b&topic=c%26d");
});

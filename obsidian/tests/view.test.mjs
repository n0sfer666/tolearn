import assert from "node:assert/strict";
import test from "node:test";

import { bound } from "../src/bound.ts";
import { state } from "../src/progress.ts";
import { programs } from "../src/registry.ts";
import { describe, dictionary } from "../src/text.ts";
import { shown } from "../src/view.ts";

const NOTE = `---
tolearn:
  roadmap: llm-agents-base
  topic: local-runtime
---

Конспект.
`;

const REGISTRY = `schema: tolearn/registry/v1
programs:
  - id: "llm-agents-base"
    title: "Агенты на LLM"
    path: "/programs/llm"
    opened_at: "2026-07-29T10:00:00Z"
`;

const PROGRESS = `schema: learning-roadmap/progress/v1
roadmap_id: llm-agents-base
topics:
  local-runtime:
    status: passed
    attempts: []
    passed_at: "2026-07-01"
    next_review_at: "2026-07-20"
    gaps: []
  openai-compatible-api:
    status: blocked
    attempts: []
    passed_at: null
    next_review_at: null
    gaps: []
`;

function world(options = {}) {
  return {
    sealed: options.sealed === true,
    programs: programs(options.registry ?? REGISTRY),
    progress: () => options.progress ?? PROGRESS,
    today: options.today ?? "2026-07-29",
  };
}

test("фронтматтер даёт программу и тему", () => {
  assert.deepEqual(bound(NOTE), { roadmap: "llm-agents-base", topic: "local-runtime" });
});

test("чужая заметка темой не притворяется", () => {
  assert.equal(bound("# Просто заметка\n"), null);
  assert.equal(bound("---\ntags: [x]\n---\n\nтекст\n"), null);
});

test("привязка читается только из фронтматтера, а не из текста заметки", () => {
  assert.equal(bound("---\ntags: [x]\n---\n\nroadmap: llm\ntopic: local\n"), null);
  assert.equal(bound("---\ntolearn:\n  roadmap: llm\n  topic: local\n"), null);
});

test("реестр разбирается со всеми программами", () => {
  assert.deepEqual(programs(REGISTRY), [
    { id: "llm-agents-base", title: "Агенты на LLM", path: "/programs/llm" },
  ]);
});

test("статус темы читается из progress.yaml", () => {
  assert.deepEqual(state(PROGRESS, "local-runtime"), {
    status: "passed",
    due: "2026-07-20",
  });
  assert.equal(state(PROGRESS, "нет-такой"), null);
});

test("пустой срок повторения не превращается в дату", () => {
  assert.deepEqual(state(PROGRESS, "openai-compatible-api"), {
    status: "blocked",
    due: null,
  });
  const tilde = PROGRESS.replace("next_review_at: null", "next_review_at: ~");
  assert.equal(state(tilde, "openai-compatible-api").due, null);
});

test("статус темы читается и из progress.json", () => {
  const json = JSON.stringify({
    topics: { "local-runtime": { status: "in_progress", next_review_at: null } },
  });
  assert.deepEqual(state(json, "local-runtime"), { status: "in_progress", due: null });
});

test("заметка со статусом показывает программу, тему и ссылку", () => {
  const found = shown(NOTE, world());

  assert.equal(found.kind, "topic");
  assert.equal(found.title, "Агенты на LLM");
  assert.equal(found.status, "passed");
  assert.equal(found.overdue, true);
  assert.equal(found.link, "tolearn://topic?roadmap=llm-agents-base&topic=local-runtime");
});

test("срок повторения в будущем не считается просроченным", () => {
  const found = shown(NOTE, world({ today: "2026-07-01" }));

  assert.equal(found.kind, "topic");
  assert.equal(found.overdue, false);
});

test("заблокированная тема показывается как не начатая", () => {
  const note = NOTE.replace("local-runtime", "openai-compatible-api");

  const found = shown(note, world());

  assert.equal(found.kind, "topic");
  assert.equal(found.status, "todo");
});

test("при включённом шифровании плагин говорит об этом, а не молчит", () => {
  const found = shown(NOTE, world({ sealed: true }));

  assert.equal(found.kind, "sealed");
  assert.equal(
    describe(found, dictionary("ru")),
    "конспекты зашифрованы: статусы видит только приложение",
  );
});

test("неизвестная программа и неизвестная тема названы по имени", () => {
  const noProgram = shown(NOTE, world({ registry: "programs: []\n" }));
  assert.equal(noProgram.kind, "no-program");
  assert.match(describe(noProgram, dictionary("ru")), /llm-agents-base/);

  const noTopic = shown(NOTE.replace("local-runtime", "missing"), world());
  assert.equal(noTopic.kind, "no-topic");
  assert.match(describe(noTopic, dictionary("ru")), /missing/);
});

test("чужая заметка описывается по-английски тоже", () => {
  const found = shown("текст без фронтматтера\n", world());

  assert.equal(found.kind, "foreign");
  assert.equal(describe(found, dictionary("en")), "the note is not bound to a tolearn topic");
});

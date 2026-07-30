import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Topic;
let Read;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/topic/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Topic } = await island("Topic"));
    ({ default: Read } = await island("Read"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const MATERIAL = {
  title: "Гайд по квантованию",
  url: "https://example.invalid/quant",
  kind: "doc",
  tier: "T1",
  lang: "ru",
  stale: false,
  delta: null,
  offline: "absent",
  note: "",
};

const TOPIC = {
  id: "local-runtime",
  title: "Локальный рантайм",
  stage: 1,
  checkpoint: false,
  status: "in_progress",
  hours: { min: 4, max: 6 },
  blocked_by: [],
  verified_at: "2026-07-26",
  outdated: false,
  outcomes: ["Поднимает модель"],
  misconceptions: [],
  materials: [MATERIAL],
  practice: {
    kind: "ops",
    tier: "P2",
    task: "Подними модель локально",
    deliverable: "Лог запуска",
    starting_point: null,
    fallback: null,
    time_box_min: 90,
    smoke_checked: true,
    constraints: [],
    acceptance: [],
  },
  questions: [],
  exam: { focus: "память", artifact_required: false, max_exchanges: 12 },
};

const STATE = {
  total: 1,
  done: 0,
  current: MATERIAL.url,
  finished: false,
  cancelled: false,
  bytes: 0,
  saved: [],
  skipped: [],
  failed: [],
};

function mountTopic(states, refuses = false) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  let step = 0;
  let saved = false;
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (refuses && name === "offline_state") {
      return Promise.reject({ code: "offline.job", message: "работа потерялась" });
    }
    if (name === "topic") {
      const materials = [{ ...MATERIAL, offline: saved ? "saved" : "absent" }];
      return Promise.resolve({ ...TOPIC, materials });
    }
    if (name === "save_offline") return Promise.resolve({ job: "job-1", total: 1 });
    if (name === "offline_state") {
      const out = states[Math.min(step, states.length - 1)];
      step += 1;
      saved = saved || (out.finished && out.saved.length > 0);
      return Promise.resolve(out);
    }
    if (name === "stop_offline") return Promise.resolve({ stopping: true });
    if (name === "note") return Promise.resolve({ body: "", stamp: null, path: "" });
    throw new Error(`лишняя команда ${name}`);
  };
  const said = toasts(document.defaultView);
  const dispose = render(
    () =>
      Topic({
        text: ru,
        locale: "ru",
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        today: "2026-07-27",
        call,
      }),
    host,
  );
  return { host, calls, dispose, said };
}

const materials = (host) => host.querySelector('[data-section="materials"]');

const tick = async (times) => {
  for (let index = 0; index < times; index += 1) {
    await new Promise((resolve) => setTimeout(resolve, 350));
  }
};

test("выгрузка запускается с шапки материалов и отчитывается по концу", async () => {
  const finished = {
    ...STATE,
    done: 1,
    current: "",
    finished: true,
    bytes: 2048,
    saved: [MATERIAL.url],
  };
  const { host, calls, dispose } = mountTopic([STATE, finished]);
  await settled();

  materials(host).querySelector("[data-save-offline]").click();
  await settled();

  assert.deepEqual(calls.find((made) => made.name === "save_offline"), {
    name: "save_offline",
    payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime", again: null },
  });
  assert.match(materials(host).querySelector("[data-unload-progress]").textContent, /0\/1/);

  await tick(2);

  assert.equal(materials(host).querySelector("[data-unload-progress]"), null, "прогресс не убрался");
  assert.match(materials(host).querySelector("[data-unload-report]").textContent, /1/);
  assert.equal(
    materials(host).querySelector("[data-offline-open]").getAttribute("href"),
    `/ru/read/?url=${encodeURIComponent(MATERIAL.url)}&program=${encodeURIComponent("/programs/llm-agents-base")}&topic=local-runtime`,
    "сохранённый материал не получил ход в читалку с темой и программой",
  );
  dispose();
});

test("упавшая выгрузка предлагает повтор и шлёт номер прошлой", async () => {
  const broken = {
    ...STATE,
    done: 1,
    finished: true,
    failed: [{ url: MATERIAL.url, why: "сеть не ответила" }],
  };
  const { host, calls, dispose } = mountTopic([broken]);
  await settled();

  materials(host).querySelector("[data-save-offline]").click();
  await tick(1);

  const again = materials(host).querySelector("[data-save-offline]");
  assert.equal(again.textContent, ru.offline.again);

  again.click();
  await settled();

  const asked = calls.filter((made) => made.name === "save_offline");
  assert.deepEqual(asked.at(-1).payload, {
    bundle: "/programs/llm-agents-base",
    topic: "local-runtime",
    again: "job-1",
  });
  dispose();
});

test("отвалившаяся выгрузка говорит об этом один раз и не долбит опросом", async () => {
  const { host, calls, dispose, said } = mountTopic([STATE], true);
  await settled();

  materials(host).querySelector("[data-save-offline]").click();
  await tick(3);

  assert.deepEqual(said, [{ tone: "error", text: "работа потерялась" }]);
  assert.equal(calls.filter((made) => made.name === "offline_state").length, 1);
  assert.ok(materials(host).querySelector("[data-save-offline]"), "кнопка не вернулась");
  dispose();
});

test("идущую выгрузку можно остановить", async () => {
  const { host, calls, dispose } = mountTopic([STATE]);
  await settled();

  materials(host).querySelector("[data-save-offline]").click();
  await tick(1);

  materials(host).querySelector("[data-unload-stop]").click();
  await settled();

  assert.deepEqual(calls.at(-1), { name: "stop_offline", payload: { job: "job-1" } });
  dispose();
});

function mountRead(answer) {
  const host = document.createElement("div");
  document.body.append(host);
  const call = (name) => {
    if (name !== "read_offline") throw new Error(`лишняя команда ${name}`);
    return answer === null ? Promise.reject(new Error("нет")) : Promise.resolve(answer);
  };
  render(() => Read({ text: ru, locale: "ru", url: MATERIAL.url, call }), host);
  return host;
}

test("читалка показывает сохранённый текст абзацами", async () => {
  const host = mountRead({
    kind: "archive",
    title: "Квантование",
    html: "<p>первый</p>",
    text: "первый абзац\n\nвторой абзац",
    path: "/store/objects/ab/abc",
    extracted: true,
  });
  await settled();

  const reading = host.querySelector("[data-reading]");
  assert.deepEqual(
    [...reading.querySelectorAll("p")].map((node) => node.textContent),
    ["первый абзац", "второй абзац"],
  );
  assert.equal(reading.querySelector("[data-source]").getAttribute("href"), MATERIAL.url);
});

test("несохранённый материал не притворяется открытым", async () => {
  const host = mountRead(null);
  await settled();

  assert.match(host.querySelector("[data-empty]").textContent, new RegExp(ru.offline.absent));
});

test("репозиторий и видео отдают путь, а не пустую страницу", async () => {
  const host = mountRead({
    kind: "repo",
    title: "",
    html: "",
    text: "",
    path: "/store/artifacts/ab/abc",
    extracted: false,
  });
  await settled();

  assert.match(host.querySelector("[data-file]").textContent, /artifacts\/ab\/abc/);
});

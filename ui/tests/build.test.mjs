import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import { readdir } from "node:fs/promises";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST, measure, pages, route, scripts } from "../scripts/budget.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const SCREENS = ["/", "/program/", "/topic/", "/exam/", "/review/"];
const ROUTES = ["/", ...["ru", "en"].flatMap((locale) => SCREENS.map((screen) => `/${locale}${screen}`))];

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

async function files() {
  const found = [];
  for (const entry of await readdir(DIST, { recursive: true, withFileTypes: true })) {
    if (entry.isFile()) found.push(path.join(entry.parentPath, entry.name));
  }
  return found;
}

test("каждый маршрут собран отдельной страницей", async () => {
  const built = (await pages()).map((file) => route(file));

  assert.deepEqual(built.sort(), [...ROUTES].sort());
});

test("страница несёт содержимое, а не пустой корень для гидрации", async () => {
  for (const file of await pages()) {
    const html = readFileSync(file, "utf8");
    assert.match(html, /<h1>/, `${route(file)} без заголовка`);
    assert.match(html, /<section /, `${route(file)} без секций`);
  }
});

test("переходы — обычные ссылки, роутера в бандле нет", async () => {
  const forbidden = /ClientRouter|ViewTransitions|astro:transitions|history\.pushState|createRouter/;

  for (const file of await files()) {
    if (!/\.(html|js|mjs)$/.test(file)) continue;
    const source = readFileSync(file, "utf8");
    assert.doesNotMatch(source, forbidden, `${path.relative(DIST, file)} тянет роутер`);
  }

  for (const file of await pages()) {
    if (route(file) === "/") continue;
    assert.match(readFileSync(file, "utf8"), /<a href="\//, `${route(file)} без ссылки назад`);
  }
});

test("страница не ссылается наружу", async () => {
  for (const file of await pages()) {
    const html = readFileSync(file, "utf8");
    assert.doesNotMatch(html, /(?:src|href)=["']https?:\/\//, `${route(file)} тянет ресурс из сети`);
  }
});

test("вес JS уложен в бюджеты", async () => {
  for (const { route: where, bytes, limit, ok } of await measure()) {
    assert.ok(ok, `${where}: ${bytes} байт (gzip) против бюджета ${limit}`);
  }
});

test("гейт краснеет, когда страница выходит за бюджет", async () => {
  const dist = path.join(os.tmpdir(), `tolearn-ui-budget-${process.pid}`);
  await rm(dist, { recursive: true, force: true });
  await mkdir(path.join(dist, "topic"), { recursive: true });
  await writeFile(path.join(dist, "big.js"), randomBytes(64 * 1024).toString("base64"));
  await writeFile(
    path.join(dist, "topic", "index.html"),
    '<html><body><script src="/big.js"></script></body></html>',
  );

  const [measured] = await measure(dist);

  assert.equal(measured.route, "/topic/");
  assert.equal(measured.limit, 30 * 1024);
  assert.equal(measured.ok, false, `${measured.bytes} байт прошли мимо бюджета`);
  await rm(dist, { recursive: true, force: true });
});

test("остров с состоянием собран и подключён к странице темы", async () => {
  const topic = (await measure()).find(({ route: where }) => where === "/ru/topic/");

  assert.ok(topic.files.length > 0, "чанки острова не попали в счёт веса");
  assert.ok(topic.bytes > 4096, `остров весит ${topic.bytes} байт — столько не весит даже Solid`);
});

test("счётчик веса видит и подключённый файл, и встроенный код", () => {
  const html = `<script src="/a.js"></script><script>alert(1)</script>
    <link rel="modulepreload" href="/b.js"><script type="application/json">{}</script>`;
  const { inline, linked } = scripts(html);

  assert.deepEqual(inline, ["alert(1)"]);
  assert.deepEqual(linked, ["/a.js", "/b.js"]);
});

import assert from "node:assert/strict";
import test from "node:test";

import { moment } from "../src/lib/moment.ts";

const at = (text) => Math.floor(new Date(text).getTime() / 1000);

test("момент показывается днём-месяцем и часами-минутами", () => {
  assert.equal(moment(at("2026-07-31T14:05:00")), "31-07 14:05");
});

test("однозначные день, месяц и час дополняются нулём", () => {
  assert.equal(moment(at("2026-03-04T09:07:00")), "04-03 09:07");
});

test("полночь не превращается в двадцать четыре часа", () => {
  assert.equal(moment(at("2026-12-01T00:00:00")), "01-12 00:00");
});

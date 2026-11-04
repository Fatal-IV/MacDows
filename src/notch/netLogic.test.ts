import { test } from "node:test";
import assert from "node:assert/strict";
import { signalBars, securityLabel, errorText } from "./netLogic.ts";

test("signal maps to 0-4 bars", () => {
  const cases: Array<[number, number]> = [[0, 0], [1, 1], [25, 1], [26, 2], [50, 2], [51, 3], [75, 3], [76, 4], [100, 4]];
  for (const [signal, bars] of cases) assert.equal(signalBars(signal), bars, `signal ${signal}`);
});

test("security label is open, secured or enterprise", () => {
  const t = (k: string) => k;
  const base = { ssid: "TestNet", signal: 50, saved: false, connected: false };
  assert.equal(securityLabel({ ...base, secured: false, enterprise: false }, t), "net.open");
  assert.equal(securityLabel({ ...base, secured: true, enterprise: false }, t), "net.secured");
  assert.equal(securityLabel({ ...base, secured: true, enterprise: true }, t), "net.enterprise");
});

test("every backend error key has a text and unknown ones fall back to failed", () => {
  const t = (k: string) => k;
  for (const key of ["not_found", "failed", "timeout", "needs_password", "enterprise", "wrong_password"]) assert.equal(errorText(key, t), `net.err.${key}`);
  assert.equal(errorText("surprise", t), "net.err.failed");
});

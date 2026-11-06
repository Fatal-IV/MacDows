import { test } from "node:test";
import assert from "node:assert/strict";
import { buildLatest } from "./release.mjs";

test("the manifest points the updater at the signed installer of the tag", () => {
  const latest = buildLatest({ version: "1.2.3", notes: "n", signature: "sig", date: "2026-01-01T00:00:00Z" });
  assert.equal(latest.version, "1.2.3");
  assert.equal(latest.platforms["windows-x86_64"].signature, "sig");
  assert.equal(latest.platforms["windows-x86_64"].url, "https://github.com/Fatal-IV/MacDows/releases/download/v1.2.3/MacDows-Setup.exe");
  assert.deepEqual(Object.keys(latest.platforms), ["windows-x86_64"]);
});

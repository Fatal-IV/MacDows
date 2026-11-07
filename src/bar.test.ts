import assert from "node:assert/strict";
import test from "node:test";
import { barKind } from "./bar.ts";

test("the bars are found by their label", () => {
  assert.equal(barKind("notch"), "notch");
  assert.equal(barKind("dock"), "dock");
  assert.equal(barKind("notch-2"), "notch");
  assert.equal(barKind("dock-13"), "dock");
});

test("other windows are not bars", () => {
  for (const l of ["settings", "preview", "tray", "addapp", "docker", "notch-", "notch-x", "dock--2", ""]) assert.equal(barKind(l), null, l);
});

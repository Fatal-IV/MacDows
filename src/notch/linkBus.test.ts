import test from "node:test";
import assert from "node:assert/strict";
import { isRepeat, linkKey, type Link } from "./linkBus.ts";

const l: Link = { kind: "wifi", name: "Home", state: "connected" };

test("the same announcement within the window is a repeat", () => {
  assert.equal(isRepeat({ key: linkKey(l), at: 1000 }, l, 5000), true);
  assert.equal(isRepeat({ key: linkKey(l), at: 1000 }, l, 20000), false);
});

test("another name or state is not a repeat", () => {
  assert.equal(isRepeat({ key: linkKey(l), at: 1000 }, { ...l, state: "disconnected" }, 2000), false);
  assert.equal(isRepeat({ key: linkKey(l), at: 1000 }, { ...l, name: "Work" }, 2000), false);
  assert.equal(isRepeat(null, l, 2000), false);
});

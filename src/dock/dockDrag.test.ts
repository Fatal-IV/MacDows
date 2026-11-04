import { test } from "node:test";
import assert from "node:assert/strict";
import { droppedOutside, sameOrder } from "./dockDrag.ts";

test("dropping inside the dock or just above it keeps the new order", () => {
  assert.equal(droppedOutside(990, 1000, 1080), false);
  assert.equal(droppedOutside(960, 1000, 1080), false); // 40 px above the dock is still close
});

test("dropping far above or below the dock cancels", () => {
  assert.equal(droppedOutside(900, 1000, 1080), true);
  assert.equal(droppedOutside(1200, 1000, 1080), true);
});

test("the same order is recognised, a different one is not", () => {
  assert.equal(sameOrder(["a", "b"], ["a", "b"]), true);
  assert.equal(sameOrder(["a", "b"], ["b", "a"]), false);
  assert.equal(sameOrder(["a"], ["a", "b"]), false);
});

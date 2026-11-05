import { test } from "node:test";
import assert from "node:assert/strict";
import { dockBox, islandBox, pad, sameBox, transitionBox, union } from "./hitBox.ts";

test("union is the smallest box holding both", () => {
  assert.deepEqual(union({ x: 100, y: 0, w: 100, h: 30 }, { x: 50, y: 0, w: 300, h: 80 }), { x: 50, y: 0, w: 300, h: 80 });
  assert.deepEqual(union({ x: 0, y: 0, w: 10, h: 10 }, { x: 20, y: 5, w: 10, h: 10 }), { x: 0, y: 0, w: 30, h: 15 });
});

test("the first shape is sent as it is, later ones together with what was sent before", () => {
  const a = { x: 220, y: 0, w: 140, h: 36 };
  const b = { x: 30, y: 0, w: 520, h: 312 };
  assert.deepEqual(transitionBox(null, a), a);
  assert.deepEqual(transitionBox(a, b), b);
  assert.deepEqual(transitionBox(b, a), b, "shrinking keeps the old shape until the settle step");
});

test("the island box is centered at the top with room for shadow and overshoot", () => {
  assert.deepEqual(islandBox(580, 140, 36, 12), { x: 208, y: 0, w: 164, h: 48 });
  assert.deepEqual(islandBox(580, 520, 312, 12), { x: 18, y: 0, w: 544, h: 324 });
});

test("the dock box has side margins and room above, down to the window bottom", () => {
  assert.deepEqual(dockBox({ x: 100, y: 60, w: 700, h: 60 }, 120, 20, 40), { x: 80, y: 20, w: 740, h: 100 });
  assert.deepEqual(dockBox({ x: 100, y: 10, w: 700, h: 110 }, 120, 20, 40), { x: 80, y: 0, w: 740, h: 120 }, "never above the window");
});

test("pad and sameBox", () => {
  assert.deepEqual(pad({ x: 10, y: 10, w: 20, h: 20 }, { left: 1, right: 2, top: 3, bottom: 4 }), { x: 9, y: 7, w: 23, h: 27 });
  assert.equal(sameBox(null, null), true);
  assert.equal(sameBox({ x: 1, y: 2, w: 3, h: 4 }, { x: 1, y: 2, w: 3, h: 4 }), true);
  assert.equal(sameBox({ x: 1, y: 2, w: 3, h: 4 }, null), false);
});

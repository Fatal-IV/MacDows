import assert from "node:assert/strict";
import test from "node:test";
import { readWatchStyle, watchProgress } from "./watchStyle.ts";

test("popcorn is the default and anything unknown is popcorn", () => {
  assert.equal(readWatchStyle(null), "popcorn");
  assert.equal(readWatchStyle("whatever"), "popcorn");
  assert.equal(readWatchStyle("progress"), "progress");
});

test("the progress follows the position and the time since it was measured", () => {
  assert.equal(watchProgress(30_000, 120_000, 1_000, false, 99_000), 0.25, "paused: the position as it is");
  assert.equal(watchProgress(30_000, 120_000, 1_000, true, 31_000), 0.5, "playing: 30 s more");
});

test("the progress stays between 0 and 1 and is 0 without a length", () => {
  assert.equal(watchProgress(200_000, 120_000, 0, false, 0), 1);
  assert.equal(watchProgress(5_000, 0, 0, true, 10_000), 0);
  assert.equal(watchProgress(-5, 100, 0, false, 0), 0);
});

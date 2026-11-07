import assert from "node:assert/strict";
import test from "node:test";
import { FALLBACK_PALETTE, GRADIENT_FX } from "./gradientFx.ts";

test("there are eight effects and each one is complete", () => {
  assert.equal(GRADIENT_FX.length, 8);
  for (const fx of GRADIENT_FX) {
    assert.ok(fx.look.length > 0);
    assert.ok(Object.keys(fx.from).length > 0);
    assert.ok(Object.keys(fx.to).length > 0);
    assert.ok(Object.keys(fx.still).length > 0);
    assert.ok(fx.move.duration > 0);
  }
});

test("the numbers in Settings are the positions: left to right first, the breathing light last", () => {
  assert.equal(GRADIENT_FX[0].look, "fx0");
  assert.equal(GRADIENT_FX[7].look, "fx3");
});

test("the two diagonals use the turned layer", () => {
  assert.match(GRADIENT_FX[4].look, /diag diag1/);
  assert.match(GRADIENT_FX[5].look, /diag diag2/);
});

test("a source without a cover gets two colors", () => {
  assert.equal(FALLBACK_PALETTE.length, 2);
});

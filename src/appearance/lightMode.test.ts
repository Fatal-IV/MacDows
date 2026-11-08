import assert from "node:assert/strict";
import test from "node:test";
import { isLightTheme } from "./lightMode.ts";

test("the light mode is light, the dark mode is not", () => {
  assert.equal(isLightTheme("light", true), true);
  assert.equal(isLightTheme("light", false), true);
  assert.equal(isLightTheme("dark", false), false);
});

test("following Windows is light only while Windows is light", () => {
  assert.equal(isLightTheme("auto", false), true);
  assert.equal(isLightTheme("auto", true), false);
});

test("the Windows accent mode and unknown values do not count as light", () => {
  assert.equal(isLightTheme("adaptive", false), false);
  assert.equal(isLightTheme("whatever", false), false);
});

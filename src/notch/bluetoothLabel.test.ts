import { test } from "node:test";
import assert from "node:assert/strict";
import { connectedLabel } from "./bluetoothLabel.ts";

const dev = (name: string, connected: boolean) => ({ id: name, name, address: name, connected, battery: null, kind: "other" as const });

test("no devices gives the fallback", () => {
  assert.equal(connectedLabel([], "On"), "On");
  assert.equal(connectedLabel(null, "On"), "On");
});

test("paired but not connected gives the fallback", () => {
  assert.equal(connectedLabel([dev("Mouse", false)], "On"), "On");
});

test("one connected device shows its name", () => {
  assert.equal(connectedLabel([dev("Mouse", false), dev("AirPods Pro", true)], "On"), "AirPods Pro");
});

test("more devices add a count", () => {
  assert.equal(connectedLabel([dev("AirPods Pro", true), dev("Keyboard", true), dev("Mouse", true)], "On"), "AirPods Pro +2");
});

test("a blank name falls back", () => {
  assert.equal(connectedLabel([dev("  ", true)], "On"), "On");
});

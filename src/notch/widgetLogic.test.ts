import { test } from "node:test";
import assert from "node:assert/strict";
import { formatRate, formatTemp, weatherIcon, visibleWidgets, widgetsWidth, needsStats, isLowBattery, moveWidget, trayWidgets, weatherCacheFresh } from "./widgetLogic.ts";

test("formatRate", () => {
  assert.equal(formatRate(0), "0");
  assert.equal(formatRate(900), "0");
  assert.equal(formatRate(1_258_291), "↓ 1,2 MB/s");
  assert.equal(formatRate(52_000), "↓ 51 KB/s");
});

test("formatTemp rounds and converts", () => {
  assert.equal(formatTemp(13.4, "celsius"), "13°");
  assert.equal(formatTemp(0, "fahrenheit"), "32°");
  assert.equal(formatTemp(-0.4, "celsius"), "0°");
});

test("weatherIcon table", () => {
  const table: [number, string][] = [[0, "clear"], [2, "partly"], [3, "cloudy"], [45, "fog"], [63, "rain"], [73, "snow"], [95, "storm"], [999, "cloudy"]];
  for (const [code, icon] of table) assert.equal(weatherIcon(code), icon, String(code));
});

test("visibleWidgets hides battery without a battery and weather that is not ready", () => {
  const w = { left: ["weather", "cpu"], right: ["battery"] };
  assert.deepEqual(visibleWidgets(w, { hasBattery: false, weatherReady: false }), { left: ["cpu"], right: [] });
  assert.deepEqual(visibleWidgets(w, { hasBattery: true, weatherReady: true }), { left: ["weather", "cpu"], right: ["battery"] });
});

test("widgetsWidth counts 54 per widget and 84 for net", () => {
  assert.equal(widgetsWidth({ left: ["weather"], right: ["battery"] }), 108);
  assert.equal(widgetsWidth({ left: ["net", "cpu"], right: [] }), 138);
  assert.equal(widgetsWidth({ left: [], right: [] }), 0);
});

test("needsStats only for cpu, ram, disk, net", () => {
  assert.equal(needsStats({ left: ["weather"], right: ["battery"] }), false);
  assert.equal(needsStats({ left: ["weather"], right: ["disk"] }), true);
});

test("moveWidget", () => {
  const w = { left: ["weather", "cpu", "ram"], right: ["battery"] };
  assert.equal(moveWidget(w, "disk", { side: "left", index: 0 }), null);
  assert.deepEqual(moveWidget(w, "battery", { side: "left", index: 0 }), null);
  assert.deepEqual(moveWidget(w, "cpu", { side: "right", index: 0 }), { left: ["weather", "ram"], right: ["cpu", "battery"] });
  assert.deepEqual(moveWidget(w, "ram", { side: "left", index: 0 }), { left: ["ram", "weather", "cpu"], right: ["battery"] });
  assert.deepEqual(moveWidget(w, "weather", { side: "tray", index: 0 }), { left: ["cpu", "ram"], right: ["battery"] });
});

test("trayWidgets lists the unplaced ones", () => {
  assert.deepEqual(trayWidgets({ left: ["weather"], right: ["battery"] }), ["cpu", "ram", "disk", "net"]);
});

test("isLowBattery", () => {
  assert.equal(isLowBattery(15, 20, false), true);
  assert.equal(isLowBattery(15, 20, true), false);
  assert.equal(isLowBattery(20, 20, false), false);
});

test("weatherCacheFresh", () => {
  assert.equal(weatherCacheFresh(0, 29 * 60_000), true);
  assert.equal(weatherCacheFresh(0, 30 * 60_000), false);
  assert.equal(weatherCacheFresh(1000, 0), false);
});

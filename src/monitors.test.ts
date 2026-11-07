import assert from "node:assert/strict";
import test from "node:test";
import type { MonitorCfg, MonitorView } from "./bridge.ts";
import { monitorCards, setFlag, showSection } from "./monitors.ts";

const view = (key: string, x: number, primary = false, island = false, dock = false): MonitorView => ({ key, name: `name of ${key}`, width: 1920, height: 1080, x, y: 0, primary, island, dock });
const cfg = (key: string, island: boolean, dock: boolean): MonitorCfg => ({ key, name: `saved ${key}`, island, dock });

test("the cards follow the order of the monitors on the desk", () => {
  const cards = monitorCards([view("b", 1920), view("a", 0, true)], []);
  assert.deepEqual(cards.map((c) => c.key), ["a", "b"]);
  assert.equal(cards[0].primary, true);
  assert.equal(cards.every((c) => c.connected), true);
});

test("a saved monitor that is not plugged in comes last and is marked", () => {
  const cards = monitorCards([view("a", 0, true, true, true)], [cfg("gone", true, false)]);
  assert.deepEqual(cards.map((c) => [c.key, c.connected]), [["a", true], ["gone", false]]);
  assert.equal(cards[1].name, "saved gone");
  assert.equal(cards[1].island, true);
  assert.equal(cards[1].dock, false);
});

test("a saved monitor that is plugged in is shown once, with the live flags", () => {
  const cards = monitorCards([view("a", 0, true, true, true)], [cfg("a", true, true)]);
  assert.equal(cards.length, 1);
});

test("switching a flag creates the entry and keeps the other flag as the card shows it", () => {
  const [card] = monitorCards([view("b", 1920, false, false, true)], []);
  assert.deepEqual(setFlag([], card, "island", true), [{ key: "b", name: "name of b", island: true, dock: true }]);
});

test("switching a flag updates the saved entry and leaves the others alone", () => {
  const [a, b] = monitorCards([view("a", 0, true, true, true), view("b", 1920, false, true, true)], []);
  const saved = [cfg("a", true, true), cfg("b", true, true)];
  assert.deepEqual(setFlag(saved, b, "dock", false), [cfg("a", true, true), { key: "b", name: "saved b", island: true, dock: false }]);
  assert.deepEqual(setFlag(saved, a, "island", false)[0], { key: "a", name: "saved a", island: false, dock: true });
});

test("the section is shown only when there is more than one monitor", () => {
  assert.equal(showSection(monitorCards([view("a", 0, true)], [])), false);
  assert.equal(showSection(monitorCards([view("a", 0, true), view("b", 1920)], [])), true);
});

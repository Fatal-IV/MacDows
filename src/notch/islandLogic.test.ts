import { test } from "node:test";
import assert from "node:assert/strict";
import { buildStops, canDisable, clickTarget, mutedApp, restingStop, screenOn, shouldGreet, GREET_EVERY_START } from "./islandLogic.ts";
import { defaultIsland } from "./islandDefaults.ts";
import type { Island, ScreenId } from "../bridge";

const only = (...on: ScreenId[]): Island => {
  const i = defaultIsland();
  for (const id of ["status", "music", "tools", "quick", "notifications", "shelf"] as ScreenId[]) i.screens[id].enabled = on.includes(id);
  return i;
};

test("by default the wheel walks every screen, with the player after the music pill when media exists", () => {
  assert.deepEqual(buildStops(defaultIsland(), true), ["status", "music", "player", "tools", "quick", "history", "shelf"]);
});

test("without media the music pill and the player drop out", () => {
  assert.deepEqual(buildStops(defaultIsland(), false), ["status", "tools", "quick", "history", "shelf"]);
});

test("a switched-off music screen drops its pill and player even with media", () => {
  const i = defaultIsland();
  i.screens.music.enabled = false;
  assert.deepEqual(buildStops(i, true), ["status", "tools", "quick", "history", "shelf"]);
});

test("the order setting decides the walk", () => {
  const i = defaultIsland();
  i.nav.order = ["shelf", "quick", "status", "music", "tools", "notifications"];
  assert.deepEqual(buildStops(i, false), ["shelf", "quick", "status", "tools", "history"]);
});

test("a single enabled screen is the only stop", () => {
  assert.deepEqual(buildStops(only("tools"), true), ["tools"]);
});

test("smart click opens the player while media plays, otherwise the quick settings", () => {
  assert.equal(clickTarget(defaultIsland(), true, true), "player");
  assert.equal(clickTarget(defaultIsland(), true, false), "quick");
  assert.equal(clickTarget(defaultIsland(), false, false), "quick");
});

test("smart click with the music screen off falls to the quick settings", () => {
  const i = defaultIsland();
  i.screens.music.enabled = false;
  assert.equal(clickTarget(i, true, true), "quick");
});

test("a click whose target is off goes to the first other screen, never to a pill", () => {
  assert.equal(clickTarget(only("status", "tools", "shelf"), false, false), "tools");
});

test("when only the status pill is on a click does nothing", () => {
  assert.equal(clickTarget(only("status"), true, true), null);
  assert.equal(clickTarget(only("status", "music"), true, true), "player");
});

test("the player click mode without media uses the quick settings", () => {
  const i = defaultIsland();
  i.nav.click = "player";
  assert.equal(clickTarget(i, false, false), "quick");
  assert.equal(clickTarget(i, true, false), "player");
});

test("the quick click mode opens the quick settings even while media plays", () => {
  const i = defaultIsland();
  i.nav.click = "quick";
  assert.equal(clickTarget(i, true, true), "quick");
});

test("the resting pill is music only while media plays and the music screen is on", () => {
  assert.equal(restingStop(defaultIsland(), true, true), "music");
  assert.equal(restingStop(defaultIsland(), true, false), "status");
  const i = defaultIsland();
  i.screens.music.enabled = false;
  assert.equal(restingStop(i, true, true), "status");
});

test("the last enabled screen cannot be switched off", () => {
  assert.equal(canDisable(defaultIsland(), "music"), true);
  assert.equal(canDisable(only("shelf"), "shelf"), false);
  assert.equal(canDisable(only("shelf", "tools"), "shelf"), true);
});

test("a switched-off screen can always be switched on", () => {
  assert.equal(canDisable(only("shelf"), "tools"), true);
});

test("muted apps match ignoring case", () => {
  const i = defaultIsland();
  i.screens.notifications.muted_apps = ["Slack"];
  assert.equal(mutedApp(i, "slack"), true);
  assert.equal(mutedApp(i, "Teams"), false);
});

test("screenOn reads the enabled flag of a screen", () => {
  assert.equal(screenOn(only("tools"), "tools"), true);
  assert.equal(screenOn(only("tools"), "shelf"), false);
});

test("the island greets on a fresh install and after an update, not on a plain restart", () => {
  assert.equal(shouldGreet(null, "0.2.2"), true, "fresh install");
  assert.equal(shouldGreet("0.2.1", "0.2.2"), true, "updated");
  assert.equal(shouldGreet("0.2.2", "0.2.2"), false, "same version again");
  assert.equal(shouldGreet(null, ""), false, "version unknown");
});

test("the temporary testing switch makes the island greet on every start", () => {
  assert.equal(shouldGreet("0.2.2", "0.2.2", true), true);
  assert.equal(shouldGreet(null, "", true), false, "still not without a version");
});

test("the testing switch is off in a release", { todo: "flip GREET_EVERY_START to false before releasing" }, () => {
  assert.equal(GREET_EVERY_START, false);
});

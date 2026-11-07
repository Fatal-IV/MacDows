import assert from "node:assert/strict";
import test from "node:test";
import { mediaActivity } from "./mediaActivity.ts";

const m = (source_app: string, title: string, artist: string, playback_type = "") => ({ source_app, playback_type, title, artist });

test("Spotify is music", () => {
  assert.equal(mediaActivity(m("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify", "YARA BANDI", "Hidra"), 1), "music");
});

test("Netflix in Edge reports only the site name, no artist and no cover: video", () => {
  assert.equal(mediaActivity(m("MSEdge", "Netflix", ""), null), "video");
});

test("a YouTube video has a wide thumbnail: video", () => {
  assert.equal(mediaActivity(m("MSEdge", "Some video title", "Some Channel"), 16 / 9), "video");
});

test("YouTube Music in a browser has a square cover and an artist: music", () => {
  assert.equal(mediaActivity(m("chrome.exe", "A song", "An artist"), 1), "music");
});

test("a browser tab with an artist and no cover is music", () => {
  assert.equal(mediaActivity(m("MSEdge", "A song", "An artist"), null), "music");
});

test("what Windows reports about the kind wins", () => {
  assert.equal(mediaActivity(m("somewhere.exe", "x", "y", "video"), 1), "video");
  assert.equal(mediaActivity(m("MSEdge", "x", "y", "music"), 16 / 9), "music");
});

test("known video players are video", () => {
  assert.equal(mediaActivity(m("VLC.exe", "film.mkv", ""), null), "video");
  assert.equal(mediaActivity(m("Microsoft.ZuneVideo_8wekyb3d8bbwe!Microsoft.ZuneVideo", "film", ""), null), "video");
});

test("a streaming site named in the title of a browser session is video", () => {
  assert.equal(mediaActivity(m("firefox.exe", "Prime Video", ""), null), "video");
  assert.equal(mediaActivity(m("MSEdge", "Disney+", "x"), 1), "video");
});

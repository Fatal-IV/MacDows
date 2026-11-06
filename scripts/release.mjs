// Builds, signs and publishes a MacDows release: the same way the other IK Software programs do it, from this PC,
// with only what a user or the in-app updater needs on the release page:
//   MacDows-Setup.exe   the installer (new installs, and the updater downloads this very file; it is signed)
//   latest.json         what the updater reads (version, notes, the installer's signature and URL)
//
//   node scripts/release.mjs              build, sign, publish
//   node scripts/release.mjs --skip-build reuse installer/dist from an earlier build
//   node scripts/release.mjs --dry-run    do everything except talking to GitHub (writes installer/dist/latest.json)
//
// Raise the version (package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml), write the CHANGELOG section for it
// and push `main` before running this.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const REPO = "Fatal-IV/MacDows";
const KEY_FILE = join(homedir(), ".macdows-keys", "updater.key");

/** The updater manifest for `version`: one platform, pointing at the signed installer of that release's tag. */
export function buildLatest({ version, notes, signature, date }) {
  return {
    version,
    notes,
    pub_date: date,
    platforms: {
      "windows-x86_64": {
        signature,
        url: `https://github.com/${REPO}/releases/download/v${version}/MacDows-Setup.exe`,
      },
    },
  };
}

const run = (cmd, args, opts = {}) => execFileSync(cmd, args, { cwd: root, stdio: "inherit", ...opts });
const out = (cmd, args) => execFileSync(cmd, args, { cwd: root, encoding: "utf8" }).trim();

function main() {
  const skipBuild = process.argv.includes("--skip-build");
  const dryRun = process.argv.includes("--dry-run");
  const version = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8")).version;
  const tag = `v${version}`;

  const notes = out("node", ["scripts/extract-notes.mjs", version]);
  if (!notes) throw new Error(`CHANGELOG.md has no notes for ${version}.`);

  if (!dryRun) {
    const releases = JSON.parse(out("gh", ["release", "list", "--repo", REPO, "--json", "tagName"]));
    if (releases.some((r) => r.tagName === tag)) throw new Error(`${tag} is already published: raise the version first.`);
    run("git", ["fetch", "origin", "main"], { stdio: "ignore" });
    if (out("git", ["rev-parse", "HEAD"]) !== out("git", ["rev-parse", "origin/main"])) {
      throw new Error("HEAD is not what origin/main has: push it first (git push origin <branch>:main).");
    }
  }

  if (!skipBuild) {
    if (!existsSync(KEY_FILE)) throw new Error(`Signing key not found: ${KEY_FILE}`);
    // The key's CONTENT goes into the variable (not its path), with an empty password.
    const env = { ...process.env, TAURI_SIGNING_PRIVATE_KEY: readFileSync(KEY_FILE, "utf8"), TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "" };
    run("node", ["scripts/build-installer.mjs"], { env });
  }

  const dist = join(root, "installer", "dist");
  const setup = join(dist, "MacDows-Setup.exe");
  const sigFile = `${setup}.sig`;
  if (!existsSync(setup) || !existsSync(sigFile)) throw new Error("installer/dist has no signed MacDows-Setup.exe: build first.");
  const latest = buildLatest({ version, notes, signature: readFileSync(sigFile, "utf8").trim(), date: new Date().toISOString().replace(/\.\d+Z$/, "Z") });
  const latestFile = join(dist, "latest.json");
  writeFileSync(latestFile, JSON.stringify(latest, null, 2));
  const notesFile = join(dist, "notes.md");
  writeFileSync(notesFile, notes);

  if (dryRun) {
    console.log(`[dry run] would publish ${tag} with MacDows-Setup.exe and latest.json`);
    return;
  }
  run("gh", ["release", "create", tag, setup, latestFile, "--repo", REPO, "--target", "main", "--title", `MacDows ${version}`, "--notes-file", notesFile]);
  console.log(`\n${tag} published: https://github.com/${REPO}/releases/tag/${tag}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (err) {
    console.error(String(err.message ?? err));
    process.exit(1);
  }
}

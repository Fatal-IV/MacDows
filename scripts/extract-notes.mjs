// Prints the CHANGELOG.md section for the given version (e.g. 0.1.0).
import { readFileSync } from "node:fs";
const version = process.argv[2];
const lines = readFileSync("CHANGELOG.md", "utf8").split(/\r?\n/);
const start = lines.findIndex((l) => l.startsWith(`## [${version}]`));
if (start < 0) {
  console.error(`No changelog section for ${version}`);
  process.exit(1);
}
let end = lines.findIndex((l, i) => i > start && l.startsWith("## ["));
if (end < 0) end = lines.length;
console.log(lines.slice(start + 1, end).join("\n").trim());

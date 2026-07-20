import { readFileSync, writeFileSync } from "fs";
import { execSync } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");

const version = process.argv[2];

if (!version) {
  console.error("Usage: node scripts/bump-version.mjs <version>");
  console.error("Example: node scripts/bump-version.mjs 1.2.0");
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error("Error: Version must be in semver format (e.g., 1.2.0)");
  process.exit(1);
}

const files = [
  {
    path: resolve(root, "package.json"),
    replace: (content) =>
      content.replace(/"version":\s*"[^"]*"/, `"version": "${version}"`),
  },
  {
    path: resolve(root, "src-tauri/tauri.conf.json"),
    replace: (content) =>
      content.replace(/"version":\s*"[^"]*"/, `"version": "${version}"`),
  },
  {
    path: resolve(root, "src-tauri/Cargo.toml"),
    replace: (content) =>
      content.replace(/^version = "[^"]*"/m, `version = "${version}"`),
  },
];

for (const file of files) {
  const content = readFileSync(file.path, "utf-8");
  writeFileSync(file.path, file.replace(content), "utf-8");
}

console.log(`Updated version to ${version} in:`);
console.log("  - package.json");
console.log("  - src-tauri/tauri.conf.json");
console.log("  - src-tauri/Cargo.toml");

execSync("git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml", { cwd: root, stdio: "inherit" });
execSync(`git commit -m "chore: bump version to ${version}"`, { cwd: root, stdio: "inherit" });
execSync(`git tag v${version}`, { cwd: root, stdio: "inherit" });

console.log("");
console.log(`Committed and tagged v${version}`);
console.log("Run 'git push && git push --tags' to publish.");

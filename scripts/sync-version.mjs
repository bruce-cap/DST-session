import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const envPath = path.join(root, ".env");
const packageJsonPath = path.join(root, "package.json");
const tauriConfigPath = path.join(root, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(root, "src-tauri", "Cargo.toml");
const cargoLockPath = path.join(root, "src-tauri", "Cargo.lock");

const appName = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")).name;

function parseEnv(content) {
  const env = {};

  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;

    const match = line.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/);
    if (!match) continue;

    let [, key, value] = match;
    value = value.trim();

    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    env[key] = value;
  }

  return env;
}

function assertSemver(version) {
  const semverPattern =
    /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

  if (!semverPattern.test(version)) {
    throw new Error(
      `APP_VERSION must be a semver value like 0.2.0, got: ${version}`,
    );
  }
}

function writeIfChanged(filePath, nextContent) {
  const previousContent = fs.existsSync(filePath)
    ? fs.readFileSync(filePath, "utf8")
    : "";

  if (previousContent === nextContent) return false;

  fs.writeFileSync(filePath, nextContent);
  return true;
}

function updateJsonVersion(filePath, version) {
  const content = fs.readFileSync(filePath, "utf8");
  const nextContent = content.replace(
    /(^\s*"version"\s*:\s*")[^"]+(")/m,
    `$1${version}$2`,
  );

  if (nextContent === content && !content.includes(`"version": "${version}"`)) {
    throw new Error(`Could not find top-level version in ${filePath}`);
  }

  return writeIfChanged(filePath, nextContent);
}

function updateCargoTomlVersion(filePath, version) {
  const content = fs.readFileSync(filePath, "utf8");
  const nextContent = content.replace(
    /(^\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m,
    `$1${version}$2`,
  );

  if (nextContent === content && !content.includes(`version = "${version}"`)) {
    throw new Error(`Could not find [package].version in ${filePath}`);
  }

  return writeIfChanged(filePath, nextContent);
}

function updateCargoLockVersion(filePath, packageName, version) {
  if (!fs.existsSync(filePath)) return false;

  const content = fs.readFileSync(filePath, "utf8");
  const escapedPackageName = packageName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const packageEntryPattern = new RegExp(
    `(^\\[\\[package\\]\\]\\r?\\nname = "${escapedPackageName}"\\r?\\nversion = ")[^"]+(")`,
    "m",
  );
  const nextContent = content.replace(packageEntryPattern, `$1${version}$2`);

  if (nextContent === content && !content.includes(`name = "${packageName}"`)) {
    throw new Error(`Could not find ${packageName} in ${filePath}`);
  }

  return writeIfChanged(filePath, nextContent);
}

if (!fs.existsSync(envPath)) {
  throw new Error("Missing .env. Copy .env.example to .env and set APP_VERSION.");
}

const env = parseEnv(fs.readFileSync(envPath, "utf8"));
const version = env.APP_VERSION;

if (!version) {
  throw new Error("Missing APP_VERSION in .env.");
}

assertSemver(version);

const changedFiles = [];

if (updateJsonVersion(packageJsonPath, version)) changedFiles.push("package.json");
if (updateJsonVersion(tauriConfigPath, version))
  changedFiles.push("src-tauri/tauri.conf.json");
if (updateCargoTomlVersion(cargoTomlPath, version))
  changedFiles.push("src-tauri/Cargo.toml");
if (updateCargoLockVersion(cargoLockPath, appName, version))
  changedFiles.push("src-tauri/Cargo.lock");

if (changedFiles.length > 0) {
  console.log(`Synced app version to ${version}: ${changedFiles.join(", ")}`);
} else {
  console.log(`App version already synced: ${version}`);
}

// Code copied from [Rome](https://github.com/rome/tools/blob/main/npm/rome/scripts/generate-packages.mjs)

import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";

const PACKAGE_BIN_NAME = "turbo-remote-cache-rs";
const PACKAGE_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const NPM_PACKAGES_ROOT = resolve(PACKAGE_ROOT, "npm");
const REPO_ROOT = PACKAGE_ROOT;
const MANIFEST_PATH = resolve(PACKAGE_ROOT, "package.json");

console.log(`PACKAGE_ROOT: ${PACKAGE_ROOT}`);
const rootManifest = JSON.parse(readFileSync(MANIFEST_PATH).toString("utf-8"));

const LIBC_MAPPING = {
  gnu: "glibc",
  musl: "musl",
} as const;

function generateNativePackage(target) {
  const packageName = `@${PACKAGE_BIN_NAME}/${target}`;
  const packageRoot = resolve(
    NPM_PACKAGES_ROOT,
    `${PACKAGE_BIN_NAME}-${target}`
  );

  // Remove the directory just in case it already exists (it's autogenerated
  // so there shouldn't be anything important there anyway)
  rmSync(packageRoot, { recursive: true, force: true });

  // Create the package directory
  console.log(`Create directory ${packageRoot}`);
  mkdirSync(packageRoot, { recursive: true });

  // Generate the package.json manifest
  const { version, author, license, homepage, bugs, repository } = rootManifest;

  const triple = target.split("-");
  const platform = triple[0];
  const arch = triple[1];
  const libc = triple[2] && { libc: [LIBC_MAPPING[triple[2]]] };
  const manifest = {
    name: packageName,
    version,
    author,
    license,
    homepage,
    bugs,
    repository,
    os: [platform],
    cpu: [arch],
    ...libc,
  };

  const manifestPath = resolve(packageRoot, "package.json");
  console.log(`Create manifest ${manifestPath}`);
  writeFileSync(manifestPath, JSON.stringify(manifest));

  // Copy the binary
  const ext = platform === "win32" ? ".exe" : "";

  const binSource = resolve(REPO_ROOT, `${PACKAGE_BIN_NAME}-${target}${ext}`);
  const binTarget = resolve(packageRoot, `${PACKAGE_BIN_NAME}${ext}`);

  console.log(`Copy binary ${binSource}`);
  copyFileSync(binSource, binTarget);
  chmodSync(binTarget, 0o755);
}

function writeManifest() {
  // const manifestPath = resolve(PACKAGES_ROOT, PACKAGE_BIN_NAME, "package.json");
  const manifestPath = resolve(MANIFEST_PATH);

  const manifestData = JSON.parse(readFileSync(manifestPath).toString("utf-8"));

  const nativePackages = TARGETS.map((target) => [
    `@${PACKAGE_BIN_NAME}/${target}`,
    rootManifest.version,
  ]);

  manifestData["version"] = rootManifest.version;
  manifestData["optionalDependencies"] = Object.fromEntries(nativePackages);

  console.log(`Update manifest ${manifestPath}`);
  const content = JSON.stringify(manifestData);
  writeFileSync(manifestPath, content);
}

// NOTE: Must update bin/turbo-remote-cache-rs
const TARGETS = [
  "win32-x64",
  "win32-arm64",
  "linux-x64-gnu",
  "linux-arm64-gnu",
  "linux-x64-musl",
  "linux-arm64-musl",
  "darwin-x64",
  "darwin-arm64",
];

for (const target of TARGETS) {
  generateNativePackage(target);
}

writeManifest();

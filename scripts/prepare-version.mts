#!/usr/bin/env -S npx tsx

import fs from "fs";
import cp from "child_process";
import cmd from "cmd-ts";
import assert from "assert";

const CARGO_TOML_PATH = new URL("../Cargo.toml", import.meta.url).pathname;

const command = cmd.command({
  name: "prepare-version",
  description: "Prepare a new package version",
  args: {},
  async handler({}) {
    updateCargoToml(await getPackageVersion());
    exec("cargo build --release");
  },
});

cmd
  .run(cmd.binary(command), process.argv)
  .then(() => {
    console.log("Done!", process.exitCode);
    process.exitCode = 0;
  })
  .catch((err) => {
    console.error(err);
    process.exitCode = process.exitCode || 1;
  });

//////////////////////
// Helper functions //
//////////////////////

async function getPackageVersion() {
  const pkgJson = await fs.promises.readFile(
    new URL("../package.json", import.meta.url),
    "utf8"
  );
  const version = JSON.parse(pkgJson).version;
  assert(version, "package.json version is not set");
  return version;
}

function updateCargoToml(nextVersion: string) {
  const cargoToml = fs.readFileSync(CARGO_TOML_PATH, "utf8");
  // replace old  version with new version
  const pattern = /(\[package\][\s\S]*?version\s*=\s*)("[^"]*")/;
  console.log(cargoToml.match(pattern));
  const newToml = cargoToml.replace(
    pattern,
    (_, p1) => `${p1}"${nextVersion}"`
  );
  if (newToml === cargoToml) {
    console.error("Cargo.toml didn't change, error!");
    process.exitCode = 1;
    return;
  }

  fs.writeFileSync(CARGO_TOML_PATH, newToml, "utf8");
  console.log("Cargo.toml updated to version", nextVersion);
  return nextVersion;
}

function exec(command: string, env: Record<string, string> = {}) {
  console.log(`$ ${command}`);
  return cp.execSync(command, {
    cwd: new URL("..", import.meta.url),
    stdio: "inherit",
    env: { ...process.env, ...env },
  });
}

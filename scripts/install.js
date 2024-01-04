#!/usr/bin/env node
/* eslint-disable */

const { execFileSync } = require("child_process");
const { detectTarget, exePath } = require("./platform");
const { normalize } = require("path");
const { mkdirSync, existsSync, cpSync, linkSync, chmodSync, symlinkSync} = require("node:fs");

const target = process.env.INPUT_TARGET_NAME || detectTarget();


const sourceExePath = exePath(target);
const pathFolder = normalize(__dirname + `/../paths/${target}`);

if (existsSync(pathFolder)) {
	console.log("Already installed");
	process.exit(0);
}

mkdirSync(pathFolder, { recursive: true });

const commands = execFileSync(sourceExePath, ["internal-list"], { encoding: "utf-8" }).split("\n").filter((line) => line !== "");

let installer
if (target.includes("windows")) {
	installer = cpSync
} else {
	installer = (source, dst) => {
		symlinkSync(source, dst);
		chmodSync(dst, 0o755);
	}
}

for (const command of commands) {
	installer(sourceExePath, `${pathFolder}/${command}`)
}

execFileSync(sourceExePath, ["gh-add-path", "--", pathFolder], { stdio: "inherit" })

#!/usr/bin/env node
/* eslint-disable */

const { execFileSync } = require("child_process");
const { detectTarget, exePath } = require("./platform");
const { normalize } = require("path");
const { mkdirSync, existsSync, cpSync, linkSync, chmodSync, symlinkSync} = require("node:fs");

function installForTarget(target) {
	console.log(`installing for ${target}`);

	const sourceExePath = exePath(target);
	const pathFolder = normalize(__dirname + `/../paths/${target}`);

	if (existsSync(pathFolder)) {
		console.log("Already installed");
		process.exit(0);
	}

	mkdirSync(pathFolder, {recursive: true});

	const commands = execFileSync(sourceExePath, ["internal-list"], {encoding: "utf-8"}).split("\n").filter((line) => line !== "");
	commands.push("something-releaser");

	let installer
	if (target.includes("windows")) {
		console.log(`using copy to create multicall binaries`);
		installer = cpSync
	} else {
		console.log(`using symlink to create multicall binaries`);
		installer = (source, dst) => {
			symlinkSync(source, dst);
			chmodSync(dst, 0o755);
		}
	}

	for (const command of commands) {
		const path = `${pathFolder}/${command}`;
		installer(sourceExePath, path)
		console.log(`::debug::installed to ${path}`);
	}

	execFileSync(sourceExePath, ["gh-add-path", "--", pathFolder], {stdio: "inherit"})
}

function toBoolean(value) {
	if (typeof value !== "string") return !!value;
	switch (value.toLowerCase()) {
		case "true":
		case "yes":
		case "1":
			return true;
		case "false":
		case "no":
		case "0":
		case null:
			return false;
		default:
			return Boolean(value);
	}
}

if (require.main === module) {
	const target = process.env["INPUT_TARGET"] || detectTarget();
	const build = toBoolean(process.env["INPUT_BUILD-ON-INSTALL"]);
	if (build) {
		require("./build").buildForTarget(target);
	}
	installForTarget(target);
}

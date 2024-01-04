/* eslint-disable */

const { normalize } = require("path");

function detectTarget() {
	switch (`${process.platform}-${process.arch}`) {
		case "win32-x64": return "x86_64-pc-windows-msvc";
		case "win32-arm64": return "aarch64-pc-windows-msvc";
		case "linux-x64": return "x86_64-unknown-linux-gnu";
		case "linux-arm64": return "aarch64-unknown-linux-gnu";
		case "darwin-x64": return "x86_64-apple-darwin";
		case "darwin-arm64": return "aarch64-apple-darwin";
		default: throw new Error(`Unknown / Unsupported architecture or platform: ${process.platform} ${process.arch}`);
	}
}

function targetExt(target) {
	return target.includes("windows") ? ".exe" : "";
}

function exeFolder(target) {
	return normalize(__dirname + `/../build/${target}/something-releaser${targetExt(target)}`);
}

function exeName(target) {
	return `something-releaser${targetExt(target)}`;
}

function exePath(target) {
	return exeFolder(target) + '/' + exeName(target);
}

module.exports = {
	detectTarget,
	targetExt,
	exeFolder,
	exeName,
	exePath,
}

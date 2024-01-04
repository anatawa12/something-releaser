#!/usr/bin/env node
/* eslint-disable */

const { execFileSync } = require("child_process");
const { mkdirSync, cpSync } = require("fs");
const { detectTarget, targetExt, exeFolder, exePath} = require("./platform");

const target = process.argv[2] || detectTarget();

execFileSync("cargo", ["build", "--release", "--target", target], { stdio: "inherit" });

const exec_dir = `target/${target}/release`;
const exec_ext = targetExt(target);

mkdirSync(exeFolder(target), { recursive: true });
cpSync(`${exec_dir}/something-releaser${exec_ext}`, exePath(target));


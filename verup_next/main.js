const { spawn } = require('child_process');
const { somethingReleaser, readInput } = require('../lib.js');

let newVersion = readInput("new_version", "no new_version is specified");
let versionChangers = readInput("version_changers", "no version changer is specified").split(',');

let args = [];
args.push("--github-actions-mode");
args.push("update-version-next");
args.push(newVersion);
for (let versionChanger of versionChangers)
    args.push("--version-changers", versionChanger);

const child = spawn(somethingReleaser, args, {
    stdio: ["ignore", "inherit", "inherit"]
})

child.on("exit", (code, signal) => {
    if (code != null) process.exit(code);
    console.log(`fails with ${signal}`);
    process.exit(-1);
})

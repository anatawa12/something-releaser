import { join } from 'path';
import { spawn } from 'child_process';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * @param name {string}
 * @param [error] {string | undefined}
 * @return {string|null}
 */
function readInput(name, error) {
    let input = process.env["INPUT_" + name.split(' ').join('_').toUpperCase()];
    if (!input || input.trim() === "") {
        if (error) throw new Error(error);
        return null;
    }
    return input;
}

let newVersion = readInput("new_version", "no new_version is specified");
let versionChangers = readInput("version_changers", "no version changer is specified").split(',');

let args = [];
args.push("--github-actions-mode");
args.push("update-version-next");
args.push(newVersion);
for (let versionChanger of versionChangers)
    args.push("--version-changers", versionChanger);

const child = spawn(join(__dirname, "..", "something-releaser"), args, {
    stdio: ["ignore", "inherit", "inherit"]
})

child.on("exit", (code, signal) => {
    if (code != null) process.exit(code);
    console.log(`fails with ${signal}`);
    process.exit(-1);
})

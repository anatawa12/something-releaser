import { join } from 'path';
import { spawn } from 'child_process';
import { file as tempFile } from '../temp.mjs';
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

let changelog = readInput("changelog", "no changelog is specified");
let repository = readInput("repository", "no repository is specified");
let versionChangers = readInput("version_changers", "no version changer is specified").split(',');
let versionChangelogHtml = readInput("version_changelog_html") || tempFile("CHANGELOG.html");
let versionChangelogMarkdown = readInput("version_changelog_markdown") || tempFile("CHANGELOG.md");

let args = [];
args.push("update-version");
args.push("--github-actions-mode");
args.push("--changelog", changelog);
args.push("--repo", repository);
for (let versionChanger of versionChangers)
    args.push("--version-changers", versionChanger);
args.push("--version-release-note-html", versionChangelogHtml);
args.push("--version-release-note-markdown", versionChangelogMarkdown);

spawn(join(__dirname, "..", "something-releaser"), args, {
    stdio: ["ignore", "inherit", "inherit"]
})

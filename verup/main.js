import { spawn } from 'child_process';
import { somethingReleaser, readInput, tempFile, getEnv } from '../lib.mjs';

let changelog = readInput("changelog", "no changelog is specified");
let repository = readInput("repository", "no repository is specified")
    || (getEnv('GITHUB_SERVER_URL', 'no GITHUB_SERVER_URL') + '/'
        + getEnv('GITHUB_REPOSITORY', 'no GITHUB_REPOSITORY'));
let versionChangers = readInput("version_changers", "no version changer is specified").split(',');
let versionChangelogHtml = readInput("changelog_html") || tempFile("CHANGELOG.html");
let versionChangelogMarkdown = readInput("changelog_markdown") || tempFile("CHANGELOG.md");

let args = [];
args.push("update-version");
args.push("--github-actions-mode");
args.push("--changelog", changelog);
args.push("--repo", repository);
for (let versionChanger of versionChangers)
    args.push("--version-changers", versionChanger);
args.push("--version-release-note-html", versionChangelogHtml);
args.push("--version-release-note-markdown", versionChangelogMarkdown);

console.log(`::set-output name=changelog_html::${versionChangelogHtml}`);
console.log(`::set-output name=changelog_markdown::${versionChangelogMarkdown}`);

const child = spawn(somethingReleaser, args, {
    stdio: ["ignore", "inherit", "inherit"]
})

child.on("exit", (code, signal) => {
    if (code != null) process.exit(code);
    console.log(`fails with ${signal}`);
    process.exit(-1);
})

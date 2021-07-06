const { spawn } = require('child_process');
const { somethingReleaser, readInput } = require("../lib.js");

let publishers = readInput("publishers", "no publishers is specified").split(',');
let changelogHtml = readInput("changelog_html", "changelog_html is required");
let changelogMarkdown = readInput("changelog_markdown", "changelog_markdown is required");
let versionName = readInput("version_name", "version_name is required");
let dry_run = readInput("dry_run", "version_name is required");

let args = [];
args.push("publish");
args.push("--github-actions-mode");
for (let publisher of publishers)
    args.push("--publishers", publisher);
args.push("--release-note-html", changelogHtml);
args.push("--release-note-markdown", changelogMarkdown);
args.push("--version", versionName);
if (dry_run.toLowerCase() === 'true') {
    args.push("--dry-run");
}

spawn(somethingReleaser, args, {
    stdio: ["ignore", "inherit", "inherit"]
})

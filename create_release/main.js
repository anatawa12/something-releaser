const {readInput, request, getEnv} = require("../lib.js");
const {readFileSync} = require('fs');

let api = getEnv("GITHUB_API_URL", "no GITHUB_API_URL env var");
let repo = getEnv("GITHUB_REPOSITORY", "no GITHUB_REPOSITORY env var");

let changelogMarkdown = readInput("changelog_markdown", "changelog_markdown is required");
let versionName = readInput("version_name", "version_name is required");
let token = readInput("token", "no token is specified");

let url = `${api}/users/repos/${repo}/releases`;

request(url, {
    method: 'POST',
    headers: {
        authorization: `Bearer ${token}`,
        accept: "application/vnd.github.v3+json",
        'Content-Type': 'application/json',
        'User-Agent': `Node/${process.version.slice(1)} something-releaser+create_release/1`,
    },
    body: JSON.stringify({
        tag_name: versionName,
        target_commitish: versionName,
        name: versionName,
        body: readFileSync(changelogMarkdown),
    }),
}, (res) => {
    if (res.statusCode !== 200) {
        throw new Error(`unsuccessful status code from response: ${url}: ${res.statusCode}`);
    }
    res.setEncoding("utf8");
    let rawData = '';
    res.on('data', (chunk) => { rawData += chunk; });
    res.on('end', () => {
        try {
            gotUserInfo(JSON.parse(rawData));
        } catch (e) {
            console.error(e.message);
        }
    });
})

const {spawnSync} = require('child_process');
const {request, readInput} = require("../lib.js");

function error(message) {
    throw new Error(message)
}

let api = process.env.GITHUB_API_URL || error("no GITHUB_API_URL env var");
let user = readInput("user", "no user is specified");
let token = readInput("token", "no token is specified");
let url = `${api}/users/${encodeURIComponent(user)}`;

request(url, {
    headers: {
        authorization: `Bearer ${token}`,
        'User-Agent': `Node/${process.version.slice(1)} something-releaser+set_user/1`,
    },
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

function gotUserInfo(json) {
    if (json.type === "Organization")
        throw new Error("You can't commit as a Organization")
    /** @type {string} */
    const login = json.login;
    /** @type {number} */
    const id = json.id;
    const mail = `${id}+${login}@users.noreply.github.com`

    spawnSync("git", ['config', '--global', 'user.name', login]);
    spawnSync("git", ['config', '--global', 'user.email', mail]);
}

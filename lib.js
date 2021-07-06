'use strict';
const fs = require('fs');
const {join} = require('path');
const crypto = require("crypto");
const os = require("os");
const http = require("http");
const https = require("https");

const uniqueString = () => crypto.randomBytes(16).toString('hex');

const tempDir = fs.realpathSync(os.tmpdir());

const getPath = (prefix = '') => join(tempDir, prefix + uniqueString());

exports = module.exports = {};

/**
 * @param name {string}
 * @param [error] {string | undefined}
 * @return {string|null}
 */
exports.readInput = function(name, error) {
    return getEnv("INPUT_" + name.split(' ').join('_').toUpperCase(), error);
}

/**
 * @param name {string}
 * @param [error] {string | undefined}
 * @return {string|null}
 */
exports.getEnv = function(name, error) {
    let input = process.env[name];
    if (!input || input.trim() === "") {
        if (error) throw new Error(error);
        return null;
    }
    return input;
}

/**
 * @param name {string}
 * @return {string}
 */
exports.tempFile = name => {
    if (name) {
        return join(tempDirectory(), name);
    } else {
        return getPath();
    }
};

/**
 * @param prefix {string}
 * @return {string}
 */
exports.tempDirectory = (prefix = '') => {
    const directory = getPath(prefix);
    fs.mkdirSync(directory);
    return directory;
};

/**
 * @callback RequestCallback
 * @param res {IncomingMessage}
 * @return {void}
 */

/**
 * @param url {string|URL}
 * @param options {http.RequestOptions}
 * @param callback {RequestCallback}
 */
exports.request = (url, options, callback) => {
    if (!(url instanceof URL))
        url = new URL(url);
    switch (url.protocol) {
        case "http:":
            return http.get(url, options, callback)
        case "https:":
            return https.get(url, options, callback)
        default:
            throw new Error(`unsupported protocol: ${url.protocol}`)
    }
}

exports.somethingReleaser = join(__dirname, "something-releaser")

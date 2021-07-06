'use strict';
import fs from 'fs';
import path, {dirname} from 'path';
import crypto from "crypto";
import os from "os";
import http from "http";
import https from "https";
import {fileURLToPath} from "url";

const uniqueString = () => crypto.randomBytes(16).toString('hex');

const tempDir = fs.realpathSync(os.tmpdir());

const getPath = (prefix = '') => path.join(tempDir, prefix + uniqueString());

/**
 * @param name {string}
 * @param [error] {string | undefined}
 * @return {string|null}
 */
export function readInput(name, error) {
    return getEnv("INPUT_" + name.split(' ').join('_').toUpperCase(), error);
}

/**
 * @param name {string}
 * @param [error] {string | undefined}
 * @return {string|null}
 */
export function getEnv(name, error) {
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
export const tempFile = name => {
    if (name) {
        return path.join(tempDirectory(), name);
    } else {
        return getPath();
    }
};

/**
 * @param prefix {string}
 * @return {string}
 */
export const tempDirectory = (prefix = '') => {
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
export const request = (url, options, callback) => {
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

export const somethingReleaser = path.join(dirname(fileURLToPath(import.meta.url)), "something-releaser")

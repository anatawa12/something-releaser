'use strict';
import fs from 'fs';
import path from 'path';
import crypto from "crypto";
import os from "os";

const uniqueString = () => crypto.randomBytes(16).toString('hex');

const tempDir = fs.realpathSync(os.tmpdir());

const getPath = (prefix = '') => path.join(tempDir, prefix + uniqueString());

/**
 * @param name {string}
 * @return {string}
 */
export const file = name => {
    if (name) {
        return path.join(directory(), name);
    } else {
        return getPath();
    }
};

/**
 * @param prefix {string}
 * @return {string}
 */
export const directory = (prefix = '') => {
    const directory = getPath(prefix);
    fs.mkdirSync(directory);
    return directory;
};

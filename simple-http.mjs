'use strict';
import http from 'http';
import https from 'https';

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

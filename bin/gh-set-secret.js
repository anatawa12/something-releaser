#!/usr/bin/env node
/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {main} = require('../dist/index')

main('gh-set-secret', ...process.argv.slice(2))

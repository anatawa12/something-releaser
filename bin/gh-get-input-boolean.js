#!/usr/bin/env node
/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {main} = require('../dist/index')

main('gh-get-input-boolean', ...process.argv.slice(2))

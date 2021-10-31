#!/usr/bin/env node
/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {main} = require('../dist/index')

main('gh-export-variable', ...process.argv.slice(2))

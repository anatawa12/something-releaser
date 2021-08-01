#!/usr/bin/env node
/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {main} = require('../dist/index')

main('prepare-gradle-plugin-portal', ...process.argv.slice(2))

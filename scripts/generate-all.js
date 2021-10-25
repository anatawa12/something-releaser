/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {mkdirSync} = require('fs')
const {join} = require('path')

mkdirSync(join(__dirname, "..", "src", "generated"), {recursive: true})

require("./generate-sequence-ts")
require("./generate-bins")
require("./generate-env-ts-json")

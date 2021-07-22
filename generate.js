const yaml = require('js-yaml')
const {readFileSync, writeFileSync, mkdirSync} = require('fs')
const {join} = require('path')
const { compile } = require('json-schema-to-typescript')

// compile from file

// or, compile a JS object
let mySchema = yaml.load(
  readFileSync(join(__dirname, "schema.yml"),
    {encoding: "utf8"}));

mkdirSync(join(__dirname, "src", "generated"), {recursive: true})

compile(mySchema, 'schema.yml')
  .then(ts =>
    writeFileSync(join(__dirname, "src", "generated", "yaml.d.ts"), ts)
  )

writeFileSync(join(__dirname, "src", "generated", "schema.json"), 
  JSON.stringify(mySchema))

/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */
const {promises: fs} = require('fs')
const {join} = require('path')
const {load: loadYaml} = require('js-yaml')
const {compile} = require('json-schema-to-typescript')

const envYml = join(__dirname, '..', 'src', 'env.yml')
const envJson = join(__dirname, '..', 'src', 'generated', 'env.json')
const envTs = join(__dirname, '..', 'src', 'generated', 'env.ts')

;(async () => {
  const yaml = loadYaml(await fs.readFile(envYml, { encoding: 'utf-8' }));
  await fs.writeFile(envJson, JSON.stringify(yaml));
  await fs.writeFile(envTs, await compile(yaml, "env.yml"));
})()

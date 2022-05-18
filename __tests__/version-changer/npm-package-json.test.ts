import {promises as fs} from 'fs'
import os from 'os'
import path from 'path'
import {expect, it} from '@jest/globals'
import {NpmPackageJson} from '../../src/version-changer/npm-package-json'
import {creator} from './util'

const create = creator('npm-package-json')

const packageJsonGen = (version: string): string => `{
  "name": "test",
  "version": "${version}",
  "description": "",
  "main": "index.js",
  "scripts": {
    "test": "echo \\"Error: no test specified\\" && exit 1"
  },
  "author": "",
  "license": "ISC"
}
`

const packageLockJsonGen = (version: string): string => `{
  "name": "test",
  "version": "${version}",
  "lockfileVersion": 1
}
`

it("default save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("package.json", packageJsonGen("1.0.0-SNAPSHOT"))
  await fs.writeFile("package-lock.json", packageLockJsonGen("1.0.0-SNAPSHOT"))
  const desc = NpmPackageJson.createFromDesc(create())
  await expect(desc.loadVersion())
    .resolves
    .toBe("1.0.0-SNAPSHOT")
  await desc.setVersion("1")
  await expect(fs.readFile("package.json", {encoding: 'utf8'}))
    .resolves
    .toBe(packageJsonGen("1"))
  await expect(fs.readFile("package-lock.json", {encoding: 'utf8'}))
    .resolves
    .toBe(packageLockJsonGen("1"))
})

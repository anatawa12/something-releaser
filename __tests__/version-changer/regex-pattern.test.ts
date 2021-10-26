import {promises as fs} from 'fs'
import os from 'os'
import path from 'path'
import {expect, it} from '@jest/globals'
import { Version } from '../../src/utils'
import {RegexPattern} from '../../src/version-changer/regex-pattern'
import {creator} from './util'

const create = creator('regex-pattern')

it("invalid descriptions", async () => {
  expect(() => RegexPattern.createFromDesc(create()))
    .toThrow(`regex-pattern requires both pattern and path`)
  expect(() => RegexPattern.createFromDesc(create("", "path")))
    .toThrow(`regex-pattern requires both pattern and path`)
  expect(() => RegexPattern.createFromDesc(create("", "")))
    .toThrow(`regex-pattern requires both pattern and path`)
  expect(() => RegexPattern.createFromDesc(create("pattern")))
    .toThrow(`regex-pattern requires both pattern and path`)
})

it("custom prop save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("test.txt", "version = \"1.0.0-SNAPSHOT\"\n" +
    "version = \"0.1.0-SNAPSHOT\"\n")
  const desc = RegexPattern.createFromDesc(create("version = \"$1\"", "test.txt"))
  await expect(desc.loadVersion())
    .resolves
    .toEqual(new Version(1, 0, 0, ['snapshot']))
  await desc.setVersion(new Version(1))
  await expect(fs.readFile("test.txt", {encoding: 'utf8'}))
    .resolves
    .toBe("version = \"1\"\n" +
      "version = \"0.1.0-SNAPSHOT\"\n")
})

import {promises as fs} from 'fs'
import os from 'os'
import path from 'path'
import {expect, it} from '@jest/globals'
import {GradleProperties} from '../../src/version-changer/gradle-properties'
import {creator} from './util'

const create = creator('gradle-properties')

it("default save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("gradle.properties", "version=1.0.0-SNAPSHOT\n")
  const desc = GradleProperties.createFromDesc(create())
  await expect(desc.loadVersion())
    .resolves
    .toBe("1.0.0-SNAPSHOT")
  await desc.setVersion("1")
  await expect(fs.readFile("gradle.properties", {encoding: 'utf8'}))
    .resolves
    .toBe("version=1\n")
})

it("custom prop save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("gradle.properties", "project-version=1.0.0-SNAPSHOT\n")
  const desc = GradleProperties.createFromDesc(create("project-version"))
  await expect(desc.loadVersion())
    .resolves
    .toBe("1.0.0-SNAPSHOT")
  await desc.setVersion("1")
  await expect(fs.readFile("gradle.properties", {encoding: 'utf8'}))
    .resolves
    .toBe("project-version=1\n")
})

it("custom file save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("versions", "version=1.0.0-SNAPSHOT\n")
  const desc = GradleProperties.createFromDesc(create(undefined, "versions"))
  await expect(desc.loadVersion())
    .resolves
    .toBe("1.0.0-SNAPSHOT")
  await desc.setVersion("1")
  await expect(fs.readFile("versions", {encoding: 'utf8'}))
    .resolves
    .toBe("version=1\n")
})

it("custom prop and file save and write", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "test"))
  process.chdir(tempDir)
  await fs.writeFile("versions", "project-version=1.0.0-SNAPSHOT\n")
  const desc = GradleProperties.createFromDesc(create("project-version", "versions"))
  await expect(desc.loadVersion())
    .resolves
    .toBe("1.0.0-SNAPSHOT")
  await desc.setVersion("1")
  await expect(fs.readFile("versions", {encoding: 'utf8'}))
    .resolves
    .toBe("project-version=1\n")
})

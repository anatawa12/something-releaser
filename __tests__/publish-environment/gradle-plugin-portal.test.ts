import {mkdtempSync, promises as fs} from 'fs'
import os from 'os'
import * as path from 'path'
import {describe, expect, test, beforeEach, afterEach, afterAll} from '@jest/globals'
import {PropertiesFile} from '../../src/files/properties'
import {GradlePluginPortal} from '../../src/publish-environment/gradle-plugin-portal'

test("generated init script", () => {
  const portal = new GradlePluginPortal({
    key: "plugin-portal-key",
    secret: "plugin-portal-secret",
  })

  const properties = PropertiesFile.parse("")
  portal.setProperties(properties)
  expect(properties.get("gradle.publish.key")).toBe("plugin-portal-key")
  expect(properties.get("gradle.publish.secret")).toBe("plugin-portal-secret")
})

describe('test with file system', () => {
  const gradleDir = mkdtempSync(path.join(os.tmpdir(), "test"))

  beforeEach(() => {
    process.env.GRADLE_USER_HOME = gradleDir
  })
  afterEach(() => {
    delete process.env.GRADLE_USER_HOME
  })
  afterAll(async () => {
    await fs.rmdir(gradleDir, {recursive: true})
  })

  test("generate gradle.properties", async () => {
    const portal = new GradlePluginPortal({
      key: "plugin-portal-key",
      secret: "plugin-portal-secret",
    })

    // configure
    await portal.configure()

    // check file data
    const body = await fs.readFile(path.join(gradleDir, "gradle.properties"), { encoding: 'utf8' })
    expect(body).toEqual(expect.stringMatching(/gradle\.publish\.key=plugin-portal-key/))
    expect(body).toEqual(expect.stringMatching(/gradle\.publish\.secret=plugin-portal-secret/))
  }, 60 * 1000)
})

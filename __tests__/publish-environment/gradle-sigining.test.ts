import {readFileSync, mkdtempSync, promises as fs} from 'fs'
import os from 'os'
import * as path from 'path'
import {describe, expect, test, beforeEach, afterEach, afterAll} from '@jest/globals'
import {GradleMaven} from '../../src/commands/gradle-maven'
import {GradleSigning} from '../../src/commands/gradle-signing'
import {spawn} from '../test-utils/process'
import {SimpleHttp} from '../test-utils/simple-http'

test("generated init script", () => {
  const maven = new GradleSigning({
    key,
    pass: '',
  })

  expect(maven.generateInitScript())
    .toBe(`afterProject { proj ->
  if (proj.plugins.findPlugin("org.gradle.publishing") == null) return
  proj.apply {
    plugin("signing")
  }
  proj.signing.useInMemoryPgpKeys("${
  key.replace(/'/g, "\\'").replace(/\n/g, "\\n")
}", "")
  proj.publishing.publications.forEach { publication ->
    proj.signing.sign(publication)
  }
}
`)
})

describe('test with project', () => {
  const previousHome = process.env.HOME
  const testUser = "test"
  const testPass = "pass"
  const tempDir = mkdtempSync(path.join(os.tmpdir(), "test"))
  const httpDir = path.join(tempDir, "http")
  const homeDir = path.join(tempDir, "home")
  const gradleDir = path.join(tempDir, "gradle")
  const port = 1081
  let server: SimpleHttp

  beforeEach(() => {
    process.env.HOME = homeDir
    process.env.GRADLE_USER_HOME = gradleDir
    server = new SimpleHttp({
      base: httpDir,
      auth: { [testUser]: testPass },
    })
    server.start(port)
  })
  afterEach(() => {
    process.env.HOME = previousHome
    delete process.env.GRADLE_USER_HOME
    server.stop()
  })
  afterAll(async () => {
    await fs.rmdir(tempDir, {recursive: true})
  })

  test("with gradle-maven.test.project", async () => {
    const sign = new GradleSigning({
      'key': key,
      'pass': '',
    })
    const maven = new GradleMaven({
      url: `http://localhost:${port}/`,
      user: testUser,
      pass: testPass,
    })

    // configure
    await sign.configure()
    await maven.configure()

    // run
    const exit = await spawn("./gradlew", ['--no-daemon', 'publish'], {
      cwd: path.join(__dirname, "../../__tests__resources/" +
        "publish-environment/gradle-maven.test.project"),
      env: process.env,
    })
    expect(exit).toBe(0)

    // check files exists
    await fs.stat(path.join(httpDir,
      "com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar"))
    await fs.stat(path.join(httpDir,
      "com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc"))
    // check private is not published
    await expect(fs.stat(path.join(httpDir, "com/anatawa12/something-releaser/test/private")))
      .rejects
      .toThrow()
  }, 60 * 1000)
})

// by https://tools.ietf.org/id/draft-bre-openpgp-samples-01.html
const key = readFileSync(path.join(__dirname, "../../__tests__resources/gpg/bob.secret-key.asc"), {encoding: 'utf8'})

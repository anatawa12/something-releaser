import {mkdtempSync, promises as fs} from 'fs'
import os from 'os'
import * as path from 'path'
import {describe, expect, test, beforeEach, afterEach, afterAll} from '@jest/globals'
import {GradleMaven} from '../../src/commands/gradle-maven'
import {spawn} from '../test-utils/process'
import {SimpleHttp} from '../test-utils/simple-http'

test("generated init script", () => {
  const maven = new GradleMaven({
    url: "https://oss.sonatype.org/service/local/staging/deploy/maven2/",
    user: "sonatype-test",
    pass: "sonatype-password",
  })

  expect(maven.generateInitScript())
    .toBe(`afterProject { proj ->
  if (proj.plugins.findPlugin("org.gradle.maven-publish") == null) return
  proj.publishing.repositories.maven {
    url = uri("https://oss.sonatype.org/service/local/staging/deploy/maven2/")
    // gradle may disallow insecure protocol
    allowInsecureProtocol = true
    credentials.username = "sonatype-test"
    credentials.password = "sonatype-password"
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
  const port = 1080
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
    const maven = new GradleMaven({
      url: `http://localhost:${port}/`,
      user: testUser,
      pass: testPass,
    })

    // configure
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
    // check private is not published
    await expect(fs.stat(path.join(httpDir, "com/anatawa12/something-releaser/test/private")))
      .rejects
      .toThrow()
  }, 60 * 1000)
})

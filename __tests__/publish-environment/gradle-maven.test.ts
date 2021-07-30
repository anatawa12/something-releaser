import {readFileSync, mkdtempSync, promises as fs} from 'fs'
import os from 'os'
import * as path from 'path'
import {describe, expect, test, beforeEach, afterEach, afterAll} from '@jest/globals'
import {GradleMaven} from '../../src/publish-environment/gradle-maven'
import {spawn} from '../test-utils/process'
import {SimpleHttp} from '../test-utils/simple-http'

test("generated init script", () => {
  const maven = new GradleMaven({
    repo: {
      url: "https://oss.sonatype.org/service/local/staging/deploy/maven2/",
      user: "sonatype-test",
      pass: "sonatype-password",
    },
    sign: {
      'gpg-key': key,
      'gpg-pass': '',
    },
  })

  expect(maven.generateInitScript())
    .toBe(`afterProject { proj ->
  if (proj.plugins.findPlugin("org.gradle.maven-publish") == null) return
  proj.apply {
    plugin("signing")
  }
  proj.signing.useInMemoryPgpKeys("${
  key.replace(/'/g, "\\'").replace(/\n/g, "\\n")
}", "")
  proj.publishing.publications.forEach { publication ->
    proj.signing.sign(publication)
  }
  proj.publishing.repositories {
    maven {
      url = uri("https://oss.sonatype.org/service/local/staging/deploy/maven2/")
      // gradle may disallow insecure protocol
      allowInsecureProtocol = true
      credentials.username = "sonatype-test"
      credentials.password = "sonatype-password"
    }
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
      repo: {
        url: `http://localhost:${port}/`,
        user: testUser,
        pass: testPass,
      },
      sign: {
        'gpg-key': key,
        'gpg-pass': '',
      },
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
    await fs.stat(path.join(httpDir,
      "com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc"))
    // check private is not published
    expect(fs.stat(path.join(httpDir, "com/anatawa12/something-releaser/test/private")))
      .rejects
      .toThrow()
  }, 60 * 1000)
})

// by https://tools.ietf.org/id/draft-bre-openpgp-samples-01.html
const key = readFileSync(path.join(__dirname, "../../__tests__resources/gpg/bob.secret-key.asc"), {encoding: 'utf8'})

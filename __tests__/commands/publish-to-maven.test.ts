import {mkdtempSync, promises as fs, readFileSync, writeFileSync} from 'fs'
import os from 'os'
import path from 'path'
import {expect, test, describe, beforeEach, afterEach, afterAll} from '@jest/globals'
import {Author, parseAuthor, publishToMaven} from '../../src/commands/publish-to-maven'
import {SimpleHttp} from '../test-utils/simple-http'

describe("parse author simple format", () => {
  function run_test(str: string, value: Author): void {
    expect(parseAuthor(str)).toEqual([value])
  }

  test('simple format', async () => {
    run_test("name <mail@test> @test", {roles: [], name: 'name', id: 'test', mail: 'mail@test'})
    run_test("     <mail@test> @test", {roles: [], id: 'test', mail: 'mail@test'})
    run_test("name <mail@test>      ", {roles: [], name: 'name', mail: 'mail@test'})
    run_test("     <mail@test>      ", {roles: [], mail: 'mail@test'})
    run_test("name             @test", {roles: [], name: 'name', id: 'test'})
    run_test("                 @test", {roles: [], id: 'test'})
    run_test("name                  ", {roles: [], name: 'name'})
  })

  test('simple kvp', async () => {
    run_test("name=name,id=test,mail=mail", {roles: [], name: 'name', id: 'test', mail: 'mail'})
    run_test("id=test,mail=mail", {roles: [], id: 'test', mail: 'mail'})
    run_test("name=name,mail=mail", {roles: [], name: 'name', mail: 'mail'})
    run_test("mail=mail", {roles: [], mail: 'mail'})
    run_test("name=name,id=test", {roles: [], name: 'name', id: 'test'})
    run_test("id=test", {roles: [], id: 'test'})
    run_test("name=name", {roles: [], name: 'name'})
  })

  test('roles', async () => {
    run_test("name=name,id=test,mail=mail,role=programmer,role=designer", {
      roles: ['programmer', 'designer'],
      name: 'name',
      id: 'test',
      mail: 'mail',
    })
    run_test("name=name,id=test,mail=mail,role=programmer", {
      roles: ['programmer'],
      name: 'name',
      id: 'test',
      mail: 'mail',
    })
  })
})

describe("run the command", () => {
  const testUser = "test"
  const testPass = "pass"
  const tempDir = mkdtempSync(path.join(os.tmpdir(), "test"))
  const httpDir = path.join(tempDir, "http")
  const testFile = path.join(tempDir, "testFile.jar")
  const port = 1082
  let server: SimpleHttp

  beforeEach(() => {
    server = new SimpleHttp({
      base: httpDir,
      auth: { [testUser]: testPass },
    })
    server.start(port)
    writeFileSync(testFile, "test-file-here")
  })
  afterEach(() => {
    server.stop()
  })
  afterAll(async () => {
    await fs.rmdir(tempDir, {recursive: true})
  })

  test("upload test", async () => {
    // configure
    await publishToMaven([
      '--file', testFile,
      '--group-id', 'com/anatawa12/something-releaser/test',
      '--artifact-id', 'publish',
      '--version-name', 'unspecified',
      '--classifier', 'classifier',
      '--packaging', 'pom',
      '--repository', `http://localhost:${port}/`,
      '--user', `${testUser}:${testPass}`,
      '--signing-key', key,
    ])

    // check files exists
    async function checkExists(name: string): Promise<void> {
      await fs.stat(path.join(httpDir,
        `com/anatawa12/something-releaser/test/publish/unspecified/${name}`))
    }
    await checkExists("publish-unspecified-classifier.jar")
    await checkExists("publish-unspecified-classifier.jar.asc")
    await checkExists("publish-unspecified-classifier.jar.asc.md5")
    await checkExists("publish-unspecified-classifier.jar.asc.sha1")
    await checkExists("publish-unspecified-classifier.jar.asc.sha256")
    await checkExists("publish-unspecified-classifier.jar.asc.sha512")
    await checkExists("publish-unspecified-classifier.jar.md5")
    await checkExists("publish-unspecified-classifier.jar.sha1")
    await checkExists("publish-unspecified-classifier.jar.sha256")
    await checkExists("publish-unspecified-classifier.jar.sha512")
    await checkExists("publish-unspecified-classifier.pom")
    await checkExists("publish-unspecified-classifier.pom.asc")
    await checkExists("publish-unspecified-classifier.pom.asc.md5")
    await checkExists("publish-unspecified-classifier.pom.asc.sha1")
    await checkExists("publish-unspecified-classifier.pom.asc.sha256")
    await checkExists("publish-unspecified-classifier.pom.asc.sha512")
    await checkExists("publish-unspecified-classifier.pom.md5")
    await checkExists("publish-unspecified-classifier.pom.sha1")
    await checkExists("publish-unspecified-classifier.pom.sha256")
    await checkExists("publish-unspecified-classifier.pom.sha512")
    // check private is not published
    await expect(fs.stat(path.join(httpDir, "com/anatawa12/something-releaser/test/private")))
      .rejects
      .toThrow()
  }, 60 * 1000)
})

const key = readFileSync(path.join(__dirname, "../../__tests__resources/gpg/bob.secret-key.asc"), {encoding: 'utf8'})

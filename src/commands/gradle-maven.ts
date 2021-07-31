import {promises as fs} from 'fs'
import * as path from 'path'
import {GroovyGenerator} from '../files/groovy'
import {gradleHomeDir} from '../utils'

type RepoConfig = {
  url: string
  'user'?: string
  'pass'?: string
}

type Config = {
  repo: RepoConfig | RepoConfig[]
  sign?: {
    'gpg-key': string
    'gpg-pass': string
  } 
}

export class GradleMaven {
  private readonly repo: RepoConfig[]
  private readonly sign: Config['sign']

  constructor(config: Config) {
    this.repo = Array.isArray(config.repo) ? config.repo : [config.repo]
    this.sign = config.sign
  }

  generateInitScript(): string {
    const ge = new GroovyGenerator()
    ge.block("afterProject { proj ->", () => {
      ge.line("if (proj.plugins.findPlugin(%s) == null) return", "org.gradle.maven-publish")

      // configure signing
      const sign = this.sign
      if (sign != null) {
        // apply signing plugin.
        ge.block("proj.apply {", () => {
          ge.line('plugin(%s)', 'signing')
        })

        // set signing key
        ge.line("proj.signing.useInMemoryPgpKeys(%s, %s)", sign['gpg-key'], sign['gpg-pass'])

        // configure as sign for each publications
        ge.block("proj.publishing.publications.forEach { publication ->", () => {
          ge.line('proj.signing.sign(publication)')
        })
      }

      // configure publish repository
      ge.block("proj.publishing.repositories {", () => {
        for (const repo of this.repo) {
          ge.block("maven {", () => {
            ge.line("url = uri(%s)", repo.url)
            ge.line("// gradle may disallow insecure protocol")
            ge.line("allowInsecureProtocol = true")
            if (repo.user)
              ge.line("credentials.username = %s", repo.user)
            if (repo.pass)
              ge.line("credentials.password = %s", repo.pass)
          })
        }
      })
    })

    return ge.toString()
  }

  async configure(): Promise<void> {

    const init_d = path.join(gradleHomeDir(), "init.d")
    const initScriptPath = path.join(init_d, "gradle-maven.gradle")
    await fs.mkdir(init_d, { recursive: true })
    // if file exists: throw error
    // eslint-disable-next-line github/no-then
    if (await fs.stat(initScriptPath).then(() => true, () => false)) {
      throw new Error("can't create gradle-maven.gradle: exists")
    }
    await fs.writeFile(initScriptPath, this.generateInitScript(), { encoding: 'utf8' })
  }
  
  toString(): string {
    let res = `gradle-maven(`
    res += `to ${this.repo.map(r => r.url).join(", ")}`
    if (this.sign) {
      res += ` with signature`
    }
    res += ')'
    return res
  }
}

import {promises as fs} from 'fs'
import * as path from 'path'
import {GroovyGenerator} from '../files/groovy'
import {gradleHomeDir} from '../utils'

type Config = {
  url: string
  'user'?: string
  'pass'?: string
}

export class GradleMaven {
  private readonly repo: Config

  constructor(repo: Config) {
    this.repo = repo
  }

  generateInitScript(): string {
    const ge = new GroovyGenerator()
    ge.block("afterProject { proj ->", () => {
      ge.line("if (proj.plugins.findPlugin(%s) == null) return", "org.gradle.maven-publish")

      // configure publish repository
      ge.block("proj.publishing.repositories.maven {", () => {
        ge.line("url = uri(%s)", this.repo.url)
        ge.line("// gradle may disallow insecure protocol")
        ge.line("allowInsecureProtocol = true")
        if (this.repo.user)
          ge.line("credentials.username = %s", this.repo.user)
        if (this.repo.pass)
          ge.line("credentials.password = %s", this.repo.pass)
      })
    })

    return ge.toString()
  }

  async configure(): Promise<void> {

    const init_d = path.join(gradleHomeDir(), "init.d")
    const initScriptPath = path.join(init_d, `gradle-maven.${Date.now()}.${process.pid}.gradle`)
    await fs.mkdir(init_d, { recursive: true })
    // if file exists: throw error
    // eslint-disable-next-line github/no-then
    if (await fs.stat(initScriptPath).then(() => true, () => false)) {
      throw new Error("can't create gradle-maven.gradle: exists")
    }
    await fs.writeFile(initScriptPath, this.generateInitScript(), { encoding: 'utf8' })
  }
  
  toString(): string {
    return `gradle-maven(to ${this.repo.url})`
  }
}

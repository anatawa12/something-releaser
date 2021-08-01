import {promises as fs} from 'fs'
import * as path from 'path'
import {GroovyGenerator} from '../files/groovy'
import {gradleHomeDir} from '../utils'

type Config = {
  key: string
  pass: string
}

export class GradleSigning {
  private readonly sign: Config

  constructor(config: Config) {
    this.sign = config
  }

  generateInitScript(): string {
    const ge = new GroovyGenerator()
    ge.block("afterProject { proj ->", () => {
      ge.line("if (proj.plugins.findPlugin(%s) == null) return", "org.gradle.publishing")

      // apply signing plugin.
      ge.block("proj.apply {", () => {
        ge.line('plugin(%s)', 'signing')
      })

      // set signing key
      ge.line("proj.signing.useInMemoryPgpKeys(%s, %s)", this.sign.key, this.sign.pass)

      // configure as sign for each publications
      ge.block("proj.publishing.publications.forEach { publication ->", () => {
        ge.line('proj.signing.sign(publication)')
      })
    })

    return ge.toString()
  }

  async configure(): Promise<void> {
    const init_d = path.join(gradleHomeDir(), "init.d")
    const initScriptPath = path.join(init_d, `gradle-signing.gradle`)
    await fs.mkdir(init_d, { recursive: true })
    // if file exists: throw error
    // eslint-disable-next-line github/no-then
    if (await fs.stat(initScriptPath).then(() => true, () => false)) {
      throw new Error("can't create gradle-maven.gradle: exists")
    }
    await fs.writeFile(initScriptPath, this.generateInitScript(), { encoding: 'utf8' })
  }
  
  toString(): string {
    return `gradle-sigining()`
  }
}

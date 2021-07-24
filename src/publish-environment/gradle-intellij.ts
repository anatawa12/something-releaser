import {promises as fs} from 'fs'
import * as path from 'path'
import {GroovyGenerator} from '../files/groovy'
import {Yaml} from '../types'
import {gradleHomeDir} from '../utils'
import {Configurator} from '.'

type Config = NonNullable<Yaml['publish-environment']['gradle-intellij']>

export class GradleIntellij implements Configurator {
  private readonly token: string

  constructor(config: Config) {
    this.token = config.token
  }

  generateInitScript(): string {
    const ge = new GroovyGenerator()
    ge.block("afterProject { proj ->", () => {
      ge.line("if (proj.plugins.findPlugin(%s) == null) return", "org.jetbrains.intellij")
      // configure publish repository
      ge.block("proj.tasks.publishPlugin {", () => {
        ge.line("token = %s", this.token)
      })
    })

    return ge.toString()
  }

  async configure(): Promise<void> {

    const init_d = path.join(gradleHomeDir(), "init.d")
    const initScriptPath = path.join(init_d, "gradle-intellij.gradle")
    await fs.mkdir(init_d, { recursive: true })
    // if file exists: throw error
    // eslint-disable-next-line github/no-then
    if (await fs.stat(initScriptPath).then(() => true, () => false)) {
      throw new Error("can't create gradle-intellij.gradle: exists")
    }
    await fs.writeFile(initScriptPath, this.generateInitScript(), { encoding: 'utf8' })
  }
  
  toString(): string {
    return 'gradle-intellij(with token)'
  }

  static create(args: Yaml['publish-environment']['gradle-intellij']): GradleIntellij[] {
    if (!args)
      return []
    return [new GradleIntellij(args)]
  }
}

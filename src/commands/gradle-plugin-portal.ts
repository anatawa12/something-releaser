import {promises as fs} from 'fs'
import * as path from 'path'
import {GroovyGenerator} from '../files/groovy'
import {PropertiesFile} from '../files/properties'
import {gradleHomeDir} from '../utils'

type Config = {
  key: string;
  secret: string;
}

export class GradlePluginPortal {
  private readonly key: string
  private readonly secret: string

  constructor(config: Config) {
    this.key = config.key
    this.secret = config.secret
  }

  setProperties(properties: PropertiesFile): void {
    properties.set("gradle.publish.key", this.key)
    properties.set("gradle.publish.secret", this.secret)
  }

  generateInitScript(): string {
    const ge = new GroovyGenerator()
    ge.block("afterProject { proj ->", () => {
      ge.line("if (proj.plugins.findPlugin(%s) == null) return", "com.gradle.plugin-publish")

      // configure publish repository
      ge.block("if (proj.pluginBundle.mavenCoordinates.groupId == null) {", () => {
        ge.line("throw new Exception(%s)", 
          "mavenCoordinates.groupId is not specified!")
      })
    })

    return ge.toString()
  }

  async configure(): Promise<void> {
    const propertiesFilePath = path.join(gradleHomeDir(), "gradle.properties")
    const properties = PropertiesFile.parse(await readOrEmpty(propertiesFilePath))
    this.setProperties(properties)

    const init_d = path.join(gradleHomeDir(), "init.d")
    const initScriptPath = path.join(init_d, "gradle-plugin-portal.gradle")
    await fs.mkdir(init_d, { recursive: true })
    // if file exists: throw error
    if (await fs.stat(initScriptPath).then(() => true, () => false)) {
      throw new Error("can't create gradle-intellij.gradle: exists")
    }
    await fs.writeFile(initScriptPath, this.generateInitScript(), { encoding: 'utf8' })

    await fs.writeFile(propertiesFilePath, properties.toSource(), { encoding: 'utf8' })
  }
  
  toString(): string {
    return 'gradle-plugin-portal(with key and secrets)'
  }
}

async function readOrEmpty(p: string): Promise<string> {
  try {
    return await fs.readFile(p, { encoding: 'utf8' })
  } catch (e) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((e as any).code === 'ENOENT')
      return ''
    throw e
  }
}

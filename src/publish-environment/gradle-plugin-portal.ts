import {promises as fs} from 'fs'
import * as path from 'path'
import {PropertiesFile} from '../files/properties'
import {Yaml} from '../types'
import {gradleHomeDir} from '../utils'
import {Configurator} from '.'

type Config = NonNullable<Yaml['publish-environment']['gradle-plugin-portal']>

export class GradlePluginPortal implements Configurator {
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

  async configure(): Promise<void> {
    const propertiesFilePath = path.join(gradleHomeDir(), "gradle.properties")
    const properties = PropertiesFile.parse(await readOrEmpty(propertiesFilePath))
    this.setProperties(properties)

    await fs.writeFile(propertiesFilePath, properties.toSource(), { encoding: 'utf8' })
  }
  
  toString(): string {
    return 'gradle-plugin-portal(with key and secrets)'
  }

  static create(args: Yaml['publish-environment']['gradle-plugin-portal']): GradlePluginPortal[] {
    if (!args)
      return []
    return [new GradlePluginPortal(args)]
  }
}

async function readOrEmpty(p: string): Promise<string> {
  try {
    return await fs.readFile(p, { encoding: 'utf8' })
  } catch (e) {
    if (e.code === 'ENOENT')
      return ''
    throw e
  }
}

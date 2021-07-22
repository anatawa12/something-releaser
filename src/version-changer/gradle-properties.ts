import {VersionChanger} from '.'
import {Version, Yaml} from '../types'
import * as fs from 'fs'
import {PropertiesFile} from '../files/properties'

export class GradleProperties implements VersionChanger {
  private readonly property: string
  private readonly path: string

  static createArray(
    args: Yaml['version-changer']['gradle-properties'],
  ): GradleProperties[] {
    if (!args) 
      return []
    if (Array.isArray(args)) {
      return args.map(arg => new GradleProperties(arg))
    } else {
      return [new GradleProperties(args)]
    }
  }

  private constructor(arg: {property?: string; path?: string}) {
    this.property = arg.property ?? 'version'
    this.path = arg.path ?? 'gradle.properties'
  }

  async loadVersion(): Promise<Version> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const properties = PropertiesFile.parse(source)
    const version = properties.get(this.property)
    if (!version)
      throw new Error(`no such property: ${this.property}`)
    return Version.parse(version)
  }

  async setVersion(version: Version): Promise<void> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const properties = PropertiesFile.parse(source)
    properties.set(this.property, version.toString())
    await fs.promises.writeFile(this.path, version, {encoding: 'utf-8'})
  }

  toString(): string {
    return `gradle-properties(at ${this.path} prop ${this.property})`
  }
}

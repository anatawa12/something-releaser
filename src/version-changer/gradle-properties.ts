import * as fs from 'fs'
import {PropertiesFile} from '../files/properties'

import {ChangerDescriptor, VersionChanger} from '.'

export class GradleProperties implements VersionChanger {
  private readonly property: string
  private readonly path: string

  static createFromDesc({info: property, path}: ChangerDescriptor): GradleProperties {
    return new GradleProperties({
      property: property || 'version',
      path: path || 'gradle.properties',
    })
  }

  private constructor(arg: {property?: string; path?: string}) {
    this.property = arg.property ?? 'version'
    this.path = arg.path ?? 'gradle.properties'
  }

  async loadVersion(): Promise<string> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const properties = PropertiesFile.parse(source)
    const version = properties.get(this.property)
    if (!version)
      throw new Error(`no such property: ${this.property}`)
    return version
  }

  async setVersion(version: string): Promise<void> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const properties = PropertiesFile.parse(source)
    properties.set(this.property, version)
    await fs.promises.writeFile(this.path, properties.toSource(), {encoding: 'utf-8'})
  }

  toString(): string {
    return `gradle-properties(at ${this.path} prop ${this.property})`
  }
}

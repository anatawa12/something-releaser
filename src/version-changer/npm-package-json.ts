import * as fs from 'fs'
import {JsonFile} from "../files/json"
import {ChangerDescriptor, VersionChanger} from '.'

export class NpmPackageJson implements VersionChanger {
  private readonly property: string
  private readonly paths: string[]
  private static readonly defaultPaths = ['package.json', 'package-lock.json']

  static createFromDesc({info: property, path}: ChangerDescriptor): NpmPackageJson {
    return new NpmPackageJson({
      property: property || 'version',
      paths: path?.split(':') || NpmPackageJson.defaultPaths,
    })
  }

  private constructor(arg: {property?: string; paths?: string[]}) {
    this.property = arg.property ?? 'version'
    this.paths = arg.paths ?? NpmPackageJson.defaultPaths
  }

  async loadVersion(): Promise<string> {
    const source = await fs.promises.readFile(this.paths[0], {encoding: 'utf-8'})
    const properties = JsonFile.parse(source)
    const version = properties.get([this.property])
    if (!version)
      throw new Error(`no such property: ${this.property}`)
    return `${version}`
  }

  async setVersion(version: string): Promise<void> {
    for (const path of this.paths) {
      const source = await fs.promises.readFile(path, {encoding: 'utf-8'})
      const properties = JsonFile.parse(source)
      properties.set([this.property], version)
      await fs.promises.writeFile(path, properties.toSource(), {encoding: 'utf-8'})
    }
  }

  toString(): string {
    return `npm(at ${this.paths.join(", ")} property ${this.property})`
  }
}

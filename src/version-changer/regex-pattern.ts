import * as fs from 'fs'
import {Version, asPair} from '../utils'
import {VersionChanger} from '.'

export class RegexPattern implements VersionChanger {
  private readonly match: RegExp
  private readonly path: string

  static createFromDesc(desc: string | undefined): RegexPattern {
    if (desc == null)
      throw new Error(`regex-pattern requires <pattern>@<path>`)
    const [pattern, path] = asPair(desc, '@', false)
    if (!pattern || !path)
      throw new Error(`regex-pattern requires <pattern>@<path>`)
    return new RegexPattern({ pattern, path })
  }

  private constructor(arg: {pattern: string; path: string}) {
    const [pre, suf] = asPair(arg.pattern, '$1', false)
    if (suf == null)
      throw new Error(`regex-pattern: pattern does not includes $1`)
    new RegExp(pre)
    new RegExp(suf)
    this.match = new RegExp(`(?<prefix>${pre})(?<version>.*)(?<suffix>${suf})`)
    this.path = arg.path
  }

  async loadVersion(): Promise<Version> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const matchResult = source.match(this.match)
    if (!matchResult)
      throw new Error(`no such region matches ${this.match}`)
    if (!matchResult.groups)
      throw new Error(`logic failure ${this.match}`)
    return Version.parse(matchResult.groups.version)
  }

  async setVersion(version: Version): Promise<void> {
    const source = await fs.promises.readFile(this.path, { encoding: 'utf-8' })
    const replaced = source.replace(this.match, `$<prefix>${version.toString()}$<suffix>`)
    await fs.promises.writeFile(this.path, replaced, {encoding: 'utf-8'})
  }

  toString(): string {
    return `regex-pattern.ts(at ${this.path} via ${this.match})`
  }
}

import {asPair, Version} from '../utils'
import {GradleProperties} from './gradle-properties'
import {RegexPattern} from './regex-pattern'

export interface VersionChanger {
  loadVersion(): Promise<Version>
  setVersion(version: Version): Promise<void>
  toString(): string
}

export class VersionChangers {
  private readonly changers: VersionChanger[]

  constructor(changers: VersionChanger[]) {
    this.changers = changers
    if (changers.length === 0)
      throw new Error("invalid version changers: empty")
  }

  async getVersionName(): Promise<Version> {
    const versionMap = Object.create(null) as { [P in string]?: VersionChanger[] }
    for (const changer of this.changers) {
      try {
        const version = (await changer.loadVersion()).toString()
        const changers = versionMap[version]
        if (changers)
          changers.push(changer)
        else
          versionMap[version] = [changer]
      } catch (e) {
        throw new Error(`loading version from ${changer.toString()}: ${e}`)
      }
    }

    const versions = Object.entries<VersionChanger[]|undefined>(versionMap)
    if (versions.length === 1)
      return Version.parse(versions[0][0])

    let msg = "multiple versions found!"
    for (const [version, changers] of versions) {
      if (!changers)
        continue
      msg += `\n${changers.join(",")} says ${version}`
    }
    throw new Error(msg)
  }

  async setVersionName(version: Version): Promise<void> {
    for (const changer of this.changers) {
      try {
        await changer.setVersion(version)
      } catch (e) {
        throw new Error(`setting version via ${changer.toString()}: ${e}`)
      }
    }
  }
}

export function createFromEnvVariable(str: string): VersionChangers {
  const result: VersionChanger[] = []

  for (const changerDesc of str.split(';')) {
    const [changer, desc] = asPair(changerDesc, /:|(?=@)/, false)
    switch (changer) {
      case 'gradle-properties':
        result.push(GradleProperties.createFromDesc(desc))
        break
      case 'regex-pattern':
        result.push(RegexPattern.createFromDesc(desc))
        break
      default:
        throw new Error(`unknown changer: ${changer}`)
    }
  }

  return new VersionChangers(result)
}

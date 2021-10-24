import {Version} from '../utils'
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

export type ChangerDescriptor = {
  changer: string,
  info: string | undefined,
  path: string | undefined,
}

export function parseDescriptor(changerDesc: string): ChangerDescriptor {
  const match = changerDesc.match(/(?<changer>[^:@]*)(?::(?<info>[^@]*))?(?:@(?<file>[\s\S]*))?/)
  if (match == null)
    throw new Error(`logic failure: don't match: '${changerDesc}'`)
  return match.groups as ChangerDescriptor
}

export function createChanger(descriptor: ChangerDescriptor): VersionChanger {
  switch (descriptor.changer) {
    case 'gradle-properties':
      return GradleProperties.createFromDesc(descriptor)
    case 'regex-pattern':
      return RegexPattern.createFromDesc(descriptor)
    default:
      throw new Error(`unknown changer: ${descriptor.changer}`)
  }
}

export function createChangers(descriptors: ChangerDescriptor[]): VersionChangers {
  return new VersionChangers(descriptors.map(createChanger))
}

export function createFromEnvVariable(str: string): VersionChangers {
  return createChangers(str.split(';').map(parseDescriptor))
}

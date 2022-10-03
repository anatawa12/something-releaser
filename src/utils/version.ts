import {logicFailre} from '../utils'

interface VersionConfig {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly release: Release
}

export type Release =
  // none
  | readonly [kind: 'stable']
  // -SNAPSHOT
  | readonly [kind: 'snapshot']
  // -alphaN
  | readonly [kind: 'alpha', number: number]
  // -betaN
  | readonly [kind: 'beta', number: number]
  // -rcN
  | readonly [kind: 'candidate', number: number]

function useSemverCompatible(): boolean {
  return process.env.SOMETHING_RELEASER_SEMVER === "1"
}

export class Version {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly release: Release

  constructor(config: VersionConfig)
  constructor(major: number, release?: Release)
  constructor(major: number, minor: number, release?: Release)
  constructor(major: number, minor: number, patch: number, release?: Release)
  constructor(
    arg0: number | VersionConfig,
    arg1?: number | Release,
    arg2?: number | Release,
    arg3?: Release,
  ) {
    if (typeof arg0 == 'object') {
      this.major = arg0.major
      this.minor = arg0.minor
      this.patch = arg0.patch
      this.release = arg0.release
      if (this.patch != null && this.minor == null)
        throw new Error("patch exists but minor doesn't")
      if (this.minor != null && this.major == null)
        throw new Error("minor exists but major doesn't")
    } else if (typeof arg1 != 'number') {
      this.major = arg0
      this.minor = undefined
      this.patch = undefined
      this.release = arg1 ?? ['stable']
    } else if (typeof arg2 != 'number') {
      this.major = arg0
      this.minor = arg1
      this.patch = undefined
      this.release = arg2 ?? ['stable']
    } else {
      this.major = arg0
      this.minor = arg1
      this.patch = arg2
      this.release = arg3 ?? ['stable']
    }
    if (!Number.isInteger(this.major))
      throw new Error('major is not a integer')
    if (this.minor !== undefined && !Number.isInteger(this.minor))
      throw new Error('minor is not a integer')
    if (this.patch !== undefined && !Number.isInteger(this.patch))
      throw new Error('patch is not a integer')
  }

  static parse(value: string): Version {
    const regex = /^v?(?<maj>\d+)(\.(?<min>\d+))?(\.(?<pat>\d+))?(-(?<snap>SNAPSHOT)|-((?<alpha>alpha)|(?<beta>beta)|(?<rc>rc))\.?(?<n>\d+))?$/i
    const match = value.match(regex)
    if (match == null)
      throw new Error(`the version name doesn't match ${regex}`)

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const groups = match.groups!

    return new Version({
      major: parseInt(groups.maj),
      minor: groups.min ? parseInt(groups.min) : undefined,
      patch: groups.pat ? parseInt(groups.pat) : undefined,
      release: 
        groups.snap ? ['snapshot']
        : groups.alpha ? ['alpha', parseInt(groups.n)]
        : groups.beta ? ['beta', parseInt(groups.n)]
        : groups.rc ? ['candidate', parseInt(groups.n)]
        : ['stable'],
    })
  }

  toString(): string {
    let r = `${this.major}`
    if (this.minor != null)
      r += `.${this.minor}`
    if (this.patch != null)
      r += `.${this.patch}`
    switch (this.release[0]) {
      case 'stable':
        break
      case 'snapshot':
        r += '-SNAPSHOT'
        break
      case 'alpha':
        r += '-alpha'
        if (useSemverCompatible())
          r += '.'
        r += this.release[1]
        break
      case 'beta':
        r += '-beta'
        if (useSemverCompatible())
          r += '.'
        r += this.release[1]
        break
      case 'candidate':
        r += '-rc'
        if (useSemverCompatible())
          r += '.'
        r += this.release[1]
        break
      default:
        logicFailre("release type", this.release[0])
    }
    return r
  }

  makeStable(): Version {
    return new Version({...this, release: ['stable']})
  }

  makeAlpha(number: number): Version {
    return new Version({...this, release: ['alpha', number]})
  }

  makeBeta(number: number): Version {
    return new Version({...this, release: ['beta', number]})
  }

  makeCandidate(number: number): Version {
    return new Version({...this, release: ['candidate', number]})
  }

  makeSnapshot(): Version {
    return new Version({...this, release: ['snapshot']})
  }

  makeMajorOnly(): Version {
    return new Version({major: this.major, minor: undefined, patch: undefined, release: this.release})
  }

  makeMajorMinor(): Version {
    return new Version({major: this.major, minor: this.minor ?? 0, patch: undefined, release: this.release})
  }

  makeMajorMinorPatch(): Version {
    return new Version({major: this.major, minor: this.minor ?? 0, patch: this.patch ?? 0, release: this.release})
  }

  next(channel?: "prerelease" | "patch" | "minor" | "major" | undefined | null): Version {
    if (channel == null) {
      if (this.release[1] != null)
        channel = "prerelease"
      else if (this.patch != null)
        channel = "patch"
      else if (this.minor != null)
        channel = "minor"
      else
        channel = "major"
    }
    switch (channel) {
      case "prerelease":
        if (this.release[1] == null)
          throw new Error("requested to update prerelease version but not found")
        return new Version({...this, release: [this.release[0], this.release[1] + 1]})
      case "patch":
        if (this.patch == null)
          throw new Error("requested to update patch version but not found")
        return new Version({...this, patch: this.patch + 1})
      case "minor":
        if (this.minor == null)
          throw new Error("requested to update minor version but not found")
        return new Version({...this, minor: this.minor + 1})
      case "major":
        return new Version({...this, major: this.major + 1})
      default:
        logicFailre("unknown channel", channel)
    }
  }
}

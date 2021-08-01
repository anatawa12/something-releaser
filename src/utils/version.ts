interface VersionConfig {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly snapshot: boolean
}

export class Version {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly snapshot: boolean

  constructor(config: VersionConfig)
  constructor(major: number, snapshot?: boolean)
  constructor(major: number, minor: number, snapshot?: boolean)
  constructor(major: number, minor: number, patch: number, snapshot?: boolean)
  constructor(
    arg0: number | VersionConfig,
    arg1?: number | boolean,
    arg2?: number | boolean,
    arg3?: boolean,
  ) {
    if (typeof arg0 == 'object') {
      this.major = arg0.major
      this.minor = arg0.minor
      this.patch = arg0.patch
      this.snapshot = arg0.snapshot
      if (this.patch != null && this.minor == null)
        throw new Error("patch exists but minor doesn't")
      if (this.minor != null && this.major == null)
        throw new Error("minor exists but major doesn't")
    } else if (typeof arg1 != 'number') {
      this.major = arg0
      this.minor = undefined
      this.patch = undefined
      this.snapshot = arg1 ?? false
    } else if (typeof arg2 != 'number') {
      this.major = arg0
      this.minor = arg1
      this.patch = undefined
      this.snapshot = arg2 ?? false
    } else {
      this.major = arg0
      this.minor = arg1
      this.patch = arg2
      this.snapshot = arg3 ?? false
    }
    if (!Number.isInteger(this.major))
      throw new Error('major is not a integer')
    if (this.minor !== undefined && !Number.isInteger(this.minor))
      throw new Error('minor is not a integer')
    if (this.patch !== undefined && !Number.isInteger(this.patch))
      throw new Error('patch is not a integer')
  }

  static parse(value: string): Version {
    const regex = /^v?(\d+)(?:.(\d+))?(?:.(\d+))?(-SNAPSHOT)?$/i
    const match = value.match(regex)
    if (match == null)
      throw new Error(`the version name doesn't match ${regex}`)
    const snapshot = !!match[4]
    const major = parseInt(match[1])
    if (match[2]) {
      const minor = parseInt(match[2])
      if (match[3]) {
        const patch = parseInt(match[3])
        return new Version(major, minor, patch, snapshot)
      } else {
        return new Version(major, minor, snapshot)
      }
    } else {
      return new Version(major, snapshot)
    }
  }

  toString(): string {
    let r = `${this.major}`
    if (this.minor != null)
      r += `.${this.minor}`
    if (this.patch != null)
      r += `.${this.patch}`
    if (this.snapshot)
      r += '-SNAPSHOT'
    return r
  }

  unSnapshot(): Version {
    return new Version({...this, snapshot: false})
  }

  makeSnapshot(): Version {
    return new Version({...this, snapshot: true})
  }

  next(): Version {
    if (this.patch != null)
      return new Version({...this, patch: this.patch + 1})
    if (this.minor != null)
      return new Version({...this, minor: this.minor + 1})
    return new Version({...this, major: this.major + 1})
  }
}

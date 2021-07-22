import {GitHub} from '@actions/github/lib/utils'

export type Octokit = InstanceType<typeof GitHub>

export {Schema as Yaml} from './generated/yaml'

export type ObjectMap<Key extends string | number | symbol, Value> = {
  [_ in Key]: Value
}

export type KeyOfValue<Object, Value> = keyof (Object &
  ObjectMap<keyof Object, Value>)

export class Version {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly snapshot: boolean

  constructor(major: number, snapshot?: boolean)
  constructor(major: number, minor: number, snapshot?: boolean)
  constructor(major: number, minor: number, patch: number, snapshot?: boolean)
  constructor(
    arg0: number,
    arg1?: number | boolean,
    arg2?: number | boolean,
    arg3?: boolean
  ) {
    if (typeof arg1 != 'number') {
      this.major = arg0
      this.minor = undefined
      this.patch = undefined
      this.snapshot = arg1 ?? true
    } else if (typeof arg2 != 'number') {
      this.major = arg0
      this.minor = arg1
      this.patch = undefined
      this.snapshot = arg2 ?? true
    } else {
      this.major = arg0
      this.minor = arg1
      this.patch = arg2
      this.snapshot = arg3 ?? true
    }
    if (!Number.isInteger(this.major)) throw new Error('major is not a integer')
    if (!Number.isInteger(this.minor)) throw new Error('minor is not a integer')
    if (!Number.isInteger(this.patch)) throw new Error('patch is not a integer')
  }

  static parse(value: string): Version {
    const regex = /^v?(\d+)(.\d+)?(.\d+)?(-SNAPSHOT)?$/i
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
    if (this.minor) r += `.${this.minor}`
    if (this.patch) r += `.${this.patch}`
    if (this.snapshot) r += '-SNAPSHOT'
    return r
  }
}

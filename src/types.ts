export class Version {
  readonly major: number
  readonly minor: number | undefined
  readonly patch: number | undefined
  readonly snapshot: boolean

  constructor(config: Version)
  constructor(major: number, snapshot?: boolean)
  constructor(major: number, minor: number, snapshot?: boolean)
  constructor(major: number, minor: number, patch: number, snapshot?: boolean)
  constructor(
    arg0: number | Version,
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
    if (!Number.isInteger(this.major)) 
      throw new Error('major is not a integer')
    if (!Number.isInteger(this.minor)) 
      throw new Error('minor is not a integer')
    if (!Number.isInteger(this.patch)) 
      throw new Error('patch is not a integer')
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
    if (this.minor) 
      r += `.${this.minor}`
    if (this.patch) 
      r += `.${this.patch}`
    if (this.snapshot) 
      r += '-SNAPSHOT'
    return r
  }

  unSnapshot(): Version {
    return new Version({...this, snapshot: false})
  }
}

// string builder
export class StringBuilder {
  body: string

  constructor() {
    this.body = ""
  }

  ln(line: string): void {
    this.body += `${line}\n`
  }

  append(elem: string): this {
    this.body += elem
    return this
  }

  toString(): string {
    return this.body
  }
}

// about formatted strings

// get list of the character after %
type FormatChars<T extends string> =
// eslint-disable-next-line @typescript-eslint/no-unused-vars
  T extends `${infer _}%`
    ? never
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    : T extends `${infer _}%${infer Format}${infer Tail}`
      ? Format extends '%'
        ? FormatChars<Tail>
        : [Format, ...FormatChars<Tail>]
      : []

// apply FormatValue for each T[n]
type FormatValues<Map, T extends string[]> =
  T extends [] ? []
    : T extends [infer Head, ...infer Tail]
      ? Head extends string
        ? Tail extends string[]
          ? [FormatValue<Map, Head>, ...FormatValues<Map, Tail>]
          : never
        : never
      : never

// returns type of Map[T] and never if not found
type FormatValue<Map, T extends string> = T extends keyof Map ? Map[T] : never

// same as FormatValues<Map, FormatChars<T>> but returns [never]
// if FormatValues<Map, FormatChars<T>> is never
export type FormatArgs<Map, T extends string> =
  FormatValues<Map, FormatChars<T>> extends never
    ? [never]
    : FormatValues<Map, FormatChars<T>>

export type FormatProcessor<Map> = {
  [P in keyof Map]: (value: Map[P]) => string
}

export type Awaitable<T> = T | PromiseLike<T>

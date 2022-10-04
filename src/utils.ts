/* eslint-disable @typescript-eslint/no-explicit-any */
import {homedir} from 'os'
import * as path from 'path'
import {FormatArgs, FormatProcessor} from './types'
export {
  asSequence, 
  asAsyncSequence, 
  AsyncSequence,
  Sequence, 
} from './utils/sequence'
export {Version} from './utils/version'
export {StringBuilder} from './utils/string-builder'

export function headingAndLast<Heading extends any[], Last>(array: [...Heading, Last]): [Heading, Last]
export function headingAndLast<E>(array: E[]): [E[], E]
export function headingAndLast(array: any[]): any {
  return [array.slice(0, array.length - 1), array[array.length - 1]] as never
}

export function logicFailre(msg: string, never?: never): never {
  throw new Error(`logic failre: ${msg}: ${never}`)
}

export function throws(error: Error): never {
  throw error
}

export function processFormat<Map, P extends string>(
  format: P,
  args: FormatArgs<Map, P>,
  processor: FormatProcessor<Map>,
): string {
  const runFormat = <D extends keyof Map>(descriptor: D, value: unknown): string => {
    if (processor[descriptor] == null)
      throw new Error(`invalid format descriptor: ${String(descriptor)}`)
    return processor[descriptor](value as Map[D])
  }

  let result = ''

  let i = 0
  let argIndex = 0

  let percent: number
  while ((percent = format.indexOf('%', i)) !== -1) {
    const descriptor = format.charAt(percent + 1) as keyof Map

    result += format.slice(i, percent)
    if (descriptor === '%')
      result += '%'
    else
      result += runFormat(descriptor, args[argIndex++])
    i = percent + 2
  }
  result += format.slice(i)

  return result
}

export function gradleHomeDir(): string {
  return process.env.GRADLE_USER_HOME || path.join(homedir(), ".gradle")
}

export function asPair(str: string, sep: string | RegExp, laterIfNotFound: true): [string | undefined, string]
export function asPair(str: string, sep: string | RegExp, laterIfNotFound: false): [string, string | undefined]
export function asPair(str: string, sep: string | RegExp, laterIfNotFound: boolean): [string | undefined, string | undefined] {
  const {index: i, 0: {length: len}} = typeof sep == 'string' ? {index: str.indexOf(sep), 0: sep}
    : sep.exec(str) ?? { index: -1, 0: "" }
  if (i === -1) {
    return laterIfNotFound ? [undefined, str] : [str, undefined]
  }
  return [str.substr(0, i), str.substr(i + len)]
}

export function includes<T, Values extends T[]>(values: Values, value: T): value is Values[number] {
  return values.includes(value)
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function checkNever(never: never): void {
}

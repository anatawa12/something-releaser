/* eslint-disable @typescript-eslint/no-explicit-any */
import {homedir} from 'os'
import * as path from 'path'
import {FormatArgs, FormatProcessor} from './types'
export {
  Commit,
  GitClient, 
  RevWalker,
} from './utils/git-client'
export {
  asSequence, 
  asAsyncSequence, 
  AsyncSequence,
  Sequence, 
} from './utils/sequence'

export function headingAndLast<Heading extends any[], Last>(array: [...Heading, Last]): [Heading, Last]
export function headingAndLast<E>(array: E[]): [E[], E]
export function headingAndLast(array: any[]): any {
  return [array.slice(0, array.length - 1), array[array.length - 1]] as never
}

export function logicFailre(msg: string, never?: never): never {
  throw new Error(`logic failre: ${msg}: ${never}`)
}

export function processFormat<Map, P extends string>(
  format: P,
  args: FormatArgs<Map, P>,
  processor: FormatProcessor<Map>,
): string {
  const runFormat = <D extends keyof Map>(descriptor: D, value: unknown): string => {
    if (processor[descriptor] == null)
      throw new Error(`invalid format descriptor: ${descriptor}`)
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

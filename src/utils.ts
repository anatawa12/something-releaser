
export function headingAndLast<T>(array: T[]): [T[], T] {
  return [array.slice(0, array.length - 1), array[array.length - 1]]
}

export function logicFailre(msg: string, never?: never): never {
  throw new Error(`logic failre: ${msg}: ${never}`)
}

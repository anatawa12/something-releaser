// the groovy file support

import {FormatArgs, FormatProcessor} from '../types'
import {headingAndLast, processFormat} from '../utils'

export class GroovyGenerator {
  private readonly lines: string[]

  constructor() {
    this.lines = []
  }

  block<P extends string>(
    format: P,
    ...argsIn: [...FormatArgs<GroovyFormat, P>, (g: this) => void]
  ): this {
    const [args, func] = headingAndLast(argsIn)

    // no signature changes so as P is mostly safe
    this.line(`${format}` as P, ...args)
      .indent()
    func(this)
    this.dedent().line("}")
    return this
  }

  line<P extends string>(format: P, ...args: FormatArgs<GroovyFormat, P>): this {
    this.lines.push(processFormat(format, args, processor))
    return this
  }

  indent(): this {
    this.lines.push("<<indent+1>>")
    return this
  }

  dedent(): this {
    this.lines.push("<<indent-1>>")
    return this
  }

  toString(): string {
    let output = ''
    let indent = ''
    for (const line of this.lines) {
      if (line === "<<indent+1>>") {
        indent += '  '
      } else if (line === "<<indent-1>>") {
        indent = indent.slice(2)
      } else {
        output += `${indent}${line}\n`
      }
    }
    return output
  }
}

interface GroovyFormat {
  // literal
  l: number,
  // string literal
  s: string,
  // integer literal
  i: number,
  // raw code (for identifiers)
  n: string,
}

const processor: FormatProcessor<GroovyFormat> = {
  l: value => `${value}`,
  s: value => {
    if (!value.match(/[\t\n\r\f"'$\p{C}\\]/u))
      return `"${value}"`
    const body = value
      .split('')
      .map(c => {
        if (escapeMap[c] != null)
          return escapeMap[c]
        if (!c.match(/\p{C}/u))
          return c
        return `\\u${c.charCodeAt(0).toString(16).padStart(4, '0')}`
      })
      .join("")
    return `"${body}"`
  },
  i: value => {
    if (!Number.isInteger(value))
      throw new Error(`invalid %i: ${value} is not a integer`)
    return `${value}`
  },
  n: value => value,
}

const escapeMap: { [_ in string]?: string } = Object.create(null, {
  ['\b']: { value : '\\b' },
  ['\n']: { value : '\\n' },
  ['\r']: { value : '\\r' },
  ['\t']: { value : '\\t' },
  ['\\']: { value : '\\\\' }, 
  ['\'']: { value : '\\\'' }, 

  ['"']: { value : '\\"' },
  ['$']: { value : '\\$' },
})

// the java .properties file support library
// based on command/src/helpers/properties.rs@d9e6052c

import {StringBuilder} from '../types'
import {headingAndLast, logicFailre} from '../utils'

export class PropertiesFile {
  private readonly body: Element[]

  private constructor(body: Element[]) {
    this.body = body
  }

  static parse(source: string): PropertiesFile {
    if (source.length === 0)
      return new PropertiesFile([])
    const lines = source.split(/(?<=\r\n|\r|\n)/)
      .map((l) => l.replace(/(\r\n|\r|\n)$/, ""))
    return new PropertiesFile(parse_file(lines))
  }

  private find_pair(key: string): KeyValuePair | undefined {
    return this.body.find(
      (value): value is KeyValuePair =>
        value.type === "kvp" && value.key_parsed === key,
    )
  }

  get(key: string): string | undefined {
    const value = this.find_pair(key)?.value
    if (value == null) 
      return undefined
    return parse_value(value)
  }

  set(key: string, value: string): void {
    const pair = this.find_pair(key)
    if (pair != null) {
      pair.value = { type: "actual", value }
    } else {
      this.body.push({
        type: "kvp",
        key_parsed: key,
        blank: "",
        key: null,
        separator: "=",
        value: { type: "actual", value },
      })
    }
  }

  toSource(): string {
    return write_to_source(this.body)
  }

  static __test__new__(body: Element[]): PropertiesFile {
    return new PropertiesFile(body)
  }
}

type Element = KeyValuePair | SkipLine

type KeyValuePair<V extends Value = Value> = {
  type: "kvp";
  key_parsed: string;

  blank: string;
  key: string | null;
  separator: string;
  value: V;
}

type SkipLine = {
  type: "skip";
  line: string;
}

type Value = ParsedValue | ActualValue

type ParsedValue = {
  type: "parsed";
  lines: [blank: string, value: string][];
}

type ActualValue = {
  type: "actual";
  value: string;
}

//////////// parsing ////////////

class UnexpectedEOF extends Error {
  constructor() {
    super("unexpected EOF")
  }
}

class UnexpectedEOL extends Error {
  readonly line?: number
  constructor(line?: number) {
    super(line ? `unexpected EOL at line ${line}` : "unexpected EOL")
    this.line = line
  }

  withLine(line: number): UnexpectedEOL {
    return new UnexpectedEOL(line)
  }
}

class InvalidCharAt extends Error {
  readonly col: number
  readonly line?: number
  constructor(col: number, line?: number) {
    super(
      line
        ? `invalid char at line ${line} col ${col}`
        : `invalid char at ${col}`,
    )
    this.col = col
    this.line = line
  }

  withLine(line: number): UnexpectedEOL {
    return new InvalidCharAt(this.col, line)
  }
}

type EndValueLine<T> = ["end_value_line", T]
type ContinueLine<T> = ["continue_line", T]
type LineType<T> = EndValueLine<T> | ContinueLine<T>
type CommentOrBlankLine = ["comment_or_blank"]

type ParseLineSuccess = {
  // end of blank before key (exclusive) or start of key(inclusive)
  blank_end: number;
  // the parsed key value
  key: string;
  // end of key (exclusive) or separator blank or char(inclusive)
  key_end: number;
  // end of separator blank or char(exclusive) or start of value
  sep_end: number;
}

function key_value_pair_from(
  line: string,
  start: ParseLineSuccess,
): KeyValuePair<ParsedValue> {
  return {
    type: "kvp",
    blank: line.slice(0, start.blank_end),
    key: line.slice(start.blank_end, start.key_end),
    key_parsed: start.key,
    separator: line.slice(start.key_end, start.sep_end),
    value: {
      type: "parsed",
      lines: [["", line.slice(start.sep_end)]],
    },
  }
}

function parse_file(lines: Iterable<string>): Element[] {
  type State = ['start'] | ['continue', KeyValuePair<ParsedValue>]

  const elements: Element[] = []
  let state: State = ['start']

  function add_trailing(
    pair: KeyValuePair<ParsedValue>,
    blank: [blank: string, value: string],
  ): void {
    pair.value.lines.push(blank)
  }

  const line_num = 0
  for (const line of lines) {
    try {
      switch (state[0]) {
        case 'start': {
          const res = parse_key_value_line(line)
          switch (res[0]) {
            case 'comment_or_blank':
              elements.push({ type: 'skip', line })
              break
            case 'end_value_line':
              elements.push(key_value_pair_from(line, res[1]))
              break
            case 'continue_line':
              state = ['continue', key_value_pair_from(line.slice(-Infinity, -1), res[1])]
              break
            default:
              logicFailre("res", res[0])
          }
          break
        }
        case 'continue': {
          const res = parse_continuous_line(line)
          switch (res[0]) {
            case 'comment_or_blank':
              elements.push(state[1])
              elements.push({type: 'skip', line})
              state = ['start']
              break
            case 'end_value_line':
              add_trailing(state[1], res[1])
              elements.push(state[1])
              state = ['start']
              break
            case 'continue_line':
              add_trailing(state[1], res[1])
              break
            default:
              logicFailre("res", res[0])
          }
          break
        }
        default:
          logicFailre("state", state[0])
      }
    } catch (e) {
      if (e instanceof UnexpectedEOL || e instanceof InvalidCharAt)
        throw e.withLine(line_num)
      throw e
    }
  }

  switch (state[0]) {
    case 'start':
      return elements
    case 'continue':
      throw new UnexpectedEOF()
    default:
      logicFailre("state", state[0])
  }
}

function parse_key_value_line(
  line: string,
): LineType<ParseLineSuccess> | CommentOrBlankLine {
  type KeyState =
    | ["building", string]
    | ["before_sep", ParseLineSuccess]
    | ["after_sep", ParseLineSuccess]
    | ["built", ParseLineSuccess]

  let key_state: KeyState = ["building", ""] as KeyState

  // utils
  function mk_success(key: string, at: number): ParseLineSuccess {
    return {
      blank_end: 0,
      key,
      key_end: at,
      sep_end: 0,
    }
  }

  function end_sep(suc: ParseLineSuccess, at: number): ParseLineSuccess {
    return { ...suc, sep_end: at }
  }

  const result = parse_line(line, (escaped, c, i) => {
    switch (key_state[0]) {
      case "building":
        // if escaped character or non-separator char
        if (escaped || !" \t\f:=".includes(c)) 
          key_state[1] += c
        else if (":=".includes(c))
          key_state = ["after_sep", mk_success(key_state[1], i)]
        else 
          key_state = ["before_sep", mk_success(key_state[1], i)]
        break
      case "before_sep":
        if (!escaped && ":=".includes(c))
          key_state = ["after_sep", key_state[1]]
        else if (!escaped && " \t\f".includes(c))
          key_state = ["before_sep", key_state[1]]
        else 
          key_state = ["built", end_sep(key_state[1], i)]
        break
      case "after_sep":
        // a non-space chars: end of separator
        if (escaped || !" \t\f".includes(c))
          key_state = ["built", end_sep(key_state[1], i)]
        break
      case "built":
        // nop
        break
      default:
        logicFailre("state", key_state[0])
    }
  })

  // continue_line: remove last '\'
  const line_len =
    result[0] === "continue_line" ? line.length - 1 : line.length

  let success: ParseLineSuccess
  switch (key_state[0]) {
    case "building":
      success = end_sep(mk_success(key_state[1], line_len), line_len)
      break
    case "before_sep":
    case "after_sep":
      success = end_sep(key_state[1], line_len)
      break
    case "built":
      success = key_state[1]
      break
    default:
      logicFailre("state", key_state[0])
  }

  switch (result[0]) {
    case "end_value_line":
    case "continue_line":
      return [result[0], { ...success, blank_end: result[1] }]
    case "comment_or_blank":
      return ["comment_or_blank"]
    default:
      logicFailre("result", result)
  }
}

function parse_continuous_line(
  line_in: string,
): LineType<[blank: string, line: string]> | CommentOrBlankLine {
  const res = parse_line(line_in, () => {})
  switch (res[0]) {
    case 'continue_line':
      return ['continue_line', [line_in.substring(0, res[1]), line_in.substring(res[1], line_in.length - 1)]]
    case 'end_value_line':
      return ['end_value_line', [line_in.substring(0, res[1]), line_in.substring(res[1])]]
    case 'comment_or_blank':
      return ['comment_or_blank']
    default:
      logicFailre("res", res[0])
  }
}

// returns: blank_end
function parse_line(
  line: string,
  callback: (escape: boolean, char: string, index: number) => void,
): EndValueLine<number> | ContinueLine<number> | CommentOrBlankLine {
  const blank_end = line.match(/[^ \t\f\r\n]/)?.index
  // is a blank
  if (blank_end == null) 
    return ["comment_or_blank"]
  // starts with '#' or '!': is a comment line
  if ("#!".includes(line.charAt(blank_end))) 
    return ["comment_or_blank"]

  const result = parse_escape_sequence(line.substr(blank_end), callback)
  return [result[0], blank_end]
}

/**
 * public only for testing
 * @throws [UnexpectedEOL|InvalidCharAt]
 */
export function parse_escape_sequence(
  line: string,
  callback: (escape: boolean, char: string, index: number) => void,
): EndValueLine<[]> | ContinueLine<[]> {
  type State =
    | ["start"]
    | ["after_back_slash"]
    | ["after_bu0"]
    | ["after_bu1", number]
    | ["after_bu2", number]
    | ["after_bu3", number]
  let state: State = ["start"]
  let last_char_index = -1

  for (let i = 0; i < line.length; i++) {
    const ch = line.charAt(i)
    const is_escape = state[0] !== "start"
    let cur: string | undefined = undefined
    switch (state[0]) {
      case "start":
        if (ch === "\\") {
          state = ["after_back_slash"]
        } else {
          cur = ch
        }
        break
      case "after_back_slash": {
        const escape = (c: string): void => {
          state = ["start"]
          cur = c
        }
        switch (ch) {
          case "u":
            state = ["after_bu0"]
            break
          case "t":
            escape("\t")
            break
          case "r":
            escape("\r")
            break
          case "n":
            escape("\n")
            break
          case "f":
            escape("\f")
            break
          default:
            escape(ch)
            break
        }
        break
      }
      case "after_bu0":
        state = ["after_bu1", parseHex(ch, i)]
        break
      case "after_bu1":
        state = ["after_bu2", state[1] << 4 | parseHex(ch, i)]
        break
      case "after_bu2":
        state = ["after_bu3", state[1] << 4 | parseHex(ch, i)]
        break
      case "after_bu3":
        cur = String.fromCharCode(state[1] << 4 | parseHex(ch, i))
        break
      default:
        logicFailre("state", state[0])
    }
    if (cur) {
      callback(is_escape, cur, last_char_index + 1)
      last_char_index = i
    }
  }

  switch (state[0]) {
    case "start":
      return ["end_value_line", []]
    case "after_back_slash":
      return ["continue_line", []]
    case "after_bu0":
      throw new UnexpectedEOL()
    case "after_bu1":
      throw new UnexpectedEOL()
    case "after_bu2":
      throw new UnexpectedEOL()
    case "after_bu3":
      throw new UnexpectedEOL()
    default:
      logicFailre("state", state[0])
  }

  // utils
  function parseHex(ch: string, index: number): number {
    const value = parseInt(ch, 16)
    if (Number.isNaN(value)) {
      throw new InvalidCharAt(index)
    }
    return value
  }
}

// export only for testing
export function parse_value(value: Value): string {
  if (value.type === "actual") {
    return value.value
  }

  let output = ""

  function parse_escape_append(
    // eslint-disable-next-line no-shadow
    value: string,
  ): EndValueLine<[]> | ContinueLine<[]> {
    return parse_escape_sequence(value, (_, x) => {
      output += x
    })
  }

  const [heading, last] = headingAndLast(value.lines)
  for (const line of heading) {
    if (parse_escape_append(line[1])[0] !== "end_value_line") {
      logicFailre("unexpected end continue line")
    }
  }
  if (parse_escape_append(last[1])[0] !== "end_value_line") {
    logicFailre("unexpected end continue line")
  }
  return output
}

//////////// writing ////////////

function write_to_source(body: Element[]): string {
  const res = new StringBuilder()
  for (const elem of body) {
    switch (elem.type) {
      case 'kvp':
        write_key_value_pair(elem, res)
        break
      case 'skip':
        res.ln(elem.line)
        break
      default:
        logicFailre("elem", elem)
    }
  }
  return res.toString()
}

function write_key_value_pair(pair: KeyValuePair, res: StringBuilder): void {
  res.append(pair.blank)
  if (pair.key) {
    res.append(pair.key)
  } else {
    res.append(escapeProperties(pair.key_parsed))
  }
  res.append(pair.separator)
  switch (pair.value.type) {
    case 'parsed': {
      const [heading, last] = headingAndLast(pair.value.lines)

      for (const line of heading)
        res.append(line[0]).append(line[1]).ln('\\')
      res.append(last[0]).ln(last[1])
      break
    }
    case 'actual':
      res.ln(escapeProperties(pair.value.value))
      break
    default:
      logicFailre("pair.value", pair.value)
  }
}

function escapeProperties(str: string): string {
  // if no special chars found, no escape sequence
  if (!str.match(/[\\\t\n\r\f=:#!\p{C}]/u) && !str.startsWith(' ')) {
    return str
  }

  let first = true
  let res = ''
  for (const c of str) {
    if (first && c === ' ')
      res += '\\ '
    else if (ESCAPE_MAP[c])
      res += ESCAPE_MAP[c]
    else if (c.match(/[\p{C}]/u))
      res += `\\u${c.charCodeAt(0).toString(16).padStart(4, '0')}`
    else
      res += c
    first = false
  }
  return res
}

const ESCAPE_MAP: { [P in string]?: string } = Object.create(null, {
  ['=']: { value: "\\=" },
  [':']: { value: "\\:" },
  ['#']: { value: "\\#" },
  ['!']: { value: "\\!" },
  ['\t']: { value: "\\t" },
  ['\n']: { value: "\\n" },
  ['\r']: { value: "\\r" },
  ['\f']: { value: "\\f" },
  ['\\']: { value: "\\\\" },
})

// the json file support library

import {headingAndLast, logicFailre, StringBuilder} from '../utils'

export class JsonFile {
  private readonly headingSpace: string
  private readonly value: JsonValue
  private readonly tailingSpace: string

  private constructor(headingSpace: string, value: JsonValue, tailingSpace: string) {
    this.headingSpace = headingSpace
    this.value = value
    this.tailingSpace = tailingSpace
  }

  static parse(source: string): JsonFile {
    const tokenizer = new Tokenizer(source)
    const headingSpace = tokenizer.current().type === "headingSpace" ? tokenizer.move().value : ""
    const [value, tailingSpace] = parseJsonValue(tokenizer)
    return new JsonFile(headingSpace, value, tailingSpace)
  }

  toSource(minified = false): string {
    const builder = new StringBuilder()
    builder.append(this.headingSpace)
    appendValue(builder, this.value, minified)
    builder.append(this.tailingSpace)
    return builder.toString()
  }

  static __test__new__(headingSpace: string, value: JsonValue, tailingSpace: string): JsonFile {
    return new JsonFile(headingSpace, value, tailingSpace)
  }

  set(keys: JsonKey[], value: JsonPrimitiveValue): void {
    const [containerKey, valueKey] = headingAndLast(keys)
    const container = this.getObject(containerKey)
    if (container?.type === "object") {
      const keyString = `${valueKey}`
      if (typeof container.values == "string") {
        container.values = [{
          headingSpace: "",
          key: primitiveToJsonValue(keyString),
          separator: ": ",
          value: primitiveToJsonValue(value),
          tailingSpace: "",
        }]
        return
      }
      const found = container.values.find(v => parseString(v.key) === keyString)
      if (found == null) {
        const lastKVP = container.values[container.values.length - 1]
        container.values.push({
          headingSpace: lastKVP.headingSpace,
          key: primitiveToJsonValue(keyString),
          separator: lastKVP.separator,
          value: primitiveToJsonValue(value),
          tailingSpace: lastKVP.tailingSpace,
        })
        lastKVP.tailingSpace = ""
      } else {
        found.value = primitiveToJsonValue(value)
      }
    } else if (container?.type === "array") {
      if (typeof valueKey !== "number")
        return
      if (typeof container.values === "string") {
        container.values = [["", primitiveToJsonValue(value), ""]]
        return
      }
      const found = container.values[valueKey]
      if (found != null) {
        found[1] = primitiveToJsonValue(value)
      } else if (valueKey === container.values.length) {
        const last = container.values[container.values.length - 1]
        const secondLast: [string, {} | null, string] = container.values.length >= 2
          ? container.values[container.values.length - 2] : [last[0], null, ""]

        container.values.push([last[0], primitiveToJsonValue(value), last[2]])
        last[0] = secondLast[0]
        last[2] = secondLast[2]
      } else {
        throw new Error(`index out of range (${valueKey})`)
      }
    } else {
      throw new Error(`specified object not found (${container?.type})`)
    }
  }

  get(keys: JsonKey[]): JsonPrimitiveValue | undefined {
    const found = this.getObject(keys)
    if (found == null)
      return undefined

    switch (found.type) {
      case "string":
        return parseString(found)
      case "number":
        return found.parsed
      case "true":
      case "false":
      case "null":
        return found.value
      default:
        throw new Error(`the value of ${keys} is not primitive`)
    }
  }

  getObject(keys: JsonKey[]): JsonValue | undefined {
    let current = this.value
    for (const key of keys) {
      if (current.type === "object") {
        const keyString = `${key}`
        if (typeof current.values == "string")
          return undefined
        const found = current.values.find(v => parseString(v.key) === keyString)?.value
        if (found == null)
          return undefined
        current = found
      } else if (current.type === "array") {
        if (typeof key !== "number" || typeof current.values === "string")
          return undefined
        const found = current.values[key]
        if (found == null)
          return undefined
        current = found[1]
      } else {
        // non container
        return undefined
      }
    }
    return current
  }
}

function primitiveToJsonValue(value: string): JsonString
function primitiveToJsonValue(value: JsonPrimitiveValue): JsonString | JsonNumber | JsonLiteral
function primitiveToJsonValue(value: JsonPrimitiveValue): JsonString | JsonNumber | JsonLiteral {
  switch (typeof value) {
    case "string":
      return {type: "string", literal: JSON.stringify(value), parsed: value}
    case "number":
      return {type: "number", literal: JSON.stringify(value), parsed: value}
    case "boolean":
      return {type: `${value}`, value}
    case "object":
      if (value == null)
        return { type: "null", value: null }
      //fallthrough
    default:
      logicFailre(`type of value: ${value}`)
  }
}

function parseString(string: JsonString): string {
  return string.parsed ?? (string.parsed = parseLiteralString(string.literal))
}

export function parseLiteralString(literal: string): string {
  let builder = ""

  let since = 1
  let i: number
  while ((i = literal.indexOf("\\", since)) !== -1) {
    builder += literal.substring(since, i)
    const c = literal.charAt(i + 1)
    if (c === 'u') {
      builder += String.fromCharCode(Number.parseInt(literal.substring(i + 2, i + 2 + 4), 16))
      i += 2 + 4
    } else {
      i += 2
      // there's extra support for \' for json path
      builder += '\'"\\/\b\f\n\r\t'['\'"\\/bfnrt'.indexOf(c)]
    }
    since = i
  }

  if (since < literal.length - 1)
    builder += literal.substring(since, literal.length - 1)

  return builder
}

type JsonKey = string | number
type JsonPrimitiveValue = string | number | boolean | null

export type JsonValue = JsonObject | JsonArray | JsonString | JsonNumber | JsonLiteral

export interface JsonObject {
  readonly type: "object",
  values: JsonKVP[] | string
}

export interface JsonKVP {
  headingSpace: string,
  key: JsonString,
  separator: string,
  value: JsonValue,
  tailingSpace: string,
}

export interface JsonArray {
  readonly type: "array",
  values: [headingSpace: string, value: JsonValue, tailingSpace: string][] | string,
}

export interface JsonString {
  readonly type: "string",
  readonly literal: string,
  parsed?: string,
}

export interface JsonNumber {
  readonly type: "number",
  readonly literal: string,
  readonly parsed: number,
}

export interface JsonLiteral<V extends false | true | null = false | true | null> {
  readonly type: `${V}`,
  readonly value: V,
}

export function toStringJsonValue(value: JsonValue, minified = false): string {
  const builder = new StringBuilder()
  appendValue(builder, value, minified)
  return builder.toString()
}

function appendValue(builder: StringBuilder, value: JsonValue, minified = false): void {
  let first = true
  switch (value.type) {
    case "object":
      builder.append("{")
      if (typeof value.values == "string") {
        if (!minified)
          builder.append(value.values)
      } else {
        for (const kvp of value.values) {
          if (!first)
            builder.append(",")
          first = false
          if (!minified)
            builder.append(kvp.headingSpace)
          appendValue(builder, kvp.key)
          if (minified)
            builder.append(":")
          else
            builder.append(kvp.separator)
          appendValue(builder, kvp.value)
          if (!minified)
            builder.append(kvp.tailingSpace)
        }
      }
      builder.append("}")
      break
    case "array":
      builder.append("[")
      if (typeof value.values == "string") {
        if (!minified)
          builder.append(value.values)
      } else {
        for (const [headingSpace, element, tailingSpace] of value.values) {
          if (!first)
            builder.append(",")
          first = false
          if (!minified)
            builder.append(headingSpace)
          appendValue(builder, element)
          if (!minified)
            builder.append(tailingSpace)
        }
      }
      builder.append("]")
      break
    case "string":
    case "number":
      builder.append(value.literal)
      break
    case "true":
    case "false":
    case "null":
      builder.append(`${value.value}`)
      break
    default:
      logicFailre(`type`, value)
  }
}

//////////// parsing ////////////

class ParsingError extends Error {
  readonly col: number
  readonly line: number

  constructor(msg: string, col: number, line: number) {
    super(`${msg} at line ${line} col ${col}`)
    this.col = col
    this.line = line
  }
}

// the value of headingSpace is space and tailingSpace will be empty
type TokenType =
  "headingSpace" | `${"open" | "close"}${"Brace" | "Bracket"}` | `${"string" | "number"}Literal`
  | "colon" | "comma" | "true" | "false" | "null"

type JsonToken = { type: TokenType, value: string, tailingSpace: string }

function parseJsonValue(tokenizer: Tokenizer): readonly [value: JsonValue, tailingSpace: string] {
  const token = tokenizer.current()
  switch (token.type) {
    case "openBrace":
      return parseObject(tokenizer)
    case "openBracket":
      return parseArray(tokenizer)
    case "stringLiteral":
      return parseJsonString(tokenizer)
    case "numberLiteral":
      return parseJsonNumber(tokenizer)
    case "true":
      return parseJsonTrueLiteral(tokenizer)
    case "false":
      return parseJsonFalseLiteral(tokenizer)
    case "null":
      return parseJsonNullLiteral(tokenizer)
    default:
      tokenizer.unexpectedToken("openBrace", token)
  }
}

function parseObject(tokenizer: Tokenizer): readonly [value: JsonObject, tailingSpace: string] {
  let token = tokenizer.move()
  if (token.type !== "openBrace")
    tokenizer.unexpectedToken("openBrace", token)
  let headingSpace = token.tailingSpace
  token = tokenizer.current()
  if (token.type === "closeBrace") {
    return [
      {
        type: "object",
        values: headingSpace,
      },
      token.tailingSpace,
    ]
  }

  const values: JsonKVP[] = []

  while (true) {
    const [key, separator0] = parseJsonString(tokenizer)
    token = tokenizer.move()
    if (token.type !== "colon")
      tokenizer.unexpectedToken("colon", token)
    const {value: separator1, tailingSpace: separator2} = token

    const [value, tailingSpace] = parseJsonValue(tokenizer)

    values.push({
      headingSpace,
      key,
      separator: separator0 + separator1 + separator2,
      value,
      tailingSpace,
    })

    token = tokenizer.move()
    headingSpace = token.tailingSpace
    if (token.type === "closeBrace")
      return [
        {
          type: "object",
          values,
        },
        headingSpace,
      ]
    if (token.type !== "comma")
      tokenizer.unexpectedToken("closeBrace or comma", token)
  }
}

function parseArray(tokenizer: Tokenizer): readonly [value: JsonArray, tailingSpace: string] {
  let token = tokenizer.move()
  if (token.type !== "openBracket")
    tokenizer.unexpectedToken("openBracket", token)
  let headingSpace = token.tailingSpace
  token = tokenizer.current()
  if (token.type === "closeBracket") {
    return [
      {
        type: "array",
        values: headingSpace,
      },
      token.tailingSpace,
    ]
  }

  const values: [string, JsonValue, string][] = []

  while (true) {
    const [value, tailingSpace] = parseJsonValue(tokenizer)

    values.push([headingSpace, value, tailingSpace])

    token = tokenizer.move()
    headingSpace = token.tailingSpace
    if (token.type === "closeBracket")
      return [
        {
          type: "array",
          values,
        },
        headingSpace,
      ]
    if (token.type !== "comma")
      tokenizer.unexpectedToken("closeBracket or comma", token)
  }
}

function singleTokenParser<Token>(
  token: TokenType,
  tokens: (value: string) => Token,
): (tokenizer: Tokenizer) => readonly [value: Token, tailingSpace: string] {
  return (tokenizer) => {
    const {type, value, tailingSpace} = tokenizer.move()
    if (type !== token)
      tokenizer.unexpectedToken(token, {type, value, tailingSpace})
    return [tokens(value), tailingSpace]
  }
}

const parseJsonString = singleTokenParser<JsonString>("stringLiteral", (literal) => ({type: "string", literal}))
const parseJsonNumber = singleTokenParser<JsonNumber>("numberLiteral", (literal) => ({
  type: "number", 
  parsed: Number(literal),
  literal, 
}))
const parseJsonFalseLiteral = singleTokenParser<JsonLiteral<false>>("false", () => ({type: "false", value: false}))
const parseJsonTrueLiteral = singleTokenParser<JsonLiteral<true>>("true", () => ({type: "true", value: true}))
const parseJsonNullLiteral = singleTokenParser<JsonLiteral<null>>("null", () => ({type: "null", value: null}))

class Tokenizer {
  private readonly string: string
  private index: number

  constructor(string: string) {
    this.string = string
    this.index = 0
  }

  private cur: JsonToken | null = null

  current(): JsonToken {
    if (this.cur != null)
      return this.cur
    this.cur = this.computeNext()
    return this.cur
  }

  // returns current and move to next
  move(): JsonToken {
    const cur = this.current()
    this.cur = null
    return cur
  }

  computeNext(): JsonToken {
    if (this.index === 0) {
      const ws = this.skipWhiteSpace()
      if (ws !== "")
        return {
          type: "headingSpace",
          value: ws,
          tailingSpace: "",
        }
    }
    const begin = this.index
    const type = this.computeNextType()
    const value = this.string.substring(begin, this.index)
    const tailingSpace = this.skipWhiteSpace()

    return {type, value, tailingSpace}
  }

  private computeLineCol(c: string): [line: number, col: number] {
    if (this.string[this.index] === c) {
      if (this.string[this.index + 1] !== c)
        logicFailre("line col target char not found")
      this.computeLineNumberAt(this.index + 1)
    }
    return this.computeLineNumberAt(this.index)
  }

  private computeLineNumberAt(index: number): [line: number, col: number] {
    const lines = this.string.substring(0, index + 1).split(/\r|\n|\r\n/)
    return [lines.length, lines[lines.length - 1].length]
  }

  private invalidCharToken(c: string, starting: string, expect: string): never {
    throw new ParsingError(`invalid char at token starting with ${starting}: ${c}. expected ${expect}`,
      ...this.computeLineCol(c))
  }

  private invalidChar(c: string, expect: string): never {
    if (c.charCodeAt(0) <= 0x20) {
      throw new ParsingError(`invalid char: '\\x${c.charCodeAt(0).toString(16).padStart(2, '0')}'. expected ${expect}`, ...this.computeLineCol(c))
    } else {
      throw new ParsingError(`invalid char: '${c}'. expected ${expect}`, ...this.computeLineCol(c))
    }
  }

  private computeNextType(): TokenType {
    let c = this.string[this.index++]
    const oneCharToken = Tokenizer.oneCharTokens[c]
    if (oneCharToken != null)
      return oneCharToken
    switch (c) {
      case 't':
        if ((c = this.string[this.index++]) !== 'r')
          this.invalidCharToken(c, "t", "true")
        if ((c = this.string[this.index++]) !== 'u')
          this.invalidCharToken(c, "tr", "true")
        if ((c = this.string[this.index++]) !== 'e')
          this.invalidCharToken(c, "tru", "true")
        return 'true'
      case 'f':
        if ((c = this.string[this.index++]) !== 'a')
          this.invalidCharToken(c, "f", "false")
        if ((c = this.string[this.index++]) !== 'l')
          this.invalidCharToken(c, "fa", "false")
        if ((c = this.string[this.index++]) !== 's')
          this.invalidCharToken(c, "fal", "false")
        if ((c = this.string[this.index++]) !== 'e')
          this.invalidCharToken(c, "fals", "false")
        return 'false'
      case 'n':
        if ((c = this.string[this.index++]) !== 'u')
          this.invalidCharToken(c, "n", "null")
        if ((c = this.string[this.index++]) !== 'l')
          this.invalidCharToken(c, "nu", "null")
        if ((c = this.string[this.index++]) !== 'l')
          this.invalidCharToken(c, "nul", "null")
        return 'null'
      case '-':
        if (!"123456789".includes(c = this.string[this.index++]))
          this.invalidChar(c, "digit")
      // fallthrough
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
      case '6':
      case '7':
      case '8':
      case '9':
        while ("0123456789".includes(this.string[this.index]))
          this.index++
      // fallthrough
      case '0':
        if (this.string[this.index] === '.') {
          this.index++ // skip '.'
          if (!"0123456789".includes(c = this.string[this.index++]))
            this.invalidChar(c, "digit")
          while ("0123456789".includes(this.string[this.index]))
            this.index++
        }

        if (this.string[this.index] === 'e' || this.string[this.index] === 'E') {
          this.index++ // skip 'e' or 'E'
          if ("-+".includes(this.string[this.index]))
            this.index++ // skip '+' or '-'
          if (!"0123456789".includes(c = this.string[this.index++]))
            this.invalidChar(c, "digit")
          while ("0123456789".includes(this.string[this.index]))
            this.index++
        }

        return "numberLiteral"

      case '"':
        while (true) {
          c = this.string[this.index++]
          if (c === '"') {
            break
          } else if (c === '\\') {
            // TODO: escape sequence
            switch (c = this.string[this.index++]) {
              case '"':
              case '\\':
              case '/':
              case 'b':
              case 'f':
              case 'n':
              case 'r':
              case 't':
                // single char escape
                break
              case 'u':
                // unicode escape. expects 4 hex
                if (!"0123456789abcdefABCDEF".includes(c = this.string[this.index++]))
                  this.invalidChar(c, "hex (escape sequence)")
                if (!"0123456789abcdefABCDEF".includes(c = this.string[this.index++]))
                  this.invalidChar(c, "hex (escape sequence)")
                if (!"0123456789abcdefABCDEF".includes(c = this.string[this.index++]))
                  this.invalidChar(c, "hex (escape sequence)")
                if (!"0123456789abcdefABCDEF".includes(c = this.string[this.index++]))
                  this.invalidChar(c, "hex (escape sequence)")
                break
              default:
                this.invalidChar(c, "escape sequence")
            }
          } else if (20 <= c.charCodeAt(0)) {
            // nop: valid char for string literal
          } else {
            this.invalidChar(c, "non control character")
          }
        }
        return "stringLiteral"

      default:
        this.invalidChar(c, "some token")
    }
  }

  private skipWhiteSpace(): string {
    const begin = this.index
    while ("\x20\x0a\x0d\x09".includes(this.string[this.index]))
      this.index++
    return this.string.substring(begin, this.index)
  }

  private static readonly oneCharTokens: { [P in string]?: TokenType } = {
    "[": "openBracket",
    "]": "closeBracket",
    "{": "openBrace",
    "}": "closeBrace",
    ",": "comma",
    ":": "colon",
  }

  unexpectedToken(expected: string, {type, value, tailingSpace}: JsonToken): never {
    if (type === "headingSpace")
      logicFailre(`headingSpace is not expected`)
    const tokenBegin = this.index - tailingSpace.length - value.length
    const shouldBeToken = this.string.substring(tokenBegin, tokenBegin + value.length)
    if (shouldBeToken !== value)
      logicFailre("token is not expected")

    throw new ParsingError(`unexpected token: ${type}, expected ${expected}`,
      ...this.computeLineNumberAt(tokenBegin))
  }
}

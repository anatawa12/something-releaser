import {promises as fs} from 'fs'
import {JsonFile, parseLiteralString, toStringJsonValue} from "../files/json"
import {PropertiesFile} from '../files/properties'
import {includes, throws} from '../utils'

export async function fileUtil(args: string[]): Promise<void> {
  switch (args[0]) {
    case 'properties':
      await propertiesUtil(args.slice(1))
      break
    case 'json':
      await jsonUtil(args.slice(1))
      break
    default:
      throw new Error(`unknown file format: ${args[0]}`)
  }
}

async function propertiesUtil(args: string[]): Promise<void> {
  const file = args[1] ?? throws(new Error(`no file specified`))
  const property = args[2] ?? throws(new Error(`no property specified`))
  switch (args[0]) {
    case 'get': {
      const propertiesFile = PropertiesFile.parse(await fs.readFile(file, {encoding:'utf-8'}))
      const value = propertiesFile.get(property)
      process.stdout.write(value ?? '')
      if (value == null)
        process.exit(1)
      break
    }
    case 'set': {
      const value = args[3] ?? throws(new Error(`no property specified`))
      const propertiesFile = PropertiesFile.parse(await fs.readFile(file, {encoding:'utf-8'}))
      propertiesFile.set(property, value)
      await fs.writeFile(file, propertiesFile.toSource())
      break
    }
    default:
      throw new Error(`unknown command for properties: ${args[0]}`)
  }
}

async function jsonUtil(args: string[]): Promise<void> {
  const file = args[1] ?? throws(new Error(`no file specified`))
  const jsonPath = parseJsonPath(args[2] ?? throws(new Error(`no property specified`)))

  switch (args[0]) {
    case 'get': {
      const options = parseOptions(args.slice(3), "--minified", "--primitive")
      const propertiesFile = JsonFile.parse(await fs.readFile(file, {encoding:'utf-8'}))
      if (options["--primitive"]) {
        const value = propertiesFile.get(jsonPath)
        if (value == null)
          process.exit(1)
        process.stdout.write(`${value}`)
      } else {
        const value = propertiesFile.getObject(jsonPath)
        if (value == null)
          process.exit(1)
        process.stdout.write(toStringJsonValue(value, options["--minified"] != null))
      }
      break
    }
    case 'set': {
      const options = parseOptions(args.slice(4), "--minified", "--string", 
        "--number", "--bool", "--null")
      if (!checkNoneOrSingle(options, "--string", "--number", "--bool", "--null"))
        throw new Error(`two ore more --string, --number, --bool, or --null is not allowed`)
      const value = args[3] ?? throws(new Error(`no property specified`))

      const propertiesFile = JsonFile.parse(await fs.readFile(file, {encoding:'utf-8'}))

      if (options["--null"]) {
        propertiesFile.set(jsonPath, null)
      } else if (options["--string"]) {
        propertiesFile.set(jsonPath, value)
      } else if (options["--number"]) {
        const num = parseFloat(value)
        if (isNaN(num))
          throw new Error(`un-parsable number value: ${num}`)
        propertiesFile.set(jsonPath, num)
      } else if (options["--bool"]) {
        propertiesFile.set(jsonPath, Boolean(value))
      } else {
        let parsedNum: number
        if (value === "null") {
          propertiesFile.set(jsonPath, null)
        } else if (value === "false") {
          propertiesFile.set(jsonPath, false)
        } else if (value === "true") {
          propertiesFile.set(jsonPath, true)
        } else if (!isNaN(parsedNum = parseFloat(value))) {
          propertiesFile.set(jsonPath, parsedNum)
        } else {
          propertiesFile.set(jsonPath, value)
        }
      }
      await fs.writeFile(file, propertiesFile.toSource(options["--minified"] != null))
      break
    }
    default:
      throw new Error(`unknown command for properties: ${args[0]}`)
  }

  function parseJsonPath(s: string): (string | number)[] {
    // append heading '.'
    if (!s.startsWith('.') && !s.startsWith('[') && !s.startsWith('$')) {
      s = `.${s}`
    }
    const keys: (string | number)[] = []
    let i = 0
    const invalidErr = (msg: string, offset = 0): never => {
      throw new Error(`invalid json path at ${i + offset} (${msg})`)
    }
    // skip heading $.
    if (s[i] === '$') {
      i++
    }

    while (i < s.length) {
      if (s[i] === '[') {
        i++
        if (s[i] === '"' || s[i] === "'") {
          const since = i
          const endChar = s[i++]
          loop: while (true) {
            switch (s[i++]) {
              case '\\':
                switch (s[i++]) {
                  case '"':
                  case "'":
                  case 'b':
                  case 'f':
                  case 'n':
                  case 'r':
                  case 't':
                    break
                  case 'u':
                    if (!"0123456789ABCDEFabcdef".includes(s[i++])
                      || !"0123456789ABCDEFabcdef".includes(s[i++])
                      || !"0123456789ABCDEFabcdef".includes(s[i++])
                      || !"0123456789ABCDEFabcdef".includes(s[i++]))
                      invalidErr("invalid escape sequence", -1)
                    break
                  default:
                    invalidErr("invalid escape sequence", -1)
                }
                break
              case '"':
              case "'":
                if (s[i - 1] === endChar)
                  break loop
                // fallthrough
              default:
                // nop
                break
              case undefined:
                invalidErr("unexpected eop in string literal", -1)
            }
          }
          keys.push(parseLiteralString(s.substring(since, i)))
        } else if ("0123456789".includes(s[i])) {
          const since = i
          while (i < s.length && "0123456789".includes(s[i]))
            i++
          keys.push(parseInt(s.substring(since, i)))
        } else {
          throw invalidErr(`unexpected char at start of brackets`)
        }
      } else if (s[i] === ".") {
        i++
        const since = i
        while (s[i] !== '.' && s[i] !== '[')
          i++
        const part = s.substring(since, i)
        const int = parseInt(part)
        if (isNaN(int)) {
          keys.push(part)
        } else {
          keys.push(int)
        }
      } else {
        throw invalidErr(`unexpected char`)
      }
    }
    return keys
  }
}

function parseOptions<Options extends string[]>(args: string[], ...options: Options): { [P in Options[number]]?: string | true } {
  const result: { [P in Options[number]]?: string | true } = {}
  for (const arg of args) {
    const pair = arg.split('=', 2)
    if (!includes(options, pair[0]))
      throw new Error(`unexpected option: ${pair[0]}`)
    if (pair.length === 2) {
      result[pair[0]] = pair[1]
    } else {
      result[pair[0]] = true
    }
  }
  return result
}

function checkNoneOrSingle(options: { [p: string]: string | true | undefined }, ...list: string[]): boolean {
  let cnt = 0
  for (const string of list) {
    if (options[string] != null)
      cnt++
  }
  return cnt >= 2
}

import {expect, test, describe} from '@jest/globals'
import {parse_escape_sequence, parse_value, PropertiesFile} from '../../src/files/properties'

describe("parse and write file", () => {
  function props(arg: Parameters<(typeof PropertiesFile)["__test__new__"]>[0]): PropertiesFile {
    return PropertiesFile.__test__new__(arg)
  }

  function run_parse_and_print(str: string, value: PropertiesFile): void {
    const file = PropertiesFile.parse(str)
    expect(file).toEqual(value)
    expect(file.toSource()).toBe(str)
  }

  test('empty', async () => {
    run_parse_and_print("", props([]))
  })

  test('new line only', async () => {
    run_parse_and_print("\n", props([{type: 'skip', line: ""}]))
  })

  test('blank', async () => {
    run_parse_and_print("    \n", props([{type: 'skip', line: "    "}]))
  })

  test('simple kvp', async () => {
    run_parse_and_print(
      "hello=world\n",
      props([
        {
          type: 'kvp',
          key_parsed: "hello",
          blank: "",
          key: "hello",
          separator: "=",
          value: {
            type: 'parsed',
            lines: [
              ["", "world"],
            ],
          },
        },
      ]),
    )
  })

  test('newline separator', async () => {
    run_parse_and_print(
      "hello\\\nworld\n",
      props([
        {
          type: 'kvp',
          key_parsed: "hello",
          blank: "",
          key: "hello",
          separator: "",
          value: {
            type: 'parsed',
            lines: [
              ["", ""],
              ["", "world"],
            ],
          },
        },
      ]),
    )
  })

  test('separator with speace', async () => {
    run_parse_and_print(
      "hello  =  world\n",
      props([
        {
          type: 'kvp',
          key_parsed: "hello",
          blank: "",
          key: "hello",
          separator: "  =  ",
          value: {
            type: 'parsed',
            lines: [
              ["", "world"],
            ],
          },
        },
      ]),
    )
  })

  test('colon separated', async () => {
    run_parse_and_print(
      "hello:world\n",
      props([
        {
          type: 'kvp',
          key_parsed: "hello",
          blank: "",
          key: "hello",
          separator: ":",
          value: {
            type: 'parsed',
            lines: [
              ["", "world"],
            ],
          },
        },
      ]),
    )
  })

  test('comment line', async () => {
    run_parse_and_print(
      "!comment\n",
      props([{type: 'skip', line: "!comment"}]),
    )
    run_parse_and_print(
      "#comment\n",
      props([{type: 'skip', line: "#comment"}]),
    )
    run_parse_and_print(
      "  #comment\n",
      props([{type: 'skip', line: "  #comment"}]),
    )
  })

  test("complicated", async () => {
    run_parse_and_print(
      `org.gradle.jvmargs=-Xmx1024m
# channel
#
# debug, snapshot, public
CHANNEL=snapshot
#
# versions
#
VERSIONS=1.5.7\\
     ,1.5.6\\
     ,1.5.5
#
# other flags
#
# enable optifine debugging
ENABLE_\\:OPTIFINE=false
`,
      props([
        {
          type: 'kvp',
          key_parsed: "org.gradle.jvmargs",
          blank: "",
          key: "org.gradle.jvmargs",
          separator: "=",
          value: {
            type: 'parsed',
            lines: [
              ["", "-Xmx1024m"],
            ],
          },
        },
        {type: 'skip', line: "# channel"},
        {type: 'skip', line: "#"},
        {type: 'skip', line: "# debug, snapshot, public"},
        {
          type: 'kvp',
          key_parsed: "CHANNEL",
          blank: "",
          key: "CHANNEL",
          separator: "=",
          value: {
            type: 'parsed',
            lines: [
              ["", "snapshot"],
            ],
          },
        },
        {type: 'skip', line: "#"},
        {type: 'skip', line: "# versions"},
        {type: 'skip', line: "#"},
        {
          type: 'kvp',
          key_parsed: "VERSIONS",
          blank: "",
          key: "VERSIONS",
          separator: "=",
          value: {
            type: 'parsed',
            lines: [
              ["", "1.5.7"],
              ["     ", ",1.5.6"],
              ["     ", ",1.5.5"],
            ],
          },
        },
        {type: 'skip', line: "#"},
        {type: 'skip', line: "# other flags"},
        {type: 'skip', line: "#"},
        {type: 'skip', line: "# enable optifine debugging"},
        {
          type: 'kvp',
          key_parsed: "ENABLE_:OPTIFINE",
          blank: "",
          key: "ENABLE_\\:OPTIFINE",
          separator: "=",
          value: {
            type: 'parsed',
            lines: [
              ["", "false"],
            ],
          },
        },
      ]),
    )
  })
})

describe("parse_value", () => {
  test("actual", () => {
    expect(parse_value({type: 'actual', value: "value"}))
      .toBe("value")
  })

  test("single line simple", () => {
    expect(parse_value({type: 'parsed', lines: [[' ', 'value']]}))
      .toBe('value')
  })

  test("multi line simple", () => {
    expect(parse_value({type: 'parsed', lines: [[' ', 'line1'], ['', 'line2']]}))
      .toBe('line1line2')
  })

  test("escape sequence", () => {
    expect(parse_value({type: 'parsed', lines: [[' ', '\\r\\n\\f\\:\\a']]}))
      .toBe('\r\n\f:a')
  })
})

describe("parse_escape_sequence", () => {
  function tester(
    line: string, 
    expect_result: "end_value_line"| "continue_line",
    expect_call: [escape: boolean, char: string, index: number][],
  ): void {
    const call: [escape: boolean, char: string, index: number][] = []
    const res = parse_escape_sequence(line, (...args) => call.push(args))

    expect(call).toEqual(expect_call)
    expect(res).toEqual([expect_result, []])
  }

  test("simple line", () => {
    tester("", 'end_value_line', [])
    tester("\\", 'continue_line', [])
    tester("hello", 'end_value_line', [
      [false, 'h', 0],
      [false, 'e', 1],
      [false, 'l', 2],
      [false, 'l', 3],
      [false, 'o', 4],
    ])
    tester("hello\\", 'continue_line', [
      [false, 'h', 0],
      [false, 'e', 1],
      [false, 'l', 2],
      [false, 'l', 3],
      [false, 'o', 4],
    ])
  })

  test("c-style escape", () => {
    tester("\\t\\r\\n\\f", 'end_value_line', [
      [true, '\t', 0],
      [true, '\r', 2],
      [true, '\n', 4],
      [true, '\f', 6],
    ])
    tester("\\t\\r\\n\\f\\", 'continue_line', [
      [true, '\t', 0],
      [true, '\r', 2],
      [true, '\n', 4],
      [true, '\f', 6],
    ])
  })

  test("same-char escape", () => {
    tester("\\a\\b\\:\\!\\ ", 'end_value_line', [
      [true, 'a', 0],
      [true, 'b', 2],
      [true, ':', 4],
      [true, '!', 6],
      [true, ' ', 8],
    ])
    tester("\\a\\b\\:\\!\\ \\", 'continue_line', [
      [true, 'a', 0],
      [true, 'b', 2],
      [true, ':', 4],
      [true, '!', 6],
      [true, ' ', 8],
    ])
  })
})

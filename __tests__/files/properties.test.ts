import {expect, test} from '@jest/globals'
import {PropertiesFile} from '../../src/files/properties'

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

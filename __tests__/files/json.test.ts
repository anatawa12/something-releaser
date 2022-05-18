import {expect, test, describe} from '@jest/globals'
import {parseLiteralString, JsonFile} from '../../src/files/json'

describe("parse and write file", () => {
  function json(...args: Parameters<(typeof JsonFile)["__test__new__"]>): JsonFile {
    return JsonFile.__test__new__(...args)
  }

  function run_parse_and_print(str: string, value: JsonFile): void {
    const file = JsonFile.parse(str)
    expect(file).toEqual(value)
    expect(file.toSource()).toBe(str)
  }

  test('minified', async () => {
    run_parse_and_print(`{"array":[0],"string":"string","number":1,"true":true,"false":false,"null":null}`, json("", {
      type: 'object',
      values: [
        {
          headingSpace: "",
          key: {type: "string", literal: `"array"`},
          separator: ":",
          value: {type: "array", values: [["", {type: "number", parsed: 0, literal: "0"}, ""]]},
          tailingSpace: "",
        },
        {
          headingSpace: "",
          key: {type: "string", literal: `"string"`},
          separator: ":",
          value: {type: "string", literal: `"string"`},
          tailingSpace: "",
        },
        {
          headingSpace: "",
          key: {type: "string", literal: `"number"`},
          separator: ":",
          value: {type: "number", parsed: 1, literal: "1"},
          tailingSpace: "",
        },
        {
          headingSpace: "",
          key: {type: "string", literal: `"true"`},
          separator: ":",
          value: {type: "true", value: true},
          tailingSpace: "",
        },
        {
          headingSpace: "",
          key: {type: "string", literal: `"false"`},
          separator: ":",
          value: {type: "false", value: false},
          tailingSpace: "",
        },
        {
          headingSpace: "",
          key: {type: "string", literal: `"null"`},
          separator: ":",
          value: {type: "null", value: null},
          tailingSpace: "",
        },
      ],
    }, ""))
  })

  test('styled', async () => {
    run_parse_and_print(`
    {
      "array": [
        0
      ],
      "string": "string",
      "number": 1,
      "true": true,
      "false": false,
      "null": null
    }
    `, json("\n    ", {
      type: 'object',
      values: [
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"array"`},
          separator: ": ",
          value: {type: "array", values: [["\n        ", {type: "number", parsed: 0, literal: "0"}, "\n      "]]},
          tailingSpace: "",
        },
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"string"`},
          separator: ": ",
          value: {type: "string", literal: `"string"`},
          tailingSpace: "",
        },
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"number"`},
          separator: ": ",
          value: {type: "number", parsed: 1, literal: "1"},
          tailingSpace: "",
        },
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"true"`},
          separator: ": ",
          value: {type: "true", value: true},
          tailingSpace: "",
        },
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"false"`},
          separator: ": ",
          value: {type: "false", value: false},
          tailingSpace: "",
        },
        {
          headingSpace: "\n      ",
          key: {type: "string", literal: `"null"`},
          separator: ": ",
          value: {type: "null", value: null},
          tailingSpace: "\n    ",
        },
      ],
    }, "\n    "))
  })
})

describe("parseString", () => {
  test("simple literal", () => {
    expect(parseLiteralString(`"the key"`)).toBe("the key")
  })

  test("escaped", () => {
    expect(parseLiteralString(`"\\"\\\\\\/\\b\\f\\n\\r\\t`)).toBe(`"\\/\b\f\n\r\t`)
  })

  test("hex escaped", () => {
    expect(parseLiteralString(`"\\u0123`)).toBe(`\u0123`)
  })

  test("mixed", () => {
    expect(parseLiteralString(`"A> hello \\"\\u0123\\"! how are you?\\nB> I'm good."`))
      .toBe(`A> hello "\u0123"! how are you?\nB> I'm good.`)
  })
})

describe("setting value and output then", () => {
  test("keep saved value", () => {
    const props = JsonFile.parse("{}")
    props.set(["the key"], "value")
    expect(props.get(["the key"])).toBe("value")
  })

  test("read value", () => {
    const props = JsonFile.parse(`{"the key": "value", "the key2": 100}`)
    expect(props.get(["the key"])).toBe("value")
    expect(props.get(["the key2"])).toBe(100)
  })

  test("write value", () => {
    const props = JsonFile.parse("{}")
    props.set(["the key"], "value")
    expect(props.toSource()).toBe(`{"the key": "value"}`)
  })

  test("indented adding value of object", () => {
    const props = JsonFile.parse(`{\n  "key0": "value0"\n}`)
    props.set(["the key"], "value1")
    expect(props.toSource()).toBe(`{\n  "key0": "value0",\n  "the key": "value1"\n}`)
  })

  test("indented adding value of array", () => {
    const props = JsonFile.parse(`[\n  "value0"\n]`)
    props.set([1], "value1")
    expect(props.toSource()).toBe(`[\n  "value0",\n  "value1"\n]`)
  })
})

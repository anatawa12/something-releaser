import {it, expect} from '@jest/globals'
import {asPair} from '../../src/utils'

it('later test', () => {
  function test(str: string, sep: string | RegExp, expected: (string|undefined)[]): void {
    expect(asPair(str, sep, true))
      .toEqual(expected)
  }

  test("gradle@java", "@", ["gradle", "java"])
  test("gradle@java", ":", [undefined, "gradle@java"])
  test("props:gradle", "@", [undefined, "props:gradle"])
  test("props:gradle", ":", ["props", "gradle"])
  test("props:gradle@java", "@", ["props:gradle", "java"])
  test("props:gradle@java", ":", ["props", "gradle@java"])

  test("props:gradle", /[:@]/, ["props", "gradle"])
  test("props@java", /[:@]/, ["props", "java"])
  test("props@java", /:|(?=@)/, ["props", "@java"])
  test("props:gradle@java", /:|(?=@)/, ["props", "gradle@java"])
})

it('early test', () => {
  function test(str: string, sep: string | RegExp, expected: (string|undefined)[]): void {
    expect(asPair(str, sep, false))
      .toEqual(expected)
  }

  test("gradle@java", "@", ["gradle", "java"])
  test("gradle@java", ":", ["gradle@java", undefined])
  test("props:gradle", "@", ["props:gradle", undefined])
  test("props:gradle", ":", ["props", "gradle"])
  test("props:gradle@java", "@", ["props:gradle", "java"])
  test("props:gradle@java", ":", ["props", "gradle@java"])

  test("props:gradle", /[:@]/, ["props", "gradle"])
  test("props@java", /[:@]/, ["props", "java"])
  test("props@java", /:|(?=@)/, ["props", "@java"])
  test("props:gradle@java", /:|(?=@)/, ["props", "gradle@java"])
})

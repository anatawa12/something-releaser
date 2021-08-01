import {describe, it, expect} from '@jest/globals'
import {Version} from '../../src/utils/version'

describe('construct', () =>  {
  it('with config', () => {
    function test(
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      snapshot: boolean,
    ): void {
      const config = {major, minor, patch, snapshot}
      expect(new Version(config))
        .toEqual(config)
    }
    test(1, undefined, undefined, false)
    test(1, 0, undefined, false)
    test(1, 1, undefined, false)
    test(1, 1, 0, false)
    test(1, 1, 1, false)
    test(1, undefined, undefined, true)
    test(1, 0, undefined, true)
    test(1, 1, undefined, true)
    test(1, 1, 0, true)
    test(1, 1, 1, true)
  })

  it('with numbers', () => {
    function test(
      instance: Version,
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      snapshot: boolean,
    ): void {
      const config = {major, minor, patch, snapshot}
      expect(instance).toEqual(config)
    }

    test(new Version(1, 0, 0),
      1, 0, 0, false)
    test(new Version(1, 0, 0, false),
      1, 0, 0, false)
    test(new Version(1, 0, 0, true),
      1, 0, 0, true)

    test(new Version(1, 0),
      1, 0, undefined, false)
    test(new Version(1, 0, false),
      1, 0, undefined, false)
    test(new Version(1, 0, true),
      1, 0, undefined, true)

    test(new Version(1, 1),
      1, 1, undefined, false)
    test(new Version(1, 1, false),
      1, 1, undefined, false)
    test(new Version(1, 1, true),
      1, 1, undefined, true)

    test(new Version(1),
      1, undefined, undefined, false)
    test(new Version(1, false),
      1, undefined, undefined, false)
    test(new Version(1, true),
      1, undefined, undefined, true)
  })

  it('parse', () => {
    function test(
      value: string,
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      snapshot: boolean,
    ): void {
      const config = {major, minor, patch, snapshot}
      expect(Version.parse(value)).toEqual(config)
    }

    test("1.0.0",
      1, 0, 0, false)
    test("1.0.0-SNAPSHOT",
      1, 0, 0, true)

    test("1.0",
      1, 0, undefined, false)
    test("1.0-SNAPSHOT",
      1, 0, undefined, true)

    test("1.1",
      1, 1, undefined, false)
    test("1.1-SNAPSHOT",
      1, 1, undefined, true)

    test("1",
      1, undefined, undefined, false)
    test("1-SNAPSHOT",
      1, undefined, undefined, true)
  })
})

describe('toString', () => {
  it('toString', () => {
    expect(`${new Version(1)}`)
      .toEqual('1')
    expect(`${new Version(1, true)}`)
      .toEqual('1-SNAPSHOT')

    expect(`${new Version(1, 0)}`)
      .toEqual('1.0')
    expect(`${new Version(1, 1)}`)
      .toEqual('1.1')
    expect(`${new Version(1, 0, true)}`)
      .toEqual('1.0-SNAPSHOT')
    expect(`${new Version(1, 1, true)}`)
      .toEqual('1.1-SNAPSHOT')

    expect(`${new Version(1, 0, 0)}`)
      .toEqual('1.0.0')
    expect(`${new Version(1, 1, 0)}`)
      .toEqual('1.1.0')
    expect(`${new Version(1, 0, 1)}`)
      .toEqual('1.0.1')
    expect(`${new Version(1, 1, 1)}`)
      .toEqual('1.1.1')
    expect(`${new Version(1, 0, 0, true)}`)
      .toEqual('1.0.0-SNAPSHOT')
    expect(`${new Version(1, 1, 0, true)}`)
      .toEqual('1.1.0-SNAPSHOT')
    expect(`${new Version(1, 0, 1, true)}`)
      .toEqual('1.0.1-SNAPSHOT')
    expect(`${new Version(1, 1, 1, true)}`)
      .toEqual('1.1.1-SNAPSHOT')
  })
})

describe('utilities', () => {
  it('unSnapshot', () => {
    expect(new Version(1, 0).unSnapshot())
      .toEqual(new Version(1, 0))
    expect(new Version(1, 0, true).unSnapshot())
      .toEqual(new Version(1, 0))
  })

  it('makeSnapshot', () => {
    expect(new Version(1, 0).makeSnapshot())
      .toEqual(new Version(1, 0, true))
    expect(new Version(1, 0, true).makeSnapshot())
      .toEqual(new Version(1, 0, true))
  })

  it('next', () => {
    expect(new Version(1).next())
      .toEqual(new Version(2))

    expect(new Version(1, true).next())
      .toEqual(new Version(2, true))

    expect(new Version(1, 0).next())
      .toEqual(new Version(1, 1))

    expect(new Version(1, 0, true).next())
      .toEqual(new Version(1, 1, true))

    expect(new Version(1, 0, 0).next())
      .toEqual(new Version(1, 0, 1))

    expect(new Version(1, 0, 0, true).next())
      .toEqual(new Version(1, 0, 1, true))
  })
})

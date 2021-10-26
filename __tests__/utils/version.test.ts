import {describe, it, expect} from '@jest/globals'
import {Release, Version} from '../../src/utils/version'

describe('construct', () =>  {
  it('with config', () => {
    function test(
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      release: Release,
    ): void {
      const config = {major, minor, patch, release}
      expect(new Version(config))
        .toEqual(config)
    }
    test(1, undefined, undefined, ['stable'])
    test(1, 0, undefined, ['stable'])
    test(1, 1, undefined, ['stable'])
    test(1, 1, 0, ['stable'])
    test(1, 1, 1, ['stable'])
    test(1, undefined, undefined, ['snapshot'])
    test(1, 0, undefined, ['snapshot'])
    test(1, 1, undefined, ['snapshot'])
    test(1, 1, 0, ['snapshot'])
    test(1, 1, 1, ['snapshot'])
  })

  it('with numbers', () => {
    function test(
      instance: Version,
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      release: Release,
    ): void {
      const config = {major, minor, patch, release}
      expect(instance).toEqual(config)
    }

    test(new Version(1, 0, 0),
      1, 0, 0, ['stable'])
    test(new Version(1, 0, 0, ['stable']),
      1, 0, 0, ['stable'])
    test(new Version(1, 0, 0, ['snapshot']),
      1, 0, 0, ['snapshot'])

    test(new Version(1, 0),
      1, 0, undefined, ['stable'])
    test(new Version(1, 0, ['stable']),
      1, 0, undefined, ['stable'])
    test(new Version(1, 0, ['snapshot']),
      1, 0, undefined, ['snapshot'])

    test(new Version(1, 1),
      1, 1, undefined, ['stable'])
    test(new Version(1, 1, ['stable']),
      1, 1, undefined, ['stable'])
    test(new Version(1, 1, ['snapshot']),
      1, 1, undefined, ['snapshot'])

    test(new Version(1),
      1, undefined, undefined, ['stable'])
    test(new Version(1, ['stable']),
      1, undefined, undefined, ['stable'])
    test(new Version(1, ['snapshot']),
      1, undefined, undefined, ['snapshot'])
  })

  it('parse', () => {
    function test(
      value: string,
      major: number,
      minor: number | undefined,
      patch: number | undefined,
      release: Release,
    ): void {
      const config = {major, minor, patch, release}
      expect(Version.parse(value)).toEqual(config)
    }

    test("1.0.0",
      1, 0, 0, ['stable'])
    test("1.0.0-SNAPSHOT",
      1, 0, 0, ['snapshot'])
    test("1.0.0-alpha1",
      1, 0, 0, ['alpha', 1])
    test("1.0.0-beta2",
      1, 0, 0, ['beta', 2])
    test("1.0.0-rc3",
      1, 0, 0, ['candidate', 3])

    test("1.0",
      1, 0, undefined, ['stable'])
    test("1.0-SNAPSHOT",
      1, 0, undefined, ['snapshot'])
    test("1.0-alpha1",
      1, 0, undefined, ['alpha', 1])
    test("1.0-beta2",
      1, 0, undefined, ['beta', 2])
    test("1.0-rc3",
      1, 0, undefined, ['candidate', 3])

    test("1.1",
      1, 1, undefined, ['stable'])
    test("1.1-SNAPSHOT",
      1, 1, undefined, ['snapshot'])
    test("1.1-alpha1",
      1, 1, undefined, ['alpha', 1])
    test("1.1-beta2",
      1, 1, undefined, ['beta', 2])
    test("1.1-rc3",
      1, 1, undefined, ['candidate', 3])

    test("1",
      1, undefined, undefined, ['stable'])
    test("1-SNAPSHOT",
      1, undefined, undefined, ['snapshot'])
    test("1-alpha1",
      1, undefined, undefined, ['alpha', 1])
    test("1-beta2",
      1, undefined, undefined, ['beta', 2])
    test("1-rc3",
      1, undefined, undefined, ['candidate', 3])
  })
})

describe('toString', () => {
  it('toString', () => {
    expect(`${new Version(1)}`)
      .toEqual('1')
    expect(`${new Version(1, ['snapshot'])}`)
      .toEqual('1-SNAPSHOT')
    expect(`${new Version(1, ['alpha', 1])}`)
      .toEqual('1-alpha1')
    expect(`${new Version(1, ['beta', 2])}`)
      .toEqual('1-beta2')
    expect(`${new Version(1, ['candidate', 3])}`)
      .toEqual('1-rc3')

    expect(`${new Version(1, 0)}`)
      .toEqual('1.0')
    expect(`${new Version(1, 1)}`)
      .toEqual('1.1')
    expect(`${new Version(1, 0, ['snapshot'])}`)
      .toEqual('1.0-SNAPSHOT')
    expect(`${new Version(1, 1, ['snapshot'])}`)
      .toEqual('1.1-SNAPSHOT')
    expect(`${new Version(1, 0, ['alpha', 1])}`)
      .toEqual('1.0-alpha1')
    expect(`${new Version(1, 1, ['alpha', 1])}`)
      .toEqual('1.1-alpha1')
    expect(`${new Version(1, 0, ['beta', 2])}`)
      .toEqual('1.0-beta2')
    expect(`${new Version(1, 1, ['beta', 2])}`)
      .toEqual('1.1-beta2')
    expect(`${new Version(1, 0, ['candidate', 3])}`)
      .toEqual('1.0-rc3')
    expect(`${new Version(1, 1, ['candidate', 3])}`)
      .toEqual('1.1-rc3')

    expect(`${new Version(1, 0, 0)}`)
      .toEqual('1.0.0')
    expect(`${new Version(1, 1, 0)}`)
      .toEqual('1.1.0')
    expect(`${new Version(1, 0, 1)}`)
      .toEqual('1.0.1')
    expect(`${new Version(1, 1, 1)}`)
      .toEqual('1.1.1')
    expect(`${new Version(1, 0, 0, ['snapshot'])}`)
      .toEqual('1.0.0-SNAPSHOT')
    expect(`${new Version(1, 1, 0, ['snapshot'])}`)
      .toEqual('1.1.0-SNAPSHOT')
    expect(`${new Version(1, 0, 1, ['snapshot'])}`)
      .toEqual('1.0.1-SNAPSHOT')
    expect(`${new Version(1, 1, 1, ['snapshot'])}`)
      .toEqual('1.1.1-SNAPSHOT')
    expect(`${new Version(1, 0, 0, ['alpha', 1])}`)
      .toEqual('1.0.0-alpha1')
    expect(`${new Version(1, 1, 0, ['alpha', 1])}`)
      .toEqual('1.1.0-alpha1')
    expect(`${new Version(1, 0, 1, ['alpha', 1])}`)
      .toEqual('1.0.1-alpha1')
    expect(`${new Version(1, 1, 1, ['alpha', 1])}`)
      .toEqual('1.1.1-alpha1')
    expect(`${new Version(1, 0, 0, ['beta', 2])}`)
      .toEqual('1.0.0-beta2')
    expect(`${new Version(1, 1, 0, ['beta', 2])}`)
      .toEqual('1.1.0-beta2')
    expect(`${new Version(1, 0, 1, ['beta', 2])}`)
      .toEqual('1.0.1-beta2')
    expect(`${new Version(1, 1, 1, ['beta', 2])}`)
      .toEqual('1.1.1-beta2')
    expect(`${new Version(1, 0, 0, ['candidate', 3])}`)
      .toEqual('1.0.0-rc3')
    expect(`${new Version(1, 1, 0, ['candidate', 3])}`)
      .toEqual('1.1.0-rc3')
    expect(`${new Version(1, 0, 1, ['candidate', 3])}`)
      .toEqual('1.0.1-rc3')
    expect(`${new Version(1, 1, 1, ['candidate', 3])}`)
      .toEqual('1.1.1-rc3')
  })
})

describe('utilities', () => {
  it('makeStable', () => {
    expect(new Version(1, 0).makeStable())
      .toEqual(new Version(1, 0))
    expect(new Version(1, 0, ['snapshot']).makeStable())
      .toEqual(new Version(1, 0))
  })

  it('makeSnapshot', () => {
    expect(new Version(1, 0).makeSnapshot())
      .toEqual(new Version(1, 0, ['snapshot']))
    expect(new Version(1, 0, ['snapshot']).makeSnapshot())
      .toEqual(new Version(1, 0, ['snapshot']))
  })

  it('next', () => {
    expect(new Version(1).next())
      .toEqual(new Version(2))

    expect(new Version(1, ['snapshot']).next())
      .toEqual(new Version(2, ['snapshot']))

    expect(new Version(1, 0).next())
      .toEqual(new Version(1, 1))

    expect(new Version(1, 0, ['snapshot']).next())
      .toEqual(new Version(1, 1, ['snapshot']))

    expect(new Version(1, 0, 0).next())
      .toEqual(new Version(1, 0, 1))

    expect(new Version(1, 0, 0, ['snapshot']).next())
      .toEqual(new Version(1, 0, 1, ['snapshot']))
  })
})

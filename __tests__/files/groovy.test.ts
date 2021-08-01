import {expect, test} from '@jest/globals'
import {GroovyGenerator} from '../../src/files/groovy'

test("string literal", () => {
  expect(new GroovyGenerator().line("%s", "hello\nworld").toString())
    .toBe('"hello\\nworld"\n')
})

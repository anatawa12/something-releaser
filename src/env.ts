import {readFileSync} from "fs"
import Ajv from 'ajv'
import {Env as EnvJson} from './generated/env'
import envSchema from './generated/env.json'
import {logicFailre} from './utils'
import {
  ChangerDescriptor, 
  parseDescriptor as parseVersionDescriptor,
} from './version-changer'

const ajv = new Ajv()
const tester = ajv.compile<EnvJson>(envSchema)

type Props = { [P in keyof Env]-?: PropDesc<P, Env[P]> }

type Helper<T> = T extends undefined ? undefined : never

type PropDesc<P extends keyof EnvJson, T> =
  & JsonPropDesc<P, T>
  & ({} | EnvPropDesc<T>)
  & (undefined extends Helper<Env[P]> ? { optional: true } : { optional: false })

type JsonPropDesc<P extends keyof EnvJson, T> = {
  parse: (value: EnvJson[P]) => T
}

type EnvPropDesc<T> = {
  env: string,
  parseEnv: (value: string) => T
}

const propKeys = [
  'releaseChanger',
] as const

type SameType<A, B> = A extends B ? B extends A ? true : false: false

// type check: propKeys have same values as keyof Env
/* eslint-disable @typescript-eslint/no-unused-vars */
// noinspection JSUnusedLocalSymbols 
const _test0: SameType<(typeof propKeys)[number], keyof Env> = true
/* eslint-restore @typescript-eslint/no-unused-vars */

const props: Props = {
  releaseChanger: {
    parse: parseReleaseChanger,
    env: 'RELEASER_CHANGER',
    parseEnv: (v: string) => v.split(';').map(parseVersionDescriptor),
    optional: false,
  },
}

const jsonFile = JSON.parse(readConfigJson())

if (!tester(jsonFile)) {
  throw new Error(`invalid .something-releaser.json: \n${tester.errors?.join("\n")}`)
}

export interface Env {
  releaseChanger: ChangerDescriptor[]
}

const parsedJson: Partial<Env> = {}

for (const key of propKeys) {
  if (jsonFile[key])
    parsedJson[key] = props[key].parse(jsonFile[key])
}

const env: Env = {} as Env

for (const key of propKeys) {
  const prop = props[key]

  Object.defineProperty(env, key, {
    enumerable: true,
    get() {
      // first check parsedJson[key]
      if (parsedJson[key])
        return parsedJson[key]
      if ('env' in prop) {
        const envValue = process.env[prop.env]
        if (envValue)
          return prop.parseEnv(envValue)
      }
      if (!prop.optional) {
        if ('env' in prop)
          throw new Error(`either environment variable '${prop.env}' or`
            + ` '${key}' in json must be specified`)
        else
          throw new Error(`'${key}' in json must be specified`)
      }
      return undefined
    },
  })
}

export default env

function readConfigJson(): string {
  try {
    return readFileSync('.something-releaser.json', 'utf8')
  } catch (e) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((e as any).code === 'ENOENT')
      return '{}'
    throw e
  }
}

function parseReleaseChanger(value: EnvJson['releaseChanger']): ChangerDescriptor[] {
  let values: EnvJson['releaseChanger'] & unknown[]
  if (Array.isArray(value)) {
    values = value
  } else if (typeof value == 'string') {
    values = value.split(';')
  } else {
    logicFailre("unknown value", value)
  }

  return values.map((elem): ChangerDescriptor => {
    if (Array.isArray(elem)) {
      return {changer: elem[0], info: elem[1], path: elem[2]}
    } else if (typeof elem == 'string') {
      return parseVersionDescriptor(elem)
    } else if (typeof elem == 'object' && elem != null) {
      return {changer: elem.changer, info: elem.info, path: elem.path}
    } else {
      logicFailre("unknown value", elem)
    }
  })
}

import * as core from '@actions/core'
import {load as loadYaml} from 'js-yaml'
import {promises} from 'fs'
import Ajv from 'ajv'
import schemaJson from '../generated/schema.json'
import {KeyOfValue, Yaml} from '../types'

const ajv = new Ajv({allErrors: true})
const schema = ajv.compile(schemaJson)

export async function parseConfig(configPath: string): Promise<Yaml> {
  const config_yaml = loadYaml(
    await promises.readFile(configPath, {encoding: 'utf8'})
  )
  if (!ajv.validate(schema, config_yaml)) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    for (const error of ajv.errors!) {
      core.error(`Config: At ${error.instancePath}: ${error.message}`)
    }
    throw new Error('config parsing error.')
  }
  // because validated by ajv, config is matched to Schema.
  const config = config_yaml as Yaml

  extractVariables(config)
  return config
}

function extractVariables(config: Yaml): void {
  const config_ = extractor(config)
  config_.e('git-user')
  config_('version-changer')('gradle-properties').e('property').e('path')
  config_('publish-environment')('gradle-maven')
    .e('repo')
    .e('sign')
    .e('maven-user')
    .e('maven-pass')
    .e('gpg-key')
  config_('publish-environment')('gradle-plugin-portal').e('key').e('sercret')
  config_('publish-environment')('gradle-intellij-publisher').e('token')
  // publish-command is
  //config_.e('publish-command')
}

function extractVariable<P extends string | number | symbol>(
  base: {[_ in P]?: string} | undefined,
  key: P,
  path: string,
): void {
  if (base == null) 
    return
  let value = base[key]
  if (value == null) 
    return

  value = value.replace(/\$({[^}]*}?|\w+)/, function (str) {
    if (str.startsWith('${') && !str.endsWith('}')) {
      throw new Error(
        `Config: At ${path}: invalid variable reference.` +
          'this supports $variable_name or ${variable_name}',
      )
    }
    // get variable name
    const name = str.startsWith('${')
      ? str.substring(2, str.length - 2)
      : str.substr(1)
    const varValue = process.env[name]
    if (varValue == null) {
      throw new Error(`Config: At ${path}: variable \`${name}\` not found.`)
    }
    return varValue
  })
  base[key] = value
}

// variable extractor

interface Extractor<Obj> {
  <Key extends keyof Obj>(key: Key): Extractor<NonNullable<Obj[Key]>>
  e<Key extends KeyOfValue<Obj, string>>(key: Key): this
  v: Obj
}

/* eslint-disable @typescript-eslint/no-explicit-any */
const NOP_EXTRACTOR: Extractor<any> = (() => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const r = function (_: any): Extractor<any> {
    return NOP_EXTRACTOR
  } as Extractor<any>
  r.e = function () {
    return this
  }
  return r
})()
/* eslint-enable */

function extractor<Obj>(obj: Obj): Extractor<Obj> {
  // eslint-disable-next-line no-shadow
  function impl<Obj>(value: Obj, path: string): Extractor<Obj> {
    const r = function <Key extends keyof Obj>(key: Key): Extractor<Obj[Key]> {
      if (value == null) 
        return NOP_EXTRACTOR
      return impl(value[key], `${path}.${key}`)
    } as Extractor<Obj>
    r.e = function <Key extends KeyOfValue<Obj, string>>(key: Key) {
      extractVariable(value, key, path)
      return this
    }
    r.v = value
    return r
  }
  return impl(obj, '')
}

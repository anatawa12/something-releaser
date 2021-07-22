import {GitHub} from '@actions/github/lib/utils'

export type Octokit = InstanceType<typeof GitHub>

export {Schema as Yaml} from './generated/yaml';

export type ObjectMap<Key extends string | number | symbol, Value> = {
  [_ in Key]: Value
}

export type KeyOfValue<Object, Value> = keyof (Object &
  ObjectMap<keyof Object, Value>)

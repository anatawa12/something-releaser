import {Yaml} from '../types'
import {GradleMaven} from './gradle-maven'

export interface Configurator {
  configure(): Promise<void>
  toString(): string
}

export class Configurators {
  private readonly configurators: Configurator[]

  constructor(configurators: Configurator[]) {
    this.configurators = configurators
    if (configurators.length === 0)
      throw new Error("invalid version configurators: empty")
  }

  async configure(): Promise<void> {
    for (const configurator of this.configurators) {
      try {
        await configurator.configure()
      } catch (e) {
        throw new Error(`configuring via ${configurator.toString()}: ${e}`)
      }
    }
  }
}

export function createFromJson(
  config: Yaml['publish-environment'],
): Configurators {
  const result: Configurator[] = []

  result.push(...GradleMaven.create(config['gradle-maven']))

  return new Configurators(result)
}

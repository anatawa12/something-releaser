import {Configurators} from '../publish-environment'

export async function configureForPublish(configurators: Configurators): Promise<void> {
  await configurators.configure()
}

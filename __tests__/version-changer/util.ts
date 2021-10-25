import {ChangerDescriptor} from '../../src/version-changer'

export function creator(changer: string): (info?: string, path?: string) => ChangerDescriptor {
  return (info, path) => ({changer, info, path})
}

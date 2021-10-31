import {main as commandMain} from './command'

export function main(...args: string[]): void {
  Promise.resolve()
    .then(async () => commandMain(...args))
    .catch(e => {
      // eslint-disable-next-line no-console
      console.error(e)
      process.exit(-1)
    })
}

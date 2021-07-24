import * as child_process from 'child_process'
import {SpawnOptions} from 'child_process'
import * as process from 'process'
import {Stream} from 'stream'

// eslint-disable-next-line @typescript-eslint/promise-function-async
export function spawn(cmd: string, args: string[], options: SpawnOptions): Promise<number> {
  // *** Return the promise
  return new Promise(function (resolve, reject) {
    const child = child_process.spawn(cmd, args, {
      ...options,
      stdio: ['pipe', 'pipe', 'pipe'],
    })
    child.stdio[0].end()
    Stream.prototype.pipe.call(child.stdio[1], process.stdout)
    Stream.prototype.pipe.call(child.stdio[2], process.stderr)
    child.on('close', resolve)
    child.on('error', reject)
  })
}

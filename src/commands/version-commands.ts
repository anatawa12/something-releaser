import * as fs from "fs"
import {logicFailre} from "../utils"
import {Version} from "../utils/version"

function eprintln(body: string): void {
  // eslint-disable-next-line no-console
  console.warn(body)
}

type Command<Name, Args extends string[] = []> = [cmd: Name, ver: string, ...args: Args]

export type VersionCommand =
  | Command<'version-unsnapshot'> // deprecated
  | Command<'version-stable'>
  | Command<'version-snapshot'>
  | Command<'version-alpha', [] | [num: string]>
  | Command<'version-beta', [] | [num: string]>
  | Command<'version-candidate', [] | [num: string]>
  | Command<'version-major'>
  | Command<'version-minor'>
  | Command<'version-patch'>
  | Command<'version-get-channel'>
  | Command<'version-set-channel', [channel: string, num: string] | [channel: string]>
  | Command<'version-next', [] | [channel: string]>
  | Command<'version-format'>

type DropVersion<Ary extends unknown[]> = Ary extends [infer Name, unknown, ...infer Args] ? [Name, ...Args] : never

export type VersionCommandArgs = DropVersion<VersionCommand>

export async function runVersionCommands(args: VersionCommand): Promise<void> {
  const [versionName, cmdArgs] = sliceVersion(args)
  let version
  if (!versionName || versionName === '-')
    version = Version.parse(fs.readFileSync(process.stdin.fd, 'utf-8'))
  else
    version = Version.parse(versionName)
  const result = doVersionCommand(version, cmdArgs)
  // eslint-disable-next-line no-console
  console.log(result.toString())
}

export function doVersionCommand(version: Version, args: VersionCommandArgs): Version | string {
  switch (args[0]) {
    case 'version-unsnapshot':
      eprintln("version-unsnapshot is deprecated. use version-stable")
      return version.makeStable()
    case 'version-stable':
      return version.makeStable()
    case 'version-snapshot':
      return version.makeSnapshot()
    case 'version-alpha':
      return version.makeAlpha(parseInt(args[1] ?? '1'))
    case 'version-beta':
      return version.makeBeta(parseInt(args[1] ?? '1'))
    case 'version-candidate':
      return version.makeCandidate(parseInt(args[1] ?? '1'))
    case 'version-major':
      return version.makeMajorOnly()
    case 'version-minor':
      return version.makeMajorMinor()
    case 'version-patch':
      return version.makeMajorMinorPatch()
    case 'version-get-channel':
      return version.release[0]
    case 'version-set-channel': {
      switch (args[1].toLowerCase()) {
        case 'a':
        case 'alpha':
        case 'α':
          version = version.makeAlpha(parseInt(args[2] ?? '1'))
          break
        case 'b':
        case 'beta':
        case 'β':
          version = version.makeBeta(parseInt(args[2] ?? '1'))
          break
        case 'rc':
        case 'candidate':
          version = version.makeCandidate(parseInt(args[2] ?? '1'))
          break
        case 'snapshot':
          version = version.makeSnapshot()
          break
        case 'stable':
          version = version.makeStable()
          break
        default:
          throw new Error(`unknown release channel: ${args[1]}`)
      }
      return version
    }
    case 'version-next': {
      let target: "prerelease" | "patch" | "minor" | "major" | null
      switch (args[1]) {
        case null:
        case undefined:
          target = null
          break
        case "pre":
        case "prerelease":
        case 'a':
        case 'alpha':
        case 'α':
        case 'b':
        case 'beta':
        case 'β':
        case 'rc':
        case 'candidate':
        case 'snapshot':
          target = "prerelease"
          break
        case "pat":
        case "patch":
          target = "patch"
          break
        case "min":
        case "minor":
          target = "minor"
          break
        case "maj":
        case "major":
          target = "major"
          break
        default:
          throw new Error(`unknown next version target: ${args[1]}`)
      }
      return version.next(target)
    }
    case "version-format":
      return version
    default:
      logicFailre("invalid version command", args[0])
  }
}

function sliceVersion<Ary extends unknown[]>(args: Ary): [Ary[1], DropVersion<Ary>] {
  return [args[1], [args[0], ...args.slice(2)] as DropVersion<Ary>]
}

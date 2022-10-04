import {throws, checkNever} from "../utils"
import {Version} from "../utils/version"

type Command<Name, Args extends string[] = []> = [cmd: Name, ver: string, ...args: Args]

function println(body: string): void {
  // eslint-disable-next-line no-console
  console.log(body)
}

function eprintln(body: string): void {
  // eslint-disable-next-line no-console
  console.warn(body)
}

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

export async function runVersionCommands(args: VersionCommand): Promise<void> {
  switch (args[0]) {
    case 'version-unsnapshot': {
      eprintln("version-unsnapshot is deprecated. use version-stable")
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeStable()
        .toString())
      break
    }
    case 'version-stable': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeStable()
        .toString())
      break
    }
    case 'version-snapshot': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeSnapshot()
        .toString())
      break
    }
    case 'version-alpha': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeAlpha(parseInt(args[2] ?? '1'))
        .toString())
      break
    }
    case 'version-beta': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeBeta(parseInt(args[2] ?? '1'))
        .toString())
      break
    }
    case 'version-candidate': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeCandidate(parseInt(args[2] ?? '1'))
        .toString())
      break
    }
    case 'version-major': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeMajorOnly()
        .toString())
      break
    }
    case 'version-minor': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeMajorMinor()
        .toString())
      break
    }
    case 'version-patch': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .makeMajorMinorPatch()
        .toString())
      break
    }
    case 'version-get-channel': {
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .release[0])
      break
    }
    case 'version-set-channel': {
      let version = Version.parse(args[1]
        ?? throws(new Error('version name required')))
      switch (args[2].toLowerCase()) {
        case 'a':
        case 'alpha':
        case 'α':
          version = version.makeAlpha(parseInt(args[3] ?? '1'))
          break
        case 'b':
        case 'beta':
        case 'β':
          version = version.makeBeta(parseInt(args[3] ?? '1'))
          break
        case 'rc':
        case 'candidate':
          version = version.makeCandidate(parseInt(args[3] ?? '1'))
          break
        case 'snapshot':
          version = version.makeSnapshot()
          break
        case 'stable':
          version = version.makeStable()
          break
        default:
          throw new Error(`unknown release channel: ${args[2]}`)
      }
      println(version.toString())
      break
    }
    case 'version-next': {
      let target: "prerelease" | "patch" | "minor" | "major" | null
      switch (args[2]) {
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
          throw new Error(`unknown next version target: ${args[2]}`)
      }
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .next(target)
        .toString())
      break
    }
    case "version-format": {
      println(Version.parse(args[1] ?? throws(new Error('version name required'))).toString())
      break
    }
    default:
      checkNever(args[0])
      throw new Error(`unknown command: ${args[0]}`)
  }
}

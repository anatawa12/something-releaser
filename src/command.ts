import {promises as fs} from 'fs'
import * as path from 'path'
import * as core from '@actions/core'
import {Octokit} from '@octokit/rest'
import {run as autoChangelog} from 'auto-changelog/src/run'
import {setGitUser} from './commands/git-user'
import {GradleIntellij} from './commands/gradle-intellij'
import {GradleMaven} from './commands/gradle-maven'
import {GradlePluginPortal} from './commands/gradle-plugin-portal'
import {GradleSigning} from './commands/gradle-signing'
import {publishToCurseForge} from './commands/publish-to-curse-forge'
import {publishToMaven} from './commands/publish-to-maven'
import {sendTweet} from './commands/send-tweet'
import env from './env'
import {Version} from './utils'
import {createChangers} from './version-changer'

function throws(error: Error): never {
  throw error
}

function println(body: string): void {
  // eslint-disable-next-line no-console
  console.log(body)
}

function eprintln(body: string): void {
  // eslint-disable-next-line no-console
  console.warn(body)
}

type Command =
  | ['something-releaser', ...string[]]
  | ['install', ...([string] | [string, string] | [] )]
  | ['set-git-user', string]
  | ['get-version']
  | ['set-version', string]
  | ['version-unsnapshot', string] // deprecated
  | ['version-stable', string]
  | ['version-snapshot', string]
  | ['version-alpha', ...([ver: string, num: string] | [ver: string])]
  | ['version-beta', ...([ver: string, num: string] | [ver: string])]
  | ['version-candidate', ...([ver: string, num: string] | [ver: string])]
  | ['version-get-channel', string]
  | ['version-set-channel', ...([ver: string, channel: string, num: string] | [ver: string, channel: string])]
  | ['version-next', string]
  | ['generate-changelog', ...string[]]
  | ['prepare-gradle-maven', string, ...string[]]
  | ['prepare-gradle-signing', string, ...string[]]
  | ['prepare-gradle-plugin-portal', string, string]
  | ['prepare-gradle-intellij', string]
  | ['publish-to-curse-forge', ...string[]]
  | ['publish-to-maven', ...string[]]
  | ['send-tweet', ...string[]]

export async function main(...args: string[]): Promise<void> {
  return await mainImpl(...args as Command)
}

async function trueOrENOENT(promise: Promise<unknown>): Promise<boolean> {
  try {
    await promise
    return true
  } catch (e) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((e as any).code === 'ENOENT')
      return false
    throw e
  }
}

const ghTokenPath = path.join(__dirname, 'gh-token')

async function mainImpl(...args: Command): Promise<void> {
  switch (args[0]) {
    case 'something-releaser': {
      await main(...args.slice(1))
      break
    }
    case 'install': {
      // if not found, find
      if (!await trueOrENOENT(fs.stat(ghTokenPath))) {
        const githubToken = args[1]
          || core.getInput('token')
          || throws(new Error('github token not found.'))
        await fs.writeFile(ghTokenPath, githubToken, 'utf8')
      }
      const isWin = process.platform === 'win32'
      const installTo = isWin
        ? path.join(__dirname, '..', 'path-win')
        : path.join(__dirname, '..', 'path-posix')
      if (process.env.GITHUB_PATH) {
        core.addPath(installTo)
      } else {
        println(`Please add ${installTo} to your path variable`)
      }
      break
    }
    case 'set-git-user': {
      const octokit = new Octokit({auth: await fs.readFile(ghTokenPath, 'utf8')})
      const user = args[1] ?? throws(new Error(`user name required`))
      await setGitUser(user, octokit)
      break
    }
    case 'get-version': {
      const changers = createChangers(env.releaseChanger)
      println((await changers.getVersionName()).toString())
      break
    }
    case 'set-version': {
      const changers = createChangers(env.releaseChanger)
      const version = Version.parse(args[1]
        ?? throws(new Error(`version name required`)))
      await changers.setVersionName(version)
      break
    }
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
      println(Version.parse(args[1]
        ?? throws(new Error('version name required')))
        .next()
        .toString())
      break
    }
    case 'generate-changelog': {
      await autoChangelog(['node', 'generate-changelog', ...args.slice(1)])
      break
    }
    case 'prepare-gradle-maven': {
      let i = 1
      const url = args[i++]
      let user: string | undefined = undefined
      let pass: string | undefined = undefined
      while (i < args.length) {
        switch (args[i++]) {
          case '--user':
            user = args[i++] ?? throws(new Error("no value for --user"))
            break
          case '--pass':
            pass = args[i++] ?? throws(new Error("no value for --pass"))
            break
          default:
            throws(new Error(`unknown option: ${args[i - 1]}`))
        }
      }
      await new GradleMaven({url, user, pass}).configure()
      break
    }
    case 'prepare-gradle-signing': {
      const key: string = args[1] ?? throws(new Error("no gpg key is specified"))
      const pass: string = args[2] ?? throws(new Error("no gpg pass is specified. " +
        "if not exists, pass empty string"))
      await new GradleSigning({key, pass}).configure()
      break
    }
    case 'prepare-gradle-plugin-portal': {
      const [key, secret] = args.slice(1)
      await new GradlePluginPortal({key, secret}).configure()
      break
    }
    case 'prepare-gradle-intellij': {
      const [token] = args.slice(1)
      await new GradleIntellij({token}).configure()
      break
    }
    case 'publish-to-curse-forge': {
      await publishToCurseForge(args.slice(1))
      break
    }
    case 'publish-to-maven': {
      await publishToMaven(args.slice(1))
      break
    }
    case 'send-tweet': {
      await sendTweet(args.slice(1))
      break
    }
    default:
      throw new Error(`unknown command: ${args[0]}`)
  }
}

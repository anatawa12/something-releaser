import {promises as fs} from 'fs'
import * as path from 'path'
import * as core from '@actions/core'
import {Octokit} from '@octokit/rest'
import {run as autoChangelog} from 'auto-changelog/src/run'
import {setGitUser} from './commands/git-user'
import {GradleIntellij} from './commands/gradle-intellij'
import {GradleMaven} from './commands/gradle-maven'
import {GradlePluginPortal} from './commands/gradle-plugin-portal'
import {Version} from './types'
import {createFromEnvVariable as createVersionChangers} from './version-changer'

function throws(error: Error): never {
  throw error
}

function getEnv(name: string): string {
  return process.env[name]
    ?? throws(new Error(`environment variable ${name} not found`))
}

function println(body: string): void {
  // eslint-disable-next-line no-console
  console.log(body)
}

type Command =
  | ['something-releaser', ...string[]]
  | ['install', ...([string] | [string, string] | [] )]
  | ['set-git-user', string]
  | ['get-version']
  | ['set-version', string]
  | ['generate-changelog', ...string[]]
  | ['prepare-gradle-maven', ...string[]]
  | ['prepare-gradle-plugin-portal', string, string]
  | ['prepare-gradle-intellij', string]

export async function main(...args: string[]): Promise<void> {
  return await mainImpl(...args as Command)
}

async function trueOrENOENT(promise: Promise<unknown>): Promise<boolean> {
  try {
    await promise
    return true
  } catch (e) {
    if (e.code === 'ENOENT')
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
      const changers = createVersionChangers(getEnv('RELEASER_CHANGER'))
      println((await changers.getVersionName()).toString())
      break
    }
    case 'set-version': {
      const changers = createVersionChangers(getEnv('RELEASER_CHANGER'))
      const version = Version.parse(args[1]
        ?? throws(new Error(`version name required`)))
      await changers.setVersionName(version)
      break
    }
    case 'generate-changelog': {
      await autoChangelog(['node', 'generate-changelog', ...args.slice(1)])
      break
    }
    case 'prepare-gradle-maven': {
      let i = 1
      let sign: {'gpg-key': string, 'gpg-pass': string} | undefined = undefined
      if (args[i] === '--signed') {
        i++
        sign = {
          'gpg-key': args[i++],
          'gpg-pass': args[i++],
        }
      } else if (args[i] !== '--unsigned')
        throws(new Error('--signed or --unsigned expected'))
      let user: string | undefined = undefined
      let pass: string | undefined = undefined
      const repo: {url: string, user?: string, pass?: string}[] = []
      while (i < args.length) {
        switch (args[i++]) {
          case '--user':
            user = args[i++]
            break
          case '--pass':
            pass = args[i++]
            break
          case '--url':
            repo.push({url: args[i++], user, pass})
            user = undefined
            pass = undefined
            break
          default:
            throws(new Error(`unknown option: ${args[i - 1]}`))
        }
      }
      await new GradleMaven({sign, repo}).configure()
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
    default:
      throw new Error(`unknown command: ${args[0]}`)
  }
}

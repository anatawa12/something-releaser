import * as path from 'path'
import env from './env'
import {checkNever, throws} from './utils'

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
  | import("./commands/version-commands").VersionCommand
  | ['generate-changelog', ...string[]]
  | ['prepare-gradle-maven', string, ...string[]]
  | ['prepare-gradle-signing', string, ...string[]]
  | ['prepare-gradle-plugin-portal', string, string]
  | ['prepare-gradle-intellij', string]
  | ['publish-to-curse-forge', ...string[]]
  | ['publish-to-maven', ...string[]]
  | ['send-tweet', ...string[]]
  | ['send-discord', ...string[]]
  | ['file-util', ...string[]]
  | import('./commands/github-commands').GithubCommands

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
      const core = await import('@actions/core')
      const {promises: fs} = await import('fs')
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
      const {promises: fs} = await import('fs')
      const {setGitUser} = await import('./commands/git-user')
      const {Octokit} = await import('@octokit/rest')
      const octokit = new Octokit({auth: await fs.readFile(ghTokenPath, 'utf8')})
      const user = args[1] ?? throws(new Error(`user name required`))
      await setGitUser(user, octokit)
      break
    }
    case 'get-version': {
      const {createChangers} = await import('./version-changer')
      const changers = createChangers(env.releaseChanger)
      println((await changers.getVersionName()).toString())
      break
    }
    case 'set-version': {
      const {createChangers} = await import('./version-changer')
      const changers = createChangers(env.releaseChanger)
      const version = args[1] ?? throws(new Error(`version name required`))
      await changers.setVersionName(version)
      break
    }
    case 'version-unsnapshot':
    case 'version-stable':
    case 'version-snapshot':
    case 'version-alpha':
    case 'version-beta':
    case 'version-candidate':
    case 'version-major':
    case 'version-minor':
    case 'version-patch':
    case 'version-get-channel':
    case 'version-set-channel':
    case 'version-next':
    case "version-format": {
      await (await import("./commands/version-commands")).runVersionCommands(args)
      break
    }
    case 'generate-changelog': {
      const {run: autoChangelog} = await import('auto-changelog/src/run')
      await autoChangelog(
        ['node', 'generate-changelog', ...args.slice(1)], 
        env.changelog,
      )
      break
    }
    case 'prepare-gradle-maven': {
      const {GradleMaven} = await import('./commands/gradle-maven')
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
      const {GradleSigning} = await import('./commands/gradle-signing')
      const key: string = args[1] ?? throws(new Error("no gpg key is specified"))
      const pass: string = args[2] ?? throws(new Error("no gpg pass is specified. " +
        "if not exists, pass empty string"))
      await new GradleSigning({key, pass}).configure()
      break
    }
    case 'prepare-gradle-plugin-portal': {
      const {GradlePluginPortal} = await import('./commands/gradle-plugin-portal')
      const [key, secret] = args.slice(1)
      await new GradlePluginPortal({key, secret}).configure()
      break
    }
    case 'prepare-gradle-intellij': {
      const {GradleIntellij} = await import('./commands/gradle-intellij')
      const [token] = args.slice(1)
      await new GradleIntellij({token}).configure()
      break
    }
    case 'publish-to-curse-forge': {
      const {publishToCurseForge} = await import('./commands/publish-to-curse-forge')
      await publishToCurseForge(args.slice(1))
      break
    }
    case 'publish-to-maven': {
      const {publishToMaven} = await import('./commands/publish-to-maven')
      await publishToMaven(args.slice(1))
      break
    }
    case 'send-tweet': {
      const {sendTweet} = await import('./commands/send-tweet')
      await sendTweet(args.slice(1))
      break
    }
    case 'send-discord': {
      const {sendDiscord} = await import('./commands/send-discord')
      await sendDiscord(args.slice(1))
      break
    }
    case 'file-util': {
      const {fileUtil} = await import('./commands/file-util')
      await fileUtil(args.slice(1))
      break
    }
    case 'gh-get-input':
    case 'gh-get-input-boolean':
    case 'gh-set-output':
    case 'gh-export-variable':
    case 'gh-set-secret':
    case 'gh-add-path':
    case 'gh-group-start':
    case 'gh-group-end':
    case 'gh-error':
    case 'gh-warning':
    case 'gh-notice': {
      const {runGithubCommands} = await import("./commands/github-commands")
      await runGithubCommands(args)
      break
    }
    default:
      checkNever(args[0])
      throw new Error(`unknown command: ${args[0]}`)
  }
}

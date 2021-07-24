import * as core from '@actions/core'
import * as github from '@actions/github'
import {parseConfig} from './steps/00.config'
import {setGitUser} from './steps/01.git-user'
import {setCurrentVersion} from './steps/02.set-current-version'

async function run(): Promise<void> {
  const baseDir = core.getInput('base-dir')
  process.chdir(baseDir || ".")

  const token = core.getInput('token')
  const octokit = github.getOctokit(token)

  const config = await parseConfig(core.getInput('config-path'))

  await setGitUser(config.gitUser, octokit)

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const currentVersion = await setCurrentVersion(config.versionChangers)
}

// eslint-disable-next-line github/no-then
run().catch(error => core.setFailed(error))

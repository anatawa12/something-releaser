import * as core from '@actions/core'
import * as github from '@actions/github'
import {parseConfig} from './steps/00.config'
import {setGitUser} from './steps/01.git-user'
import {setCurrentVersion} from './steps/02.set-current-version'
import {createFromJson as createVersionChanger} from './version-changer'

async function run(): Promise<void> {
  const token = core.getInput('token')
  const octokit = github.getOctokit(token)

  const config = await parseConfig(core.getInput('config-path'))
  const changers = createVersionChanger(config['version-changer'])

  await setGitUser(config['git-user'], octokit)

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const currentVersion = await setCurrentVersion(changers)
}

// eslint-disable-next-line github/no-then
run().catch(error => core.setFailed(error))

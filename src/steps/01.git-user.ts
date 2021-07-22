import * as exec from '@actions/exec'
import {Octokit} from '../types'

export async function setGitUser(
  username: string,
  octokit: Octokit,
): Promise<void> {
  const [login, mail] = await findGitUser(username, octokit)

  await exec.exec('git', ['config', '--global', 'user.name', login])
  await exec.exec('git', ['config', '--global', 'user.email', mail])
}

export async function findGitUser(
  username: string,
  octokit: Octokit,
): Promise<[string, string]> {
  const res = await octokit.rest.users.getByUsername({username})
  if (res.data.type === 'Organization')
    throw new Error("You can't commit as a Organization")

  const login = res.data.login as string
  const id = res.data.id
  const mail = `${id}+${login}@users.noreply.github.com`

  return [login, mail]
}

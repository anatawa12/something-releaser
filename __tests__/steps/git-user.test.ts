import {expect, test} from '@jest/globals'
import {Octokit} from '@octokit/rest'
import * as gitUser from '../../src/commands/git-user'

const octokit = new Octokit({auth: process.env.GH_TOKEN})

test('throws organization', async () => {
  await expect(gitUser.findGitUser('fixrtm', octokit))
    .rejects.toThrow("You can't commit as a Organization")
})

test('user account', async () => {
  await expect(gitUser.findGitUser('anatawa12', octokit))
    .resolves.toEqual([
      'anatawa12',
      '22656849+anatawa12@users.noreply.github.com',
    ])
})

test('github actions account', async () => {
  await expect(gitUser.findGitUser('github-actions[bot]', octokit))
    .resolves.toEqual([
      'github-actions[bot]',
      '41898282+github-actions[bot]@users.noreply.github.com',
    ])
})

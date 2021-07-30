import {describe, test, expect} from '@jest/globals'
import {CommitObject} from 'isomorphic-git'
import {asAsyncSequence, Commit, RevWalker} from '../../src/utils'

function newRevWalker(
  readCommit: (oid: string) => Promise<Commit>,
): RevWalker {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return new (RevWalker as any)(readCommit)
}

describe('walk', () => {
  test('simple walk', async () => {
    const walker: RevWalker = newRevWalker(walkTestingReadCommit)
    walker.push(await walkTestingReadCommit('05'))
    await expect(asAsyncSequence(walker.asIterable()).asArray())
      .resolves
      .toEqual([
        commit('05', ['03', '04'], author(120), author(30)),
        commit('04', ['02'], author(110), author(25)),
        commit('03', ['02'], author(110), author(20)),
        commit('02', ['01'], author(90), author(10)),
        commit('01', [], author(100), author(0)),
      ])
  })
  test('hiding', async () => {
    const walker: RevWalker = newRevWalker(walkTestingReadCommit)
    walker.push(await walkTestingReadCommit('05'))
    walker.hide(await walkTestingReadCommit('03'))
    await expect(asAsyncSequence(walker.asIterable()).asArray())
      .resolves
      .toEqual([
        commit('05', ['03', '04'], author(120), author(30)),
        commit('04', ['02'], author(110), author(25)),
      ])
  })
})

const commit = (
  oid: string,
  parent: string[],
  author: CommitObject['author'],
  committer: CommitObject['committer'],
): Commit => ({
  oid,
  message: 'message',
  tree: 'tree',
  parent,
  author,
  committer,
})

const author = (timestamp: number): CommitObject['author'] => ({
  name: 'test',
  email: 'test@example.com',
  timestamp,
  timezoneOffset: 0,
})

async function walkTestingReadCommit(oid: string): Promise<Commit> {
  switch (oid) {
    case '01':
      return commit('01', [], author(100), author(0))
    case '02':
      return commit('02', ['01'], author(90), author(10))
    case '03':
      return commit('03', ['02'], author(110), author(20))
    case '04':
      return commit('04', ['02'], author(110), author(25))
    case '05':
      return commit('05', ['03', '04'], author(120), author(30))
    default:
      throw new Error(`oid not found: ${oid}`)
  }
}

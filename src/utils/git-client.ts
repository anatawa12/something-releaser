import fs from 'fs'
import FastPriorityQueue from 'fastpriorityqueue'
import {CallbackFsClient, PromiseFsClient} from 'isomorphic-git'
import * as git from 'isomorphic-git'
import simpleGit, {DiffResult, SimpleGit} from 'simple-git/promise'

/* eslint-disable @typescript-eslint/promise-function-async */
export class GitClient {
  private opts: {fs: CallbackFsClient | PromiseFsClient, dir: string}
  private simple: SimpleGit

  constructor(dir: string) {
    this.opts = {fs, dir}
    this.simple = simpleGit(dir)
  }

  listTags(): Promise<string[]> {
    return git.listTags({...this.opts})
  }

  resolveRef(ref: string): Promise<string> {
    return git.resolveRef({...this.opts, ref})
  }

  async readCommit(oid: string): Promise<Commit> {
    const result =  await git.readCommit({...this.opts, oid})
    return {
      oid,
      ...result.commit,
    }
  }

  diff(from: string, to: string): Promise<DiffResult> {
    return this.simple.diffSummary([from, to])
  }

  revWalk(): RevWalker {
    // it's safe because RevWalker has new(GitClient).
    // but that's internal so it may not safe in the feature.
    // don't do in your code.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return new (RevWalker as any)(this.readCommit.bind(this))
  }
}

/* eslint-enable @typescript-eslint/promise-function-async */

interface Author {
  name: string
  email: string
  timestamp: number
  timezoneOffset: number
}

export interface Commit {
  oid: string
  message: string
  tree: string
  parent: string[]
  author: Author
  committer: Author
  gpgsig?: string
}

export class RevWalker {
  private readonly queue: DateSortedQueue
  private readonly seen: Set<string>
  private readonly hidden: Set<string>
  private readonly readCommit: (oid: string) => Promise<Commit>

  // internal: not public so private but will be accessed via as any
  // in GitClient and tests
  private constructor(readCommit: (oid: string) => Promise<Commit>) {
    this.readCommit = readCommit
    this.seen = new Set()
    this.hidden = new Set()
    this.queue = new DateSortedQueue()
  }

  push(commit: Commit): void {
    const found = this.findCommit(commit)
    if (found != null)
      this.queue.add(found)
  }

  hide(commit: Commit): void {
    this.hidden.add(commit.oid)
    const found = this.findCommit(commit)
    if (found != null)
      this.queue.add(found)
  }

  async next(): Promise<Commit | null> {
    for (;;) {
      const c = await this.lookForNext()
      if (c == null)
        break
      if (!this.hidden.has(c.oid))
        return c
      // if all commits are not for show: all commits are hidden, no commits will be shown
      if (this.queue.isAllHidden(this.hidden))
        break
    }
    this.queue.clear()
    return null
  }

  // computes next RevWalkCommit and add parents to queue
  private async lookForNext(): Promise<Commit | null> {
    const c = this.queue.poll()
    if (c == null)
      return null

    const hidden = this.hidden.has(c.oid)
    // expand parents
    for (const parentSha of c.parent) {
      if (hidden)
        this.hidden.add(parentSha)
      const p = await this.findSha(parentSha)
      if (p == null)
        continue
      this.queue.add(p)
    }

    return c
  }

  private async findSha(commit: string): Promise<Commit | null> {
    const oid = commit
    if (this.seen.has(oid))
      return null
    this.seen.add(oid)
    return await this.readCommit(commit)
  }

  private findCommit(commit: Commit): Commit | null {
    if (this.seen.has(commit.oid))
      return null
    this.seen.add(commit.oid)
    return commit
  }

  asIterator(): AsyncIterableIterator<Commit> {
    return this.asIterable()
  }

  asIterable(): AsyncIterableIterator<Commit> {
    const iterable: AsyncIterableIterator<Commit> = {
      next: async () => {
        const r = await this.next()
        return r ? { done: false, value: r } : { done: true, value: undefined }
      },
      [Symbol.asyncIterator]: () => {
        return iterable
      },
    }
    return iterable
  }
}

class DateSortedQueue {
  private backed: FastPriorityQueue<Commit>

  constructor() {
    // Placeholder, initialized by clear
    this.backed = null as never
    this.clear()
  }

  add(commit: Commit): void {
    this.backed.add(commit)
  }

  poll(): Commit | undefined {
    return this.backed.poll()
  }

  clear(): void {
    this.backed = new FastPriorityQueue((a, b) =>
      a.committer.timestamp > b.committer.timestamp)
  }

  isAllHidden(hidden: Set<string>): boolean {
    if (this.backed.isEmpty())
      return true

    const fpq = this.backed.clone()

    while (!fpq.isEmpty()) {
      if (!hidden.has(fpq.poll()!.oid))
        return false
    }
    return true
  }
}

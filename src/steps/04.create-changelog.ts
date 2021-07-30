import {promises as fs} from 'fs'
import {DiffResultTextFile} from 'simple-git/typings/response'
import {asSequence, GitClient, asAsyncSequence, Commit} from '../utils'

interface ChangelogInfo {
  markdown: string,
  tags: ReleaseInfo[]
  createMarkdownForRelease(release: ReleaseInfo): string
  createHtmlForRelease(release: ReleaseInfo): string
}

export async function createChangeLog(
  githubRepoUrl: string, 
): Promise<ChangelogInfo> {
  const repo = new GitClient('.')
  const tags = await asSequence(await fetchTags(repo))
    .asAsync()
    .map(async tag => await parseRelease(repo, tag))
    .asArray()
  const creator = new GithubLinkCreator(githubRepoUrl)
  const result =  {
    markdown: createMarkdown(tags, creator),
    tags,
    createMarkdownForRelease(release: ReleaseInfo) {
      return createMarkdownForRelease(release, creator)
    },
    createHtmlForRelease(release: ReleaseInfo) {
      return createHtmlForRelease(release, creator)
    },
  }

  await fs.writeFile('CHANGELOG.md', result.markdown, 'utf8')

  return result
}

interface LinkCreator {
  compareLink(from: string | null, to: string): string | null;
  issueLink(id: number): string | null;
  mergeLink(id: number): string | null;
  commitLink(id: string): string | null;
}

type ChangelogRelease_ = readonly [TagCommit, TagCommit | null]
// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface ChangelogRelease extends ChangelogRelease_ {}

type TagCommit_ = readonly [string, Commit]
// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface TagCommit extends TagCommit_ {}

class GithubLinkCreator implements LinkCreator {
  readonly base: string

  constructor(base: string) {
    this.base = base
  }

  compareLink(from: string | null, to: string): string | null {
    return `${this.base}/compare/${from}...${to}`
  }

  issueLink(id: number): string | null {
    return `${this.base}/issues/${id}`
  }

  mergeLink(id: number): string | null {
    return `${this.base}/pull/${id}`
  }

  commitLink(id: string): string | null {
    return `${this.base}/commit/${id}`
  }
}

async function fetchTags(repo: GitClient): Promise<ChangelogRelease[]> {
  const tags = await repo.listTags()
  const commits = await asSequence<string>(tags).asAsync()
    .filter(name => name.match(/^v?[0-9.]*$/) != null)
    .map(async t => [t, await repo.resolveRef(`ref/tags/${t}`)] as const)
    .map(async ([t, r]) => [t, await repo.readCommit(r)] as const)
    .asArray()

  // newer first
  commits.sort(([, c0], [, c1]) => 
    c1.committer.timestamp - c0.committer.timestamp)

  return asSequence(commits).zipWithNext(true).asArray()
}

interface ReleaseInfo {
  merges: MergeInfo[]
  fixes: CommitFixInfo[]
  commits: Commit[]
  summary: string | null,
  date: number,
  tag: string,
  prev: string | null,
}

async function parseRelease(repo: GitClient, [newer, older]: ChangelogRelease): Promise<ReleaseInfo> {
  const commits = await getCommits(repo, newer, older)
  const merges = asSequence(commits).mapNotNull(tryParseMerge).asArray()
  const fixes = asSequence(commits).mapNotNull(tryParseFix).asArray()
  const summary = commits[0]?.message
  const date = newer[1].committer.timestamp
  const emptyRelease = merges.length === 0 && fixes.length === 0
  return {
    merges,
    fixes,
    summary,
    date,
    tag: newer[0],
    prev: older?.[0] ?? null,
    commits: emptyRelease ? await collectAndSortCommitsForReleaseNote(repo, commits): [],
  }
}

async function collectAndSortCommitsForReleaseNote(
  repo: GitClient, 
  commits: Commit[],
): Promise<Commit[]> {
  const commitPattern = /^v?\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$/
  const commitInfos = await asSequence(commits)
    .filter(it => it.message.match(commitPattern) != null)
    .asAsync()
    .map(async (commit) => {
      const diff = await repo.diff(commit.oid, commit.parent[0])
      const differences = asSequence(diff.files)
        .filter((it): it is DiffResultTextFile => !it.binary)
        .map(it => it.insertions + it.deletions)
        .sum()
      return [differences, commit] as const
    })
    .asArray()

  commitInfos.sort((a, b) =>
    a[0] - b[0])
  return asSequence(commitInfos).map(it => it[1]).asArray()
}

async function getCommits(
  repo: GitClient, 
  newer: TagCommit, 
  older: TagCommit | null,
): Promise<Commit[]> {
  const walker = repo.revWalk()
  walker.push(newer[1])
  if (older)
    walker.hide(older[1])
  return await asAsyncSequence(walker.asIterable()).asArray()
}

interface MergeInfo {
  id: number
  message: string,
  commit: Commit
}

function tryParseMerge(commit: Commit): MergeInfo | null {
  const commitMessage = commit.message

  let idStr: string
  let message: string
  let cap: RegExpMatchArray | null

  // Merge pull request #<id> from <branch>\n\n<message>
  // message \(#(\d+)\)(?:$|\n\n)

  if ((cap = commitMessage.match(/Merge pull request #(\d+) from .+\n\n(.+)/)) != null) {
    idStr = cap[1]
    message = cap[2]
  } else if ((cap = commitMessage.match(/(.+) \(#(\d+)\)(?:$|\n\n)/)) != null) {
    idStr = cap[2]
    message = cap[1]
  } else {
    // not a merge
    return null
  }
  const id = parseInt(idStr)
  // invalid merge request id
  if (!Number.isInteger(id))
    return null
  return { id, message, commit }
}

interface CommitFixInfo {
  ids: number[]
  commit: Commit
}

function tryParseFix(commit: Commit): CommitFixInfo | null {
  const regex = /(?:close[sd]?|fix(e[sd])?|resolve[sd]?)\s(?:#|https?:\/\/.+?\/(?:issues|pull|pull-requests|merge_requests)\/)(\d+)/g
  const ids: number[] = []
  let match: RegExpExecArray | null

  while ((match = regex.exec(commit.message)) !== null) {
    const id = parseInt(match[0])
    if (Number.isInteger(id)) {
      ids.push(id)
    }
  }

  if (ids.length === 0)
    return null

  return { ids, commit }
}

function mdLink(body: string, link: string | null): string {
  if (!link)
    return body
  else
    return `[${body}](${link})`
}

function createMarkdown(
  releases: Iterable<ReleaseInfo>, 
  links: LinkCreator,
): string {
  let out = ''
  out += `### Changelog\n`
  out += `All notable changes to this project will be documented in this file. Dates are displayed in UTC.\n`
  out += `\n`
  out += `Generated by [\`something-releaser\`](https://github.com/anatawa12/something-releaser).\n`
  out += `\n`
  for (const release of releases) {
    out += `#### ${mdLink(encodeHtml(release.tag), links.compareLink(release.prev, release.tag))}\n`
    if (release.date) {
      out += '\n'
      out += `> ${formatDate}\n`
    }
    out += '\n'
    out += createMarkdownForRelease(release, links)
    out += '\n'
  }
  return out
}

function createMarkdownForRelease(
  release: ReleaseInfo, 
  links: LinkCreator,
): string {
  let out = ''
  for (const merge of release.merges) {
    out += `- ${encodeHtml(merge.message)}`
      + ` ${mdLink(`\`${merge.id}\``, links.mergeLink(merge.id))}\n`
  }
  for (const fix of release.fixes) {
    out += `- ${encodeHtml(commitSummary(fix.commit))}`
    for (const id of fix.ids) {
      out += ` ${mdLink(`\`${id}\``, links.mergeLink(id))}\n`
    }
    out += `\n`
  }
  for (const commit of release.commits) {
    const short = commit.oid.slice(0, 7)
    out += `- ${encodeHtml(commitSummary(commit))}`
      + ` ${mdLink(`\`${short}\``, links.commitLink(commit.oid))}\n`
  }
  return out
}

function htmlLink(body: string, link: string | null): string {
  if (!link)
    return body
  else
    return `<a href="${encodeHtml(link)}">${body}</a>`
}

function createHtmlForRelease(
  release: ReleaseInfo,
  links: LinkCreator,
): string {
  let out = ''
  out += '<ul>\n'
  for (const merge of release.merges) {
    out += `<li>${encodeHtml(merge.message)}`
      + ` ${mdLink(`\`${merge.id}\``, links.mergeLink(merge.id))}</li>\n`
  }
  for (const fix of release.fixes) {
    out += `<li>${encodeHtml(commitSummary(fix.commit))}`
    for (const id of fix.ids) {
      out += ` ${htmlLink(`\`${id}\``, links.mergeLink(id))}\n`
    }
    out += `</li>\n`
  }
  for (const commit of release.commits) {
    const short = commit.oid.slice(0, 7)
    out += `<li>${encodeHtml(commitSummary(commit))}`
      + ` ${htmlLink(`\`${short}\``, links.commitLink(commit.oid))}</ul>\n`
  }
  out += '</ul>\n'
  return out
}

function formatDate(timestamp: number): string {
  const months = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ]

  const instance = new Date(timestamp)
  const date = instance.getUTCDate()
  const month = months[instance.getUTCMonth()]
  const year = `${instance.getUTCFullYear()}`.padStart(4, '0')
  return `${date} ${month} ${year}`
}

function commitSummary(commit: Commit): string {
  return asSequence(commit.message.split(/\n/))
    .takeWhile(it => it !== '')
    .map(it => it.trim())
    .asArray()
    .join(' ')
}

function encodeHtml(body: string): string {
  return body
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

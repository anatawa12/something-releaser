import {createReadStream, readFileSync} from 'fs'
import {basename} from 'path'
import {Command, InvalidArgumentError, Option} from 'commander'
import FormData from 'form-data'
import fetch from 'node-fetch'

export async function publishToCurseForge(args: string[]): Promise<void> {
  const opts = parseArgs(args)

  const data = new FormData()
  data.append('file', createReadStream(opts.file), basename(opts.file))
  data.append('metadata', JSON.stringify(opts.metadata))

  const response = await fetch(`https://minecraft.curseforge.com/api/projects/${opts.projectId}/upload-file`, {
    body: data,
    method: "POST",
    headers: {
      "X-Api-Token": opts.token,
    },
  })
  // eslint-disable-next-line no-console
  console.log(await response.text())
  process.exit(response.ok ? 0 : 1)
}

interface ParsedOptions {
  // request information
  file: string,
  token: string,
  projectId: number,

  metadata: {
    changelog: string,
    changelogType: 'html' | 'text' | 'markdown',
    displayName?: string,
    parentFileID?: number,
    gameVersions?: number[],
    releaseType: 'alpha' | 'beta' | 'release',
  }
}

function parseArgs(args: string[]): ParsedOptions {
  interface RealArgs {
    readonly file: string,
    readonly token: string,
    readonly projectId: number,

    readonly parentFile?: number,
    readonly name?: string,
    readonly changelog?: string,
    readonly changelogFile?: string,
    readonly changelogType: 'html' | 'text' | 'markdown',
    readonly releaseType: 'alpha' | 'beta' | 'release',
    readonly gameVersions: number[],
  }

  const opts = new Command()
    .requiredOption('-f, --file <resource file>', 'the path to jar file')
    .requiredOption('-t, --token <token>', 'the API token')
    .requiredOption('-i, --project-id <id>', 'the project id', numberParser('project id'))

    // metadata options
    .option('-p, --parent-file <id>', 'the parent file id', numberParser('parent file id'))
    .option('-n, --name <display name>', 'the display name of file')
    .option('--changelog <changelog text>', 'the changelog')
    .option('--changelog-file <changelog file>', 'the path to changelog')
    .addOption(new Option('--changelog-type <changelog file>', 'the document type of changelog')
      .default('html')
      .choices(['html', 'text', 'markdown']))
    .addOption(new Option('--release-type <release type>', 'the path to changelog file')
      .makeOptionMandatory()
      .choices(['alpha', 'beta', 'release']))
    .option('--game-versions <version id...>', 'the path to changelog file', parseVersionId, [])
    .parse(args, {from: 'user'})
    .opts<RealArgs>()

  if (opts.changelogFile && opts.changelog) {
    process.stderr.write("can't set both --changelog and --changelog-file")
    process.exit(1)
  }
  if (!(opts.changelogFile || opts.changelog)) {
    process.stderr.write("please set either --changelog or --changelog-file")
    process.exit(1)
  }

  const changelog: string = opts.changelogFile
    ? readFileSync(opts.changelogFile, {encoding: "utf8"})
    : opts.changelog as string
  

  return {
    file: opts.file,
    token: opts.token,
    projectId: opts.projectId,
    metadata: {
      changelog,
      changelogType: opts.changelogType,
      displayName: opts.name,
      parentFileID: opts.parentFile,
      gameVersions: opts.parentFile ? undefined : opts.gameVersions,
      releaseType: opts.releaseType,
    },
  }
}

function parseVersionId(value: string, previous: number[]): number[] {
  previous = previous || []
  const number = parseInt(value)
  if (Number.isNaN(number))
    throw new InvalidArgumentError(`invalid version id: ${value}`)
  return [...previous, number]
}

function numberParser(name: string): (value: string) => number {
  return (value) => {
    const number = parseInt(value)
    if (Number.isNaN(number))
      throw new InvalidArgumentError(`invalid ${name}: ${value}`)
    return number
  }
}

import {createHash} from 'crypto'
import {createReadStream} from 'fs'
import {extname} from 'path'
import {Readable} from 'stream'
import {Command, InvalidArgumentError} from 'commander'
import {JSDOM} from 'jsdom'
import nodeFetch, {RequestInfo, RequestInit, Response} from 'node-fetch'
import {
  createMessage as createPgpMessage, 
  decryptKey, 
  PrivateKey, 
  readPrivateKey, 
  sign as signPgp,
} from 'openpgp'
import env from '../env'
import {asPair} from '../utils'
import {createChangers} from '../version-changer'

const jsdom = new JSDOM("")

type OurFetch = (url: RequestInfo, init?: RequestInit) => Promise<Response>

export async function publishToMaven(args: string[]): Promise<void> {
  const opts = await parseArgs(args)

  const {
    metadataXmlUrl,
    fileBaseUrl,
    fileUrl,
  } = computeUrls(opts)

  let fetch: OurFetch
  if (opts.user) {
    const authorizationHeader = `Basic ${Buffer.from(opts.user, 'utf-8').toString('base64')}`
    fetch = async (url, init) => nodeFetch(url, {
      ...init,
      headers: {
        ...init?.headers,
        'Authorization': authorizationHeader,
      },
    })
  } else {
    fetch = nodeFetch
  }

  let privateKey: PrivateKey | null = null
  if (opts.signingKey) {
    privateKey = await readPrivateKey({armoredKey: opts.signingKey})
    if (opts.signingPass) {
      privateKey = await decryptKey({
        privateKey,
        passphrase: opts.signingPass,
      })
    }
  }

  const fileReading = opts.file === '-' ? process.stdin : createReadStream(opts.file)
  await updateMetadataXml(metadataXmlUrl, opts, fetch)
  const filesToUpload: FileDesc[] = []
  filesToUpload.push([fileUrl, await readToBuffer(fileReading)])
  if (opts.pom)
    filesToUpload.push([`${fileBaseUrl}.pom`, createPomFile(opts)])
  if (privateKey)
    await addSigns(filesToUpload, privateKey)
  addHashFiles(filesToUpload)

  await pushFiles(filesToUpload, fetch)
}

type FileDesc = [url: string, body: Buffer]

function computeUrls(opts: ParsedOptions): {
  artifactUrl: string
  metadataXmlUrl: string
  fileBaseName: string
  fileBaseUrl: string
  fileUrl: string
} {

  let artifactUrl = opts.repository
  if (opts.groupId)
    artifactUrl += `/${opts.groupId.replace(/\./g, '/')}`
  if (opts.artifactId)
    artifactUrl += `/${opts.artifactId}`

  let fileBaseName = opts.artifactId
  if (opts.versionName)
    fileBaseName += `-${opts.versionName}`
  if (opts.classifier)
    fileBaseName += `-${opts.classifier}`

  let fileBaseUrl = artifactUrl
  if (opts.versionName)
    fileBaseUrl += `/${opts.versionName}`
  fileBaseUrl += `/${fileBaseName}`
  
  let fileUrl = fileBaseUrl
  if (opts.extension)
    fileUrl += `.${opts.extension}`

  return {
    artifactUrl,
    fileBaseName,
    fileBaseUrl,
    fileUrl,
    metadataXmlUrl: `${artifactUrl}/maven-metadata.xml`,
  }
}

// returns true if updated, false not updated
async function updateMetadataXml(
  metadataXmlUrl: string,
  opts: ParsedOptions,
  fetch: OurFetch,
): Promise<boolean> {
  const response = await fetch(metadataXmlUrl, {})
  let xmlText: string
  if (response.status === 404) {
    xmlText = `<?xml version="1.0" encoding="UTF-8"?>\n<metadata>\n</metadata>`
  } else {
    handleFetchError(`fetching ${metadataXmlUrl}`, response)
    xmlText = await response.text()
  }
  const document = new jsdom.window.DOMParser().parseFromString(xmlText, "application/xml")
  const versioning = findOrAppendNode(document, 'versioning')
  const versions = findOrAppendNode(versioning, 'versions')
  const versionList = versions.querySelectorAll(":scope > version")
  for (const version of versionList) {
    if (version.textContent === opts.versionName)
      return false
  }

  // requires update
  addTextElement(versions, 'version', opts.versionName)
  if (!opts.notRelease)
    findOrAppendNode(versioning, 'release', true).textContent = opts.versionName
  findOrAppendNode(versioning, 'latest', true).textContent = opts.versionName
  await fetch(metadataXmlUrl, {
    method: "PUT",
    body: new jsdom.window.XMLSerializer().serializeToString(document),
  })
    .then(handleFetchError.bind(null, `PUTing ${metadataXmlUrl}`))
    .then(async it => it.text())
  return true
}

async function readToBuffer(readable: Readable): Promise<Buffer> {
  const buffers: Buffer[] = []
  for await (const buf of readable) {
    buffers.push(Buffer.isBuffer(buf) ? buf : Buffer.from(buf))
  }
  return Buffer.concat(buffers)
}

function createPomFile(opts: ParsedOptions): Buffer {
  const document = jsdom.window.document.implementation
    .createDocument(null, "project")
  addTextElement(document, "modelVersion", "4.0.0")
  opts.groupId && addTextElement(document, "groupId", opts.groupId)
  opts.artifactId && addTextElement(document, "artifactId", opts.artifactId)
  opts.versionName && addTextElement(document, "version", opts.versionName)
  opts.packaging && addTextElement(document, "packaging", opts.packaging)

  opts.name && addTextElement(document, "name", opts.name)
  opts.description && addTextElement(document, "description", opts.description)
  opts.url && addTextElement(document, "url", opts.url)
  //opts.licenses
  if (opts.developers.length) {
    const developers = addTextElement(document, "developers")
    for (const config of opts.developers) {
      const developer = addTextElement(developers, "developer")
      config.name && addTextElement(developer, "name", config.name)
      config.id && addTextElement(developer, "id", config.id)
      config.mail && addTextElement(developer, "mail", config.mail)
      if (config.roles.length) {
        const roles = addTextElement(developer, "roles")
        for (const role of config.roles)
          addTextElement(roles, "role", role)
      }
      config.timezone && addTextElement(developer, "timezone", config.timezone)
    }
  }

  // dependencies
  if (opts.dependencies.length) {
    const dependencies = addTextElement(document, "dependencies")
    for (const depCfg of opts.dependencies) {
      const dependency = addTextElement(dependencies, "dependency")
      depCfg.group && addTextElement(dependency, "groupId", depCfg.group)
      depCfg.artifact && addTextElement(dependency, "artifactId", depCfg.artifact)
      depCfg.version && addTextElement(dependency, "version", depCfg.version)
      depCfg.classifier && addTextElement(dependency, "classifier", depCfg.classifier)
      depCfg.extension && addTextElement(dependency, "type", depCfg.extension)
    }
  }
  return Buffer.from(new jsdom.window.XMLSerializer().serializeToString(document), "utf-8")
}

async function addSigns(filesToUpload: FileDesc[], privateKey: PrivateKey): Promise<void> {
  const len = filesToUpload.length
  for (let i = 0; i < len; i++) {
    const file = filesToUpload[i]
    const detached = await signPgp<Uint8Array>({
      message: await createPgpMessage({binary: file[1]}),
      signingKeys: privateKey,
      detached: true,
    })
    filesToUpload.push([`${file[0]}.asc`, Buffer.from(detached, 'utf-8')])
  }
}

function addHashFiles(filesToUpload: FileDesc[]): void {
  const addHashFile = (file: FileDesc, hashName: string): void => {
    const hash = createHash(hashName)
    hash.update(file[1])
    filesToUpload.push([
      `${file[0]}.${hashName}`, 
      Buffer.from(hash.digest().toString("hex"), 'utf-8'),
    ])
  }

  const len = filesToUpload.length
  for (let i = 0; i < len; i++) {
    const file = filesToUpload[i]
    addHashFile(file, 'md5')
    addHashFile(file, 'sha1')
    addHashFile(file, 'sha256')
    addHashFile(file, 'sha512')
  }
}

async function pushFiles(filesToUpload: FileDesc[], fetch: OurFetch): Promise<void> {
  async function pushFile([file, desc]: FileDesc): Promise<void> {
    await fetch(file, {
      method: 'PUT',
      body: desc,
    }).then(handleFetchError.bind(null,  `putting to ${file}`))
  }

  for (const fileToUpload of filesToUpload) {
    await pushFile(fileToUpload)
  }
}

function handleFetchError(operation: string, response: Response): Response {
  if (200 <= response.status && response.status < 300)
    return response
  if (400 <= response.status)
    throw new Error(`${operation} returns error status: ${response.status} ${response.statusText}`)
  throw new Error(`${operation} returns unknown status: ${response.status} ${response.statusText}`)
}

function findOrAppendNode(parent: Element | Document, node: string, first?: true): Element {
  return parent.querySelector(`:scope > ${node}`) ?? addElement(parent, node, first)
}

function addTextElement(parent: Document | Element, node: string, content?: string): Element {
  const newElement = addElement(parent, node)
  if (content)
    newElement.textContent = content
  return newElement
}

function addElement(parent: Element | Document, node: string, first?: boolean): Element {
  const newElem = (parent.ownerDocument || parent).createElementNS(null, node)
  const addTo: Element = 'documentElement' in parent ? parent.documentElement : parent
  if (first)
    addTo.prepend(newElem)
  else
    addTo.appendChild(newElem)
  return newElem
}

// if there is no equals on argument, it will be parsed in simple format:
//   1. find /(@\S+)\s*/
//   2. if found
//     1. set the mail as the first matched group
//     2. remove the matched part from the string
//   3. find /<([^>]*)>\s*/
//   4. if found
//     1. set the id as the first matched group
//     2. remove the matched part from the string
//   5. trim the string
//   6. replace /\s+/ to single space
//   7. of the string is not empty
//     1. set the name as the string
//   8. if none of id, mail, and name are defined
//     1. make error
// If there is one or more equals on argument, it will be parsed as key-value pair
// example: ` anatawa12 bot @anatawa12-bot <87023934+anatawa12-bot@users.noreply.github.com> `
// example: `id=anatawa12-bot,name=anatawa12 bot,mail=87023934+anatawa12-bot@users.noreply.github.com`
// export for test
export interface Author {
  id?: string
  name?: string
  mail?: string
  roles: string[]
  timezone?: string
}

interface DependencyNotation {
  group?: string
  artifact: string
  version?: string
  classifier?: string
  extension?: string
}

interface ParsedOptions {
  // artifact information
  readonly file: string
  readonly groupId?: string
  readonly artifactId: string
  readonly versionName: string
  readonly classifier?: string
  readonly extension: string
  readonly packaging: string

  readonly name?: string
  readonly description?: string
  readonly url?: string
  readonly developers: Author[]
  readonly dependencies: DependencyNotation[]

  // repository information
  readonly repository: string
  readonly user?: string

  // signing information
  readonly signingKey?: string
  readonly signingPass?: string

  // operation flags
  readonly notRelease: boolean
  readonly notLatest: boolean
  readonly pom: boolean
}

async function parseArgs(args: string[]): Promise<ParsedOptions> {
  interface RealArgs {
    readonly file: string
    readonly groupId?: string
    readonly artifactId: string
    readonly versionName?: string
    readonly classifier?: string
    readonly extension?: string
    readonly packaging?: string

    readonly name?: string
    readonly description?: string
    readonly url?: string
    readonly developers?: Author[]
    readonly dependencies?: DependencyNotation[]

    readonly repository: string
    readonly user?: string

    readonly signingKey?: string
    readonly signingPass?: string

    readonly notRelease: boolean
    readonly notLatest: boolean
    readonly pom: boolean
  }

  const opts = new Command()
    // artifact identifier
    .requiredOption("-f,--file </path/to/file>", "The file to path. hyphen to read from stdin")
    .option("-g,--group-id <group-id>", "The name of group")
    .requiredOption("-i,--artifact-id <artifact-id>", "The name of artifact")
    .option("-v,--version-name <version-name>", "The name of version")
    .option("-c,--classifier <classifier>", "The classifier of artifact")
    .option("-e,--extension <extension>", "The extension of artifact. defaults extension of the file")
    .option("--packaging <packaging>", "The type of packaging. defaults extension of the file")

    .option("-n,--name <name>", "The name of the artifact.")
    .option("--description,--desc <description>", "The description of the artifact.")
    .option("--url <url>", "The url to the project page.")
    .option(
      "-a,--developers <...developers>",
      "Developer of the artifact." +
      "you can use `$name @$id <$mail>` format or `id=$id,name=$name,mail=$mail` (key-value-pair) format." +
      "On key-value-pair format, you can specify roles (with multiple role=) and timezone (timezone=)." + 
      "One of name, id, or mail must be specified.",
      parseAuthor)
    .option(
      "-d,--dependency <...notation>",
      "Dependencies of the artifact in gradle's text format. " +
      "see https://docs.gradle.org/7.2/dsl/org.gradle.api.artifacts.dsl.DependencyHandler.html#N16E48",
      parseDependencyNotation)

    // repository
    .requiredOption("-r,--repository <url>", "The URL to repository")
    .option("-u,--user <user:pass>", "Basic authorization")

    // signing
    .option("-k,--signing-key <armored key>", "The armored signing key")
    .option("-p,--signing-pass <passphrase>", "The armored signing pass")

    // operation flags
    .option("--not-release,--latest", "Specify the version is not a release, just latest.")
    .option("--not-latest", "Specify the version is not a latest, old release. implies --not-release")
    .option("--no-pom", "Do not publish pom")

    .parse(args, {from: 'user'})
    .opts<RealArgs>()

  let extension: string
  if (opts.extension != null)
    extension = opts.extension
  else if (opts.file !== '-')
    extension = extname(opts.file).substring(1)
  else {
    process.stderr.write("--extension for stdin")
    process.exit(1)
  }

  return {
    ...opts,
    versionName: opts.versionName
      || await createChangers(env.releaseChanger).getVersionName(),
    extension,
    packaging: opts.packaging ?? extension,
    developers: opts.developers ?? [],
    dependencies: opts.dependencies ?? [],
    notRelease: opts.notRelease || opts.notLatest,
  }
}

function parseDependencyNotation(value: string, previous?: DependencyNotation[]): DependencyNotation[] {
  previous = previous || []
  const [notation, extension] = asPair(value, '@', false)
  const elements = notation.split(';')
  if (elements.length < 2 || elements.length > 4)
    throw new InvalidArgumentError(`invalid dependency notation: ${value}`)
  return [...previous, {
    group: elements[0] || undefined,
    artifact: elements[1],
    version: elements[2] || undefined,
    classifier: elements[3] || undefined,
    extension,
  }]
}

// for test
export function parseAuthor(input: string, previous?: Author[]): Author[] {
  previous = previous || []
  const author: Author = {roles: []} 
  if (input.match(/=/)) {
    for (const kvp of input.split(',')) {
      if (kvp === '')
        continue
      const [key, value] = asPair(kvp, '=', false)
      if (!value)
        throw new InvalidArgumentError(`no value specified for: ${key}`)
      switch (key) {
        case 'id':
          author.id = value
          break
        case 'name':
          author.name = value
          break
        case 'mail':
          author.mail = value
          break
        case 'role':
        case 'roles':
          author.roles.push(value)
          break
        case 'timezone':
          author.timezone = value
          break
        default:
          throw new InvalidArgumentError(`unknown key: ${key}`)
      }
    }
  } else {
    let name = input
    const extractMatchedValue = (pattern: RegExp, index: number): string | undefined => {
      let result: string | undefined = undefined
      name = name.replace(pattern, (...matches) => {
        result = matches[index]
        return ''
      })
      return result
    }
    author.mail = extractMatchedValue(/<([^>]+)>/, 1)?.trim()
    author.id = extractMatchedValue(/@(\S+)/, 1)?.trim()
    author.name = name.replace(/\s+/, ' ').trim() || undefined
  }
  if (!author.name
    && !author.id
    && !author.mail)
    throw new InvalidArgumentError(`None of mame, id and mail are specified: ${input}`)
  return [...previous, author]
}


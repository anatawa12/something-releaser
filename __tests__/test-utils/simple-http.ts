import fsSync, {promises as fs} from 'fs'
import http, {IncomingMessage, Server, ServerResponse} from 'http'
import path from 'path'
import httpAuth from 'http-auth'
import mimeType from 'mime-types'

export class SimpleHttp {
  private readonly base: string
  private readonly auth?: { [P in string]?: string }
  private readonly basic: ReturnType<typeof httpAuth.basic>
  private server?: Server

  constructor(args: {
    base: string,
    auth?: { [P in string]?: string }
  }) {
    this.base = args.base
    this.auth = args.auth
    this.basic = httpAuth.basic({}, this.checkPassword.bind(this))
  }

  start(port: number): this {
    this.server = http.createServer(this.handle.bind(this))
    this.server.listen(port)
    return this
  }

  stop(): void {
    if (!this.server)
      throw new Error("no server running")
    this.server.close()
  }

  private async handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const url = req.url!
    if (url.includes("..")) {
      res.writeHead(400)
      res.write("'..' in path  not allowed")
      res.end()
      return
    }
    const targetFile = path.join(this.base, url)
    const contentType = mimeType.lookup(targetFile)

    switch (req.method) {
      case 'GET': {
        this.handleGet(req, res, targetFile, contentType)
        return
      }
      case 'PUT': {
        await this.handlePut(req, res, targetFile)
        return
      }
      default:
        res.writeHead(405)
        res.write(`method ${req.method} not allowed`)
        res.end()
        return
    }
  }

  private handleGet(
    _: IncomingMessage, 
    res: ServerResponse, 
    targetFile: string, 
    contentType: string | false,
  ): void {
    const handle = fsSync.createReadStream(targetFile)
    res.setHeader('Content-Type', contentType || "application/octet-stream")
    handle.pipe(res)
    handle.on('error', (e: NodeJS.ErrnoException) => {
      res.writeHead(e.code && httpCodeByCommonSystemError[e.code] || 500, {
        'Content-Type': 'text/plain',
      })
      res.end(`error: ${e.code}`)
    })
  }

  private async handlePut(
    req: IncomingMessage,
    res: ServerResponse,
    targetFile: string,
  ): Promise<void> {
    if (this.auth && !await this.checkAuth(req)) {
      res.setHeader("WWW-Authenticate", this.basic.generateHeader())
      res.writeHead(401)
      res.end("auth required")
      return
    }
    await fs.mkdir(path.dirname(targetFile), {recursive: true})
    const handle = fsSync.createWriteStream(targetFile)
    req.pipe(handle)
    handle.on('finish', () => {
      res.writeHead(201)
      res.end()
    })
    handle.on('error', (e: NodeJS.ErrnoException) => {
      res.writeHead(e.code && httpCodeByCommonSystemError[e.code] || 500)
      res.end()
    })
  }

  private async checkAuth(req: IncomingMessage): Promise<string | null> {
    const header = req.headers['authorization']
    if (!header)
      return null
    const options = this.basic.parseAuthorization(header)
    if (!options)
      return null
    const {user, pass} = await new Promise<{user?: string, pass?: boolean}>((res, rej) => {
      this.basic.findUser(req, options, (v) => {
        if (v instanceof Error)
          rej(v)
        else
          res(v)
      })
    })
    if (!pass)
      return null
    return user ?? null
  }

  private checkPassword(
    username: string, 
    password: string, 
    callback: (isAuthorized: boolean) => void,
  ): void {
    callback(!this.auth || this.auth[username] === password)
  }
}

httpAuth.basic({}, (user, pass, back) => {
  back(user === process.argv[4] && pass === process.argv[5])
})

const httpCodeByCommonSystemError = Object.create(null, {
  EACCES: {value: 403},
  ENOENT: {value: 404},
  EISDIR: {value: 403},
})

if (module === require.main) {
  const base = process.argv[2]
  const port = parseInt(process.argv[3])
  const username = process.argv[4]
  const password = process.argv[5]
  const auth = username ? { [username]: password } : undefined
  new SimpleHttp({base, auth}).start(port)
}

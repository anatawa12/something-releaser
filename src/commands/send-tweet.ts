import {Cli, Command, Option} from 'clipanion'
import {OAuth} from 'oauth'
import {sendOAuthPost} from '../utils/oauth'

export async function sendTweet(args: string[]): Promise<void> {
  const cli = new Cli({
    binaryLabel: `send-tweet`,
    binaryName: `send-tweet`,
  })
  cli.register(SendTweetCommand)
  await cli.runExit(args)
}

class SendTweetCommand extends Command {
  readonly consumerKey = Option.String("-k,--consumer-key", {
    description: "OAuth Consumer Key",
    required: true,
  })
  readonly consumerSecret = Option.String("-t,--consumer-secret", {
    description: "OAuth Consumer Secret",
    required: true,
  })
  readonly accessToken = Option.String("-a,--access,--access-token", {
    description: "OAuth User Access Token",
    required: true,
  })
  readonly accessSecret = Option.String("-s,--secret,--access-secret", {
    description: "OAuth User Access Token Secret",
    required: true,
  })

  async execute(): Promise<number | void> {
    const chunks = []
    for await (const chunk of this.context.stdin) {
      chunks.push(Buffer.from(chunk))
    }
    const statusBody = Buffer.concat(chunks).toString("utf-8")

    const oauth = new OAuth(
      'https://api.twitter.com/oauth/request_token',
      'https://api.twitter.com/oauth/access_token',
      this.consumerKey,
      this.consumerSecret,
      '1.0',
      null,
      'HMAC-SHA1',
    )
    oauth.post(
      'https://api.twitter.com/1.1/statuses/update.json', 
      this.accessToken,
      this.accessSecret,
      new URLSearchParams({status: statusBody}).toString(),
      'application/x-www-form-urlencoded',
    )

    const response = await sendOAuthPost({
      consumerKey: this.consumerKey,
      consumerSecret: this.consumerSecret,
      url: 'https://api.twitter.com/1.1/statuses/update.json',
      accessToken: this.accessToken,
      accessSecret: this.accessSecret,
      body: {status: statusBody},
    })
    this.context.stdout.write(response.body)
    this.context.stdout.write('\n')
    // if OK, return zero
    if (!response.statusCode)
      return 0
    this.context.stderr.write(`status:${response.statusCode}\n`)
    // if not ok, return by status code
    if (500 <= response.statusCode)
      return 1 // 1: server internal error
    if (401 === response.statusCode)
      return 2 // 2: auth error
    if (403 === response.statusCode)
      return 2 // 2: auth error
    if (400 <= response.statusCode)
      return -2 // -2/254: other client errors
    return -1 // -1/255: unknown eror
  }
}

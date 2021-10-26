import {URL} from 'url'
import {Cli, Command, Option} from 'clipanion'
import fetch from 'node-fetch'

export async function sendDiscord(args: string[]): Promise<void> {
  const cli = new Cli({
    binaryLabel: `send-tweet`,
    binaryName: `send-tweet`,
  })
  cli.register(SendDiscordCommand)
  await cli.runExit(args)
}

class SendDiscordCommand extends Command {
  readonly webhookId = Option.String("-i,--webhook-id", {
    description: "The ID of webhook",
    required: true,
  })
  readonly webhookToken = Option.String("-t,--webhook-token", {
    description: "The token of webhook",
    required: true,
  })
  readonly thread = Option.String("-h,--thread", {
    description: "The thread the message will be sent",
  })
  readonly name = Option.String("-n,--name", {
    description: "The name the message will be sent as",
  })
  readonly avatar = Option.String("-a,--avatar", {
    description: "The avatar the message will be sent as",
  })

  async execute(): Promise<number | void> {
    const chunks = []
    for await (const chunk of this.context.stdin) {
      chunks.push(Buffer.from(chunk))
    }
    const messageBody = Buffer.concat(chunks).toString("utf-8")

    const url = new URL(`https://discord.com/api/webhooks/${this.webhookId}/${this.webhookToken}`)
    if (this.thread)
      url.searchParams.append('thread_id', this.thread)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const bodyJson: any = {content: messageBody}
    if (this.name)
      bodyJson.username = this.name
    if (this.avatar)
      bodyJson.avatar_url = this.avatar
    const response = await fetch(url.toString(), {
      method: 'POST',
      body: JSON.stringify(bodyJson),
      headers: {
        "Content-Type": 'application/json',
        "User-Agent": 'something-releaser',
      },
    })
    const responseBody = await response.text()
    this.context.stdout.write(responseBody)
    this.context.stdout.write('\n')
    // if OK, return zero
    if (200 <= response.status && response.status < 300)
      return 0
    this.context.stderr.write(`status:${response.status}\n`)
    return -1
  }
}

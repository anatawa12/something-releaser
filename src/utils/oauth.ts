import {OAuth} from 'oauth'

interface Response {
  // undefined for 2xx
  statusCode?: number,
  body: string,
}

interface BasicOptions {
  consumerKey: string
  consumerSecret: string
  url: string
  accessToken: string
  accessSecret: string
}

type SendingOptions = BasicOptions & (
  | {body: string | Buffer, contentType: string}
  | {body: { [P in string]?: string }}
)

export async function sendOAuthPost(options: SendingOptions): Promise<Response> {
  return new Promise<Response>((resolve, reject) => {
    const oauth = new OAuth(
      "UNUSED",
      "UNUSED",
      options.consumerKey,
      options.consumerSecret,
      '1.0',
      null,
      'HMAC-SHA1',
    )
    oauth.post(
      options.url,
      options.accessToken,
      options.accessSecret,
      options.body,
      'contentType' in options ? options.contentType : undefined,
      (err, result) => {
        if (result) {
          resolve({
            statusCode: err?.statusCode ?? undefined,
            // buffer will never be passed. see implementation
            body: result as string,
          })
        } else {
          reject(err)
        }
      },
    )
  })
}

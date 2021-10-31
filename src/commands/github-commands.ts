import {
  addPath, AnnotationProperties,
  endGroup, error,
  exportVariable,
  getBooleanInput,
  getInput, notice,
  setOutput,
  setSecret,
  startGroup, warning,
} from '@actions/core'
import {asPair, logicFailre, throws} from '../utils'

export type GithubCommands =
  | ['gh-get-input', string]
  | ['gh-get-input-boolean', string]
  | ['gh-set-output', string, string]
  | ['gh-export-variable', string, string]
  | ['gh-set-secret', string]
  | ['gh-add-path', string]
  | ['gh-group-start', string]
  | ['gh-group-end']
  | ['gh-error', string, ...string[]]
  | ['gh-warning', string, ...string[]]
  | ['gh-notice', string, ...string[]]

export async function runGithubCommands(args: GithubCommands): Promise<void> {
  switch (args[0]) {
    case 'gh-get-input': return inputCommand(getInput, args)
    case 'gh-get-input-boolean': return inputCommand(getBooleanInput, args)
    case 'gh-set-output': return valueSetCommand(setOutput, 'output', args)
    case 'gh-export-variable': return valueSetCommand(exportVariable, 'output', args)
    case 'gh-set-secret': return valueAddCommand(setSecret, 'secret', args)
    case 'gh-add-path': return valueAddCommand(addPath, 'secret', args)
    case 'gh-group-start': return valueAddCommand(startGroup, 'secret', args)
    case 'gh-group-end': return endGroup()

    case 'gh-error':
    case 'gh-warning':
    case 'gh-notice': 
      return annotationCommand(args)
    default:
      logicFailre("invalid github command", args[0])
  }
}

function inputCommand(func: (v: string) => unknown, args: [string, string]): void {
  process.stdout.write(String(func(args[1] ?? throws(new Error("input name not specified")))))
}

function valueSetCommand(
  func: (k: string, v: string) => void,
  target: string,
  args: [string, string, string],
): void {
  func(
    args[1] ?? throws(new Error(`${target} name not specified`)),
    args[2] ?? throws(new Error(`${target} value not specified`)),
  )
}

function valueAddCommand(
  func: (v: string) => void,
  target: string,
  args: [string, string],
): void {
  func(args[1] ?? throws(new Error(`${target} not specified`)))
}

function annotationCommand(
  args: GithubCommands & ['gh-error' | 'gh-warning' | 'gh-notice', ...string[]],
): void {
  function parseIntArg(value: string, name: string): number {
    const input = value ?? throws(new Error(`no value specified for ${name}`))
    const int = parseInt(input)
    if (!Number.isInteger(int))
      new Error(`un-parsable value is specified for ${name}: ${value}`)
    return int
  }

  const command = args[0] === 'gh-error' ? error
    : args[0] === 'gh-warning' ? warning
    : notice

  const message = args[1] ?? throws(new Error("message not specified"))
  const props: AnnotationProperties = {}

  for (let i = 2; i < args.length; i++) {
    switch (args[1]) {
      case '-t':
      case '--title':
        props.title = args[2] ?? throws(new Error("no value specified for --title"))
        break
      case '-f':
      case '--file':
        props.file = args[2] ?? throws(new Error("no value specified for --file"))
        break
      case '-s':
      case '--start-line':
        props.startLine = parseIntArg(args[2], '--start-line')
        break
      case '-e':
      case '--end-line':
        props.startLine = parseIntArg(args[2], '--end-line')
        break
      case '-c':
      case '--column': {
        const [start, end] = asPair(args[2] ?? throws(new Error("no value specified for --column")),
          ':', false)
        props.startColumn = parseIntArg(start, 'start column')
        // undefined: defaults to endColumn
        props.endColumn = end ? parseIntArg(end, 'start column') : undefined
        break
      }
      default:
        throw new Error(`unknown option: ${args[1]}`)
    }
  }
  // do not check endColumn because it will be set if startColumn is set
  if (props.endLine && props.endLine !== props.startLine && props.startColumn != null) {
    throw new Error("you can't set column if you specified different start line and end line")
  }

  command(message, props)
}

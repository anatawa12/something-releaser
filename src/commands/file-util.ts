import {promises as fs} from 'fs'
import {PropertiesFile} from '../files/properties'
import {throws} from '../utils'

export async function fileUtil(args: string[]): Promise<void> {
  switch (args[0]) {
    case 'properties':
      await propertiesUtil(args.slice(1))
      break
    default:
      throw new Error(`unknown file format: ${args[0]}`)
  }
}

async function propertiesUtil(args: string[]): Promise<void> {
  const file = args[1] ?? throws(new Error(`no file specified`))
  const property = args[2] ?? throws(new Error(`no property specified`))
  switch (args[0]) {
    case 'get': {
      const propertiesFile = PropertiesFile.parse(await fs.readFile(file, {encoding:'utf-8'}))
      const value = propertiesFile.get(property)
      process.stdout.write(value ?? '')
      if (value == null)
        process.exit(1)
      break
    }
    case 'set': {
      const value = args[3] ?? throws(new Error(`no property specified`))
      const propertiesFile = PropertiesFile.parse(await fs.readFile(file, {encoding:'utf-8'}))
      propertiesFile.set(property, value)
      await fs.writeFile(file, propertiesFile.toSource())
      break
    }
    default:
      throw new Error(`unknown command for properties: ${args[0]}`)
  }
}

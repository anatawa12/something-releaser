/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */
const {mkdirSync, promises: fs} = require('fs')
const {join} = require('path')
const cmdShim = require('cmd-shim')

const commands = [
  'something-releaser',
  'set-git-user',
  'get-version',
  'set-version',
  'generate-changelog',
  'prepare-gradle-maven',
  'prepare-gradle-plugin-portal',
  'prepare-gradle-intellij',
];

const jsDir = join(__dirname, '..', 'bin')
const winInstallTo = join(__dirname, '..', 'path-win')
const posixInstallTo = join(__dirname, '..', 'path-posix')

mkdirSync(jsDir, { recursive: true })
mkdirSync(winInstallTo, { recursive: true })
mkdirSync(posixInstallTo, { recursive: true })

// noinspection JSIgnoredPromiseFromCall
Promise.all(commands.map(makeLinkFor))

async function makeLinkFor(command) {
  const body = `#!/usr/bin/env node
/* eslint-disable eslint-comments/no-unlimited-disable */
/* eslint-disable */

const {main} = require('../dist/index')

main('${command}', ...process.argv.slice(2))
`

  await fs.writeFile(join(jsDir, `${command}.js`), body, 'utf8')
  await cmdShim(join(jsDir, `${command}.js`), join(winInstallTo, command))
  await ignoreENOENT(fs.unlink(join(posixInstallTo, command)))
  await fs.symlink(join('..', 'bin', `${command}.js`), join(posixInstallTo, command))
  await fs.chmod(join(posixInstallTo, command), '755')
}

/**
 * @template T
 * @param promise {Promise<T>}
 * @return {Promise<T | undefined>}
 */
function ignoreENOENT(promise) {
  return promise.catch(e => {
    if (e.code === 'ENOENT')
      return undefined
    else
      throw e
  })
}
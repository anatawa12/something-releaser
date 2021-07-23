import {Version} from '../types'
import {VersionChangers} from '../version-changer'

export async function setCurrentVersion(changers: VersionChangers): Promise<Version> {
  const currentSnapshot = await changers.getVersionName()
  if (!currentSnapshot.snapshot)
    throw new Error("current version is not a snapshot version!")
  const currentVersion = currentSnapshot.unSnapshot()
  await changers.setVersionName(currentVersion)
  return currentVersion
}

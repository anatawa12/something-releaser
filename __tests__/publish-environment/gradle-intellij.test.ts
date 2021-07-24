import {expect, test} from '@jest/globals'
import {GradleIntellij} from '../../src/publish-environment/gradle-intellij'

test("generated init script", () => {
  const maven = new GradleIntellij({
    token: 'gradle-intellij-token-here',
  })

  expect(maven.generateInitScript())
    .toBe(`afterProject { proj ->
  if (proj.plugins.findPlugin("org.jetbrains.intellij") == null) return
  proj.tasks.publishPlugin {
    token = "gradle-intellij-token-here"
  }
}
`)
})

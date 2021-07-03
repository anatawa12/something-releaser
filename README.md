# something-releaser

The actions for me to publish files

## subactions

### `anatawa12/something-releaser/set_user@v1`

The action to set username and email to commit.
This sets `user.name` and `user.email` in global.

#### inputs

- user: The user the environment will be. \[default: github-actions[bot\]\]
- token: The github api token. \[default: ${{ github.token }}\]

#### outputs
No outputs.

### `anatawa12/something-releaser/verup@v1`

The action to upgrade version configurations.

#### inputs

- changelog: Path to changelog file. \[default: CHANGELOG.md\]
- repository: The url to this repository. \[default: ${{ env.GITHUB_SERVER_URL }}/${{ env.GITHUB_REPOSITORY }}\]
- version_changers: The comma separated [version changer](#version-changer) list. required

#### outputs
- changelog_html: the path to the changelog in HTML
- changelog_markdown: the path to the changelog in markdown
- version: the current version
- next_version: the next version

### `anatawa12/something-releaser/publish@v1`

The action to build and publish artifacts.

#### inputs

- publishers: The comma separated [publisher](#publisher) list. required
- changelog_html: the path to the changelog in html. required
- changelog_markdown: the path to the changelog in markdown. required
- version_name: The name of new version. required
- dry_run: true if do not actually run publish

#### outputs
No outputs in this action.

### `anatawa12/something-releaser/verup_next@v1`

The action to upgrade version configurations.

#### inputs

- new_version: The next version. required
- version_changers: The comma separated [version changer](#version-changer) list. required

#### outputs
No outputs in this action.

## version changer

### `gradle-properties`

Upgrades `version` property in `gradle-properties`.

## Publisher

### `gradle-maven`

Publishes gradle artifact to [ossrh] staging maven repository.

#### Expected Environment Variables
##### `GRADLE_MAVEN_AUTH`
The value must be ``<user-name>:<password>`` to publish to [ossrh].

##### `GPG_PRIVATE_KEY`
The value must be armored PGP Key to sign artifact.

##### `GPG_PRIVATE_PASS`
The passphrase for PGP Key specified in `GPG_PRIVATE_PASS`

### `gradle-plugin-portal`

Publishes gradle plugin to gradle plugin portal.

#### Expected Environment Variables
##### `GRADLE_PUBLISH_AUTH`
The value must be ``<key>:<secret>`` to publish to [ossrh].

### `gradle-intellij-publisher`

Publishes intellij plugin to intellij plugin portal.

#### Expected Environment Variables
##### `GRADLE_INTELLIJ_TOKEN`
The API TOKEN for intellij plugin portal.

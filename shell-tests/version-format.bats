#!/usr/bin/env bats

@test 'version-format simple version' {
  export SOMETHING_RELEASER_SEMVER=1
  result="$(version-format 1.0)"
  [ "$result" = 1.0 ]
}

@test 'version-format SNAPSHOT version' {
  export SOMETHING_RELEASER_SEMVER=1
  result="$(version-format 1.0-SNAPSHOT)"
  [ "$result" = 1.0-SNAPSHOT ]
}

@test 'version-format semver compatible mode' {
  export SOMETHING_RELEASER_SEMVER=1
  result="$(version-format 1.0-alpha6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-format semver compatible mode' {
  export SOMETHING_RELEASER_SEMVER=1
  result="$(version-format 1.0-beta6)"
  [ "$result" = 1.0-beta.6 ]
}

@test 'version-format semver compatible mode' {
  export SOMETHING_RELEASER_SEMVER=1
  result="$(version-format 1.0-rc6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-format traditional mode' {
  result="$(version-format 1.0-alpha.6)"
  [ "$result" = 1.0-alpha6 ]
}

@test 'version-format traditional mode' {
  result="$(version-format 1.0-beta.6)"
  [ "$result" = 1.0-beta6 ]
}

@test 'version-format traditional mode' {
  result="$(version-format 1.0-rc.6)"
  [ "$result" = 1.0-rc6 ]
}

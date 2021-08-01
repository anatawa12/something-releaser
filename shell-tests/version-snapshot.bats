#!/usr/bin/env bats

@test 'version-snapshot non snapshot major only' {
  result="$(version-snapshot 1)"
  [ "$result" = 1-SNAPSHOT ]
}

@test 'version-snapshot snapshot major only' {
  result="$(version-snapshot 1-SNAPSHOT)"
  [ "$result" = 1-SNAPSHOT ]
}

@test 'version-snapshot non snapshot major.minor' {
  result="$(version-snapshot 1.0)"
  [ "$result" = 1.0-SNAPSHOT ]
}

@test 'version-snapshot snapshot major.minor' {
  result="$(version-snapshot 1.0-SNAPSHOT)"
  [ "$result" = 1.0-SNAPSHOT ]
}

@test 'version-snapshot non snapshot major.minor.patch' {
  result="$(version-snapshot 1.0.0)"
  [ "$result" = 1.0.0-SNAPSHOT ]
}

@test 'version-snapshot snapshot major.minor.patch' {
  result="$(version-snapshot 1.0.0-SNAPSHOT)"
  [ "$result" = 1.0.0-SNAPSHOT ]
}

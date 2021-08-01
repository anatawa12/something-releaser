#!/usr/bin/env bats

@test 'version-next non snapshot major only' {
  output="$(version-next 1)"
  [ "$output" = 2 ]
}

@test 'version-next snapshot major only' {
  output="$(version-next 1-SNAPSHOT)"
  [ "$output" = 2-SNAPSHOT ]
}

@test 'version-next non snapshot major.minor' {
  output="$(version-next 1.0)"
  [ "$output" = 1.1 ]
}

@test 'version-next snapshot major.minor' {
  output="$(version-next 1.0-SNAPSHOT)"
  [ "$output" = 1.1-SNAPSHOT ]
}

@test 'version-next non snapshot major.minor.patch' {
  output="$(version-next 1.0.0)"
  [ "$output" = 1.0.1 ]
}

@test 'version-next snapshot major.minor.patch' {
  output="$(version-next 1.0.0-SNAPSHOT)"
  [ "$output" = 1.0.1-SNAPSHOT ]
}

#!/usr/bin/env bats

@test 'version-major from stable' {
  result="$(version-major 1.2.3)"
  [ "$result" = 1 ]
}

@test 'version-major from alpha' {
  result="$(version-major 1.2.3-alpha.1)"
  [ "$result" = 1-alpha.1 ]
}

@test 'version-major from beta' {
  result="$(version-major 1.2.3-beta.1)"
  [ "$result" = 1-beta.1 ]
}

@test 'version-major from candidate' {
  result="$(version-major 1.2.3-rc.1)"
  [ "$result" = 1-rc.1 ]
}

@test 'version-major from snapshot' {
  result="$(version-major 1.2.3-SNAPSHOT)"
  [ "$result" = 1-SNAPSHOT ]
}

@test 'version-major from major only' {
  result="$(version-major 1)"
  [ "$result" = 1 ]
}

@test 'version-major from major.minor' {
  result="$(version-major 1.2)"
  [ "$result" = 1 ]
}

@test 'version-major from major.minor.patch' {
  result="$(version-major 1.2.3)"
  [ "$result" = 1 ]
}

@test 'version-major pipe mode' {
  result="$(echo 1.2.3 | version-major)"
  [ "$result" = 1 ]
}

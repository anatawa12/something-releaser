#!/usr/bin/env bats

@test 'version-major from stable' {
  result="$(version-major 1.2.3)"
  [ "$result" = 1 ]
}

@test 'version-major from alpha' {
  result="$(version-major 1.2.3-alpha1)"
  [ "$result" = 1-alpha1 ]
}

@test 'version-major from beta' {
  result="$(version-major 1.2.3-beta1)"
  [ "$result" = 1-beta1 ]
}

@test 'version-major from candidate' {
  result="$(version-major 1.2.3-rc1)"
  [ "$result" = 1-rc1 ]
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

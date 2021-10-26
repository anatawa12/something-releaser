#!/usr/bin/env bats

@test 'version-beta defaults beta1' {
  result="$(version-beta 1.0)"
  [ "$result" = 1.0-beta1 ]
}

@test 'version-beta from alpha' {
  result="$(version-beta 1.0-alpha1 6)"
  [ "$result" = 1.0-beta6 ]
}

@test 'version-beta from beta' {
  result="$(version-beta 1.0-beta1 6)"
  [ "$result" = 1.0-beta6 ]
}

@test 'version-beta from candidate' {
  result="$(version-beta 1.0-rc1 6)"
  [ "$result" = 1.0-beta6 ]
}

@test 'version-beta from snapshot' {
  result="$(version-beta 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-beta6 ]
}

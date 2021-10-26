#!/usr/bin/env bats

@test 'version-cancicate defaults rc1' {
  result="$(version-cancicate 1.0 1)"
  [ "$result" = 1.0-rc1 ]
}

@test 'version-candidate from stable' {
  result="$(version-candidate 1.0 6)"
  [ "$result" = 1.0-rc6 ]
}

@test 'version-candidate from alpha' {
  result="$(version-candidate 1.0-alpha1 6)"
  [ "$result" = 1.0-rc6 ]
}

@test 'version-candidate from beta' {
  result="$(version-candidate 1.0-beta1 6)"
  [ "$result" = 1.0-rc6 ]
}

@test 'version-candidate from candidate' {
  result="$(version-candidate 1.0-rc1 6)"
  [ "$result" = 1.0-rc6 ]
}

@test 'version-candidate from snapshot' {
  result="$(version-candidate 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-rc6 ]
}

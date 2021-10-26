#!/usr/bin/env bats

@test 'version-alpha defaults alpha1' {
  result="$(version-alpha 1.0)"
  [ "$result" = 1.0-alpha1 ]
}

@test 'version-alpha from stable' {
  result="$(version-alpha 1.0 6)"
  [ "$result" = 1.0-alpha6 ]
}

@test 'version-alpha from alpha' {
  result="$(version-alpha 1.0-alpha1 6)"
  [ "$result" = 1.0-alpha6 ]
}

@test 'version-alpha from beta' {
  result="$(version-alpha 1.0-beta1 6)"
  [ "$result" = 1.0-alpha6 ]
}

@test 'version-alpha from candidate' {
  result="$(version-alpha 1.0-rc1 6)"
  [ "$result" = 1.0-alpha6 ]
}

@test 'version-alpha from snapshot' {
  result="$(version-alpha 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-alpha6 ]
}

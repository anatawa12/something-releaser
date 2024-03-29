#!/usr/bin/env bats

@test 'version-candidate defaults rc.1' {
  result="$(version-candidate 1.0)"
  [ "$result" = 1.0-rc.1 ]
}

@test 'version-candidate from stable' {
  result="$(version-candidate 1.0 6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-candidate from alpha' {
  result="$(version-candidate 1.0-alpha.1 6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-candidate from beta' {
  result="$(version-candidate 1.0-beta.1 6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-candidate from candidate' {
  result="$(version-candidate 1.0-rc.1 6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-candidate from snapshot' {
  result="$(version-candidate 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-rc.6 ]
}

@test 'version-candidate pipe mode' {
  result="$(echo 1.0-SNAPSHOT | version-candidate - 6)"
  [ "$result" = 1.0-rc.6 ]
}

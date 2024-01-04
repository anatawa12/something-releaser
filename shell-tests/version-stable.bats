#!/usr/bin/env bats

@test 'version-stable from stable' {
  result="$(version-stable 1.0)"
  [ "$result" = 1.0 ]
}

@test 'version-stable from alpha' {
  result="$(version-stable 1.0-alpha.1)"
  [ "$result" = 1.0 ]
}

@test 'version-stable from beta' {
  result="$(version-stable 1.0-beta.1)"
  [ "$result" = 1.0 ]
}

@test 'version-stable from candidate' {
  result="$(version-stable 1.0-rc.1)"
  [ "$result" = 1.0 ]
}

@test 'version-stable from snapshot' {
  result="$(version-stable 1.0-SNAPSHOT)"
  [ "$result" = 1.0 ]
}

@test 'version-stable pipe mode' {
  result="$(echo 1.0-SNAPSHOT | version-stable)"
  [ "$result" = 1.0 ]
}

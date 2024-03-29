#!/usr/bin/env bats

@test 'version-alpha defaults alpha.1' {
  result="$(version-alpha 1.0)"
  [ "$result" = 1.0-alpha.1 ]
}

@test 'version-alpha from stable' {
  result="$(version-alpha 1.0 6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-alpha from alpha' {
  result="$(version-alpha 1.0-alpha.1 6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-alpha from beta' {
  result="$(version-alpha 1.0-beta.1 6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-alpha from candidate' {
  result="$(version-alpha 1.0-rc.1 6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-alpha from snapshot' {
  result="$(version-alpha 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-alpha.6 ]
}

@test 'version-alpha pipe mode' {
  result="$(echo 1.0-SNAPSHOT | version-alpha - 6)"
  [ "$result" = 1.0-alpha.6 ]
}

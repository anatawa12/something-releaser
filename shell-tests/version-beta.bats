#!/usr/bin/env bats

@test 'version-beta defaults beta.1' {
  result="$(version-beta 1.0)"
  [ "$result" = 1.0-beta.1 ]
}

@test 'version-beta from alpha' {
  result="$(version-beta 1.0-alpha.1 6)"
  [ "$result" = 1.0-beta.6 ]
}

@test 'version-beta from beta' {
  result="$(version-beta 1.0-beta.1 6)"
  [ "$result" = 1.0-beta.6 ]
}

@test 'version-beta from candidate' {
  result="$(version-beta 1.0-rc.1 6)"
  [ "$result" = 1.0-beta.6 ]
}

@test 'version-beta from snapshot' {
  result="$(version-beta 1.0-SNAPSHOT 6)"
  [ "$result" = 1.0-beta.6 ]
}

@test 'version-beta pipe mode' {
  result="$(echo 1.0-SNAPSHOT | version-beta - 6)"
  [ "$result" = 1.0-beta.6 ]
}

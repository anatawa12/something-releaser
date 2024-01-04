#!/usr/bin/env bats

@test 'version-minor from stable' {
  result="$(version-minor 1.2.3)"
  [ "$result" = 1.2 ]
}

@test 'version-minor from alpha' {
  result="$(version-minor 1.2.3-alpha.1)"
  [ "$result" = 1.2-alpha.1 ]
}

@test 'version-minor from beta' {
  result="$(version-minor 1.2.3-beta.1)"
  [ "$result" = 1.2-beta.1 ]
}

@test 'version-minor from candidate' {
  result="$(version-minor 1.2.3-rc.1)"
  [ "$result" = 1.2-rc.1 ]
}

@test 'version-minor from snapshot' {
  result="$(version-minor 1.2.3-SNAPSHOT)"
  [ "$result" = 1.2-SNAPSHOT ]
}

@test 'version-minor from major only' {
  result="$(version-minor 1)"
  [ "$result" = 1.0 ]
}

@test 'version-minor from major.minor' {
  result="$(version-minor 1.2)"
  [ "$result" = 1.2 ]
}

@test 'version-minor from major.minor.patch' {
  result="$(version-minor 1.2.3)"
  [ "$result" = 1.2 ]
}

@test 'version-minor pipe mode' {
  result="$(echo 1.2.3 | version-minor)"
  [ "$result" = 1.2 ]
}

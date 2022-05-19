#!/usr/bin/env bats

@test 'version-patch from stable' {
  result="$(version-patch 1.2.3)"
  [ "$result" = 1.2.3 ]
}

@test 'version-patch from alpha' {
  result="$(version-patch 1.2.3-alpha1)"
  [ "$result" = 1.2.3-alpha1 ]
}

@test 'version-patch from beta' {
  result="$(version-patch 1.2.3-beta1)"
  [ "$result" = 1.2.3-beta1 ]
}

@test 'version-patch from candidate' {
  result="$(version-patch 1.2.3-rc1)"
  [ "$result" = 1.2.3-rc1 ]
}

@test 'version-patch from snapshot' {
  result="$(version-patch 1.2.3-SNAPSHOT)"
  [ "$result" = 1.2.3-SNAPSHOT ]
}

@test 'version-patch from major only' {
  result="$(version-patch 1)"
  [ "$result" = 1.0.0 ]
}

@test 'version-patch from major.minor' {
  result="$(version-patch 1.2)"
  [ "$result" = 1.2.0 ]
}

@test 'version-patch from major.minor.patch' {
  result="$(version-patch 1.2.3)"
  [ "$result" = 1.2.3 ]
}

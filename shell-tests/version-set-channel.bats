#!/usr/bin/env bats

@test 'version-set-channel to stable' {
  result="$(version-set-channel 1.0-rc.0 stable)"
  [ "$result" = 1.0 ]
}

@test 'version-set-channel to a' {
  result="$(version-set-channel 1.0 a)"
  [ "$result" = 1.0-alpha.1 ]
}

@test 'version-set-channel to alpha' {
  result="$(version-set-channel 1.0 alpha)"
  [ "$result" = 1.0-alpha.1 ]
}

@test 'version-set-channel to α' {
  result="$(version-set-channel 1.0 α)"
  [ "$result" = 1.0-alpha.1 ]
}

@test 'version-set-channel to b' {
  result="$(version-set-channel 1.0 b)"
  [ "$result" = 1.0-beta.1 ]
}

@test 'version-set-channel to beta' {
  result="$(version-set-channel 1.0 beta)"
  [ "$result" = 1.0-beta.1 ]
}

@test 'version-set-channel to β' {
  result="$(version-set-channel 1.0 β)"
  [ "$result" = 1.0-beta.1 ]
}

@test 'version-set-channel to rc' {
  result="$(version-set-channel 1.0 rc)"
  [ "$result" = 1.0-rc.1 ]
}

@test 'version-set-channel to candidate' {
  result="$(version-set-channel 1.0 candidate)"
  [ "$result" = 1.0-rc.1 ]
}

@test 'version-set-channel to snapshot' {
  result="$(version-set-channel 1.0 snapshot)"
  [ "$result" = 1.0-SNAPSHOT ]
}

@test 'version-set-channel pipe mode' {
  result="$(echo 1.0 | version-set-channel - snapshot)"
  [ "$result" = 1.0-SNAPSHOT ]
}

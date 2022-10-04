#!/usr/bin/env bats

@test 'version-unsnapshot non snapshot major only' {
  result="$(version-unsnapshot 1)"
  [ "$result" = 1 ]
}

@test 'version-unsnapshot snapshot major only' {
  result="$(version-unsnapshot 1-SNAPSHOT)"
  [ "$result" = 1 ]
}

@test 'version-unsnapshot non snapshot major.minor' {
  result="$(version-unsnapshot 1.0)"
  [ "$result" = 1.0 ]
}

@test 'version-unsnapshot snapshot major.minor' {
  result="$(version-unsnapshot 1.0-SNAPSHOT)"
  [ "$result" = 1.0 ]
}

@test 'version-unsnapshot non snapshot major.minor.patch' {
  result="$(version-unsnapshot 1.0.0)"
  [ "$result" = 1.0.0 ]
}

@test 'version-unsnapshot pipe mode' {
  result="$(echo 1.0.0-SNAPSHOT | version-unsnapshot)"
  [ "$result" = 1.0.0 ]
}

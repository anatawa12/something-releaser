#!/usr/bin/env bats

@test 'version-get-channel stable' {
  result="$(version-get-channel 1.0)"
  [ "$result" = stable ]
}

@test 'version-get-channel alpha' {
  result="$(version-get-channel 1.0-alpha1)"
  [ "$result" = alpha ]
}

@test 'version-get-channel beta' {
  result="$(version-get-channel 1.0-beta1)"
  [ "$result" = beta ]
}

@test 'version-get-channel candidate' {
  result="$(version-get-channel 1.0-rc1)"
  [ "$result" = candidate ]
}

@test 'version-get-channel snapshot' {
  result="$(version-get-channel 1.0-SNAPSHOT)"
  [ "$result" = snapshot ]
}

@test 'version-get-channel pipe mode' {
  result="$(echo 1.0-SNAPSHOT | version-get-channel)"
  [ "$result" = snapshot ]
}

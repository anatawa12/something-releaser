#!/usr/bin/env bats

@test 'set-git-user github-actions' {
  set-git-user github-actions[bot]
  [ "$(git config user.name )" = 'github-actions[bot]' ]
  [ "$(git config user.email)" = '41898282+github-actions[bot]@users.noreply.github.com' ]
}

@test 'set-git-user octocat' {
  set-git-user octocat
  [ "$(git config user.name )" = 'octocat' ]
  [ "$(git config user.email)" = '583231+octocat@users.noreply.github.com' ]
}

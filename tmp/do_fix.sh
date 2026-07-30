#!/bin/bash
set -e
cd /Users/sohosoki/dev/fucso/bake-loose/.worktree/iddue/37
git rm -r --cached tmp >/dev/null
git commit -m "chore: remove worktree tmp files from tracked changes"

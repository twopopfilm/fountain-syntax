#!/usr/bin/env bash
# Fails unless a release tag is safe to publish. The publish workflow runs it first, on a tag push; run it by hand
# (no arguments) to rehearse a release before any tag exists.
#
# Usage: scripts/check-release.sh [TAG] [COMMIT]
#   TAG     the tag to publish; empty or missing means "v" plus the version in Cargo.toml
#   COMMIT  the commit the tag points at; defaults to HEAD
#
# Checks, in order, stopping at the first failure:
#   - TAG looks like vMAJOR.MINOR.PATCH
#   - TAG equals "v" plus the version in the [package] table of Cargo.toml
#   - CHANGELOG.md has a heading "## [<version>] - <date>"
#   - COMMIT is on origin/main (the workflow must fetch the full history, fetch-depth: 0)

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

TAG="${1:-}"
COMMIT="${2:-HEAD}"

fail() {
  echo "check-release: $1" >&2
  exit 1
}

# The first version line inside [package], not the first "version" anywhere in the file.
VERSION=$(awk '
  /^\[/ { in_package = ($0 == "[package]"); next }
  in_package && /^version[ \t]*=/ { split($0, parts, "\""); print parts[2]; exit }
' Cargo.toml)
[[ -n "$VERSION" ]] || fail "no version found in the [package] table of Cargo.toml"

[[ -n "$TAG" ]] || TAG="v$VERSION"

[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "tag $TAG is not of the form vMAJOR.MINOR.PATCH"

[[ "$TAG" == "v$VERSION" ]] || fail "tag $TAG does not match Cargo.toml version $VERSION"

HEADING="## [$VERSION] - "
awk -v prefix="$HEADING" 'index($0, prefix) == 1 { found = 1 } END { exit !found }' CHANGELOG.md ||
  fail "CHANGELOG.md has no heading starting \"$HEADING\" for version $VERSION"

git rev-parse --verify --quiet origin/main >/dev/null ||
  fail "origin/main not found; the workflow must fetch the full history (fetch-depth: 0)"
git merge-base --is-ancestor "$COMMIT" origin/main ||
  fail "commit $COMMIT is not on origin/main"

echo "check-release: ok ($TAG)"

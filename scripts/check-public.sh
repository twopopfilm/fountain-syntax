#!/usr/bin/env bash
# Fails when files or history would leak local paths, personal email addresses or listed names.
# The repository may go public at any time, so this runs before every commit and on every PR.
#
# Checks:
#   - every tracked or new (not ignored) file on disk
#   - commit messages and author/committer emails in the history
# Names to forbid are read from a private list outside the repository (one extended regex per line):
#   ${TWOPOP_PUBLICCHECK_FILE:-${XDG_CONFIG_HOME:-$HOME/.config}/twopop/publiccheck.local}
# A line containing "public-check: allow" is skipped.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

SELF="scripts/check-public.sh"
NAMES_FILE="${TWOPOP_PUBLICCHECK_FILE:-${XDG_CONFIG_HOME:-$HOME/.config}/twopop/publiccheck.local}"

PATH_PATTERN='(/Users/[A-Za-z0-9._-]+|/home/[A-Za-z0-9._-]+|[A-Za-z]:\\+Users\\+|~/dev/)'
EMAIL_PATTERN='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
EMAIL_ALLOWED='(noreply|example\.(com|org|net)|@localhost)'
ALLOW_MARKER='public-check: allow'

failures=0

report() {
  echo "check-public: $1" >&2
  failures=$((failures + 1))
}

scan_text() {
  local label="$1"
  local text="$2"
  local hits

  hits=$(grep -nE "$PATH_PATTERN" <<<"$text" | grep -v "$ALLOW_MARKER" || true)
  [[ -n "$hits" ]] && report "local path in $label:"$'\n'"$hits"

  hits=$(grep -noE "$EMAIL_PATTERN" <<<"$text" | grep -vE "$EMAIL_ALLOWED" || true)
  [[ -n "$hits" ]] && report "email address in $label:"$'\n'"$hits"

  if [[ -f "$NAMES_FILE" ]]; then
    while IFS= read -r name; do
      [[ -z "$name" || "$name" == \#* ]] && continue
      hits=$(grep -niE "$name" <<<"$text" | grep -v "$ALLOW_MARKER" || true)
      [[ -n "$hits" ]] && report "listed name in $label (pattern $name):"$'\n'"$hits"
    done <"$NAMES_FILE"
  fi
  return 0
}

while IFS= read -r -d '' file; do
  [[ "$file" == "$SELF" || ! -f "$file" ]] && continue
  grep -Iq . "$file" 2>/dev/null || continue # skip binary and empty files
  scan_text "$file" "$(cat "$file")"
done < <(git ls-files -z --cached --others --exclude-standard)

if git rev-parse --verify --quiet HEAD >/dev/null; then
  scan_text "commit messages" "$(git log --format='%H%n%B')"
  bad_ids=$(git log --format='%h %ae %ce' | grep -vE "^[0-9a-f]+ [^ ]*$EMAIL_ALLOWED[^ ]* [^ ]*$EMAIL_ALLOWED" || true)
  [[ -n "$bad_ids" ]] && report "commit author or committer email is not a noreply address:"$'\n'"$bad_ids"
fi

if [[ ! -f "$NAMES_FILE" ]]; then
  echo "check-public: note: no private name list at $NAMES_FILE; names were not checked" >&2
fi

if ((failures > 0)); then
  echo "check-public: $failures problem(s) found" >&2
  exit 1
fi
echo "check-public: ok"

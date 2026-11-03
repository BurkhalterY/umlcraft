#!/usr/bin/env bash
# The version of the next release, computed from the Conventional Commits since the last `vX.Y.Z` tag.
#
#   packaging/next-version.sh                 print the next version; print nothing if no commit since the last tag
#   packaging/next-version.sh --always        print a version even without new commits (test builds)
#   packaging/next-version.sh --notes FILE    also write the release notes (Markdown) to FILE
#
# Bump rules (semver; before 1.0 everything moves down one place, as Cargo reads 0.x versions):
#   breaking change (`type!:` or a `BREAKING CHANGE:` footer)  0.x: minor   1.x+: major
#   `feat:`                                                     0.x: patch   1.x+: minor
#   anything else                                               patch
# A higher version written in Cargo.toml wins, so `version = "1.0.0"` there makes the next release 1.0.0.
set -euo pipefail
cd "$(dirname "$0")/.."

always=0
notes=""
while [ $# -gt 0 ]; do
  case "$1" in
    --always) always=1 ;;
    --notes) notes="$2"; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

last=$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --merged HEAD | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n1 || true)
if [ -n "$last" ]; then
  range="$last..HEAD"
  base="${last#v}"
else
  range="HEAD"
  base="0.0.0"
fi

count=$(git rev-list --count "$range")
if [ "$count" -eq 0 ] && [ "$always" -eq 0 ]; then
  exit 0
fi

conv='^[a-zA-Z]+(\([^)]*\))?'
subjects=$(git log --format=%s "$range")
breaking=0
feat=0
if grep -Eq "$conv!:" <<<"$subjects" || git log --format=%b "$range" | grep -Eq '^BREAKING[ -]CHANGE:'; then
  breaking=1
fi
if grep -Eq '^feat(\([^)]*\))?!?:' <<<"$subjects"; then
  feat=1
fi

IFS=. read -r major minor patch <<<"$base"
if [ "$major" -eq 0 ]; then
  if [ "$breaking" -eq 1 ]; then minor=$((minor + 1)); patch=0; else patch=$((patch + 1)); fi
else
  if [ "$breaking" -eq 1 ]; then major=$((major + 1)); minor=0; patch=0
  elif [ "$feat" -eq 1 ]; then minor=$((minor + 1)); patch=0
  else patch=$((patch + 1)); fi
fi
next="$major.$minor.$patch"

cargo=$(sed -n 's/^version = "\([0-9]*\.[0-9]*\.[0-9]*\)"$/\1/p' Cargo.toml | head -n1)
if [ -n "$cargo" ] && [ "$(printf '%s\n%s\n' "$next" "$cargo" | sort -V | tail -n1)" != "$next" ]; then
  next="$cargo"
fi

if [ -n "$notes" ]; then
  # one line per commit, grouped by kind: "- **scope:** description (abc1234)"
  {
    for kind in breaking feat fix other; do
      case "$kind" in
        breaking) title="Breaking changes" ;;
        feat) title="Features" ;;
        fix) title="Fixes" ;;
        other) title="Other changes" ;;
      esac
      lines=$(git log --format='%h%x09%s%x09%b%x1e' "$range" | KIND="$kind" awk -v RS='\036' -F'\t' '
        NF >= 2 {
          sha = $1; gsub(/^[ \n]+/, "", sha); subj = $2; body = $3
          bang = subj ~ /^[a-zA-Z]+(\([^)]*\))?!:/
          brk = bang || body ~ /(^|\n)BREAKING[ -]CHANGE:/
          type = ""; scope = ""; text = subj
          if (match(subj, /^[a-zA-Z]+(\([^)]*\))?!?: /)) {
            head = substr(subj, 1, RLENGTH - 2); text = substr(subj, RLENGTH + 1)
            sub(/!$/, "", head)
            if (match(head, /\(.*\)/)) { scope = substr(head, RSTART + 1, RLENGTH - 2); type = substr(head, 1, RSTART - 1) } else type = head
          }
          kind = brk ? "breaking" : (type == "feat" ? "feat" : (type == "fix" ? "fix" : "other"))
          if (kind == ENVIRON["KIND"]) print "- " (scope != "" ? "**" scope ":** " : "") text " (" sha ")"
        }')
      if [ -n "$lines" ]; then
        printf '## %s\n\n%s\n\n' "$title" "$lines"
      fi
    done
    if [ -n "$last" ]; then
      printf 'All commits since %s: https://github.com/BurkhalterY/umlcraft/compare/%s...v%s\n' "$last" "$last" "$next"
    fi
  } >"$notes"
fi

echo "$next"

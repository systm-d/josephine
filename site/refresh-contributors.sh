#!/usr/bin/env bash
# Refresh the contributors the site thanks: site/data/contributors.json and the
# avatars it points at.
#
# The avatars are downloaded here, at build time, and served from the site
# itself. The page must not hotlink avatars.githubusercontent.com: this site
# makes no third-party request at all today, and a project whose whole argument
# is "nothing leaves your machine" should not hand every visitor's IP to GitHub
# to draw two faces.
#
# Both outputs are committed. They are the fallback: `zola build` works offline,
# and a GitHub hiccup on deploy day leaves the last known list standing instead
# of failing the site.
set -euo pipefail

repo="${GH_REPO:-systm-d/josephine}"
here="$(cd "$(dirname "$0")" && pwd)"
data="$here/data/contributors.json"
avatars="$here/static/contributors"

# Bots contribute, but they are not who this section thanks. `claude` is left
# out by name — the commits are real and sit in the history for anyone to read;
# the section credits the people.
exclude='["dependabot[bot]", "github-actions[bot]", "claude"]'

people=$(gh api "repos/${repo}/contributors" --paginate | jq -c --argjson x "$exclude" '
  [ .[]
    | select(.type == "User")
    | select([.login] | inside($x) | not)
    | { login, contributions, avatar_url,
        avatar: ("contributors/" + .login + ".png") } ]')

# An empty list means the API answered but told us nothing useful. Keep what is
# committed rather than shipping a section that thanks nobody.
if [ "$(jq 'length' <<<"$people")" -eq 0 ]; then
  echo "no human contributors returned — keeping the committed list" >&2
  exit 1
fi

mkdir -p "$avatars" "$(dirname "$data")"
while IFS=$'\t' read -r login url; do
  # avatar_url already carries a query (?v=4), hence &s=.
  curl -fsSL "${url}&s=160" -o "${avatars}/${login}.png"
  echo "  avatar: ${login}"
done < <(jq -r '.[] | "\(.login)\t\(.avatar_url)"' <<<"$people")

jq 'map(del(.avatar_url))' <<<"$people" > "$data"
echo "wrote $(jq 'length' "$data") contributors to ${data#"$here/"}"

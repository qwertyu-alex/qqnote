#!/bin/bash
# Builds qqnote for Apple silicon and Intel, signs it with the Developer ID certificate, has
# Apple notarize it, and publishes it as a GitHub release. Installed copies update themselves
# from the newest release.
#
#   pnpm release 0.2.0
#
# The one-time setup is under "Releasing" in the README.
set -euo pipefail

REPO=qwertyu-alex/qqnote
# Saved with `xcrun notarytool store-credentials qqnote`.
NOTARY_PROFILE=qqnote
UPDATER_KEY="$HOME/.tauri/qqnote.key"
# The keychain item holding the updater key's password.
UPDATER_KEY_PASSWORD_ITEM=qqnote-updater-key

VERSION=${1:-}
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Usage: pnpm release <version>, e.g. pnpm release 0.2.0" >&2
  exit 1
fi
TAG="v$VERSION"

cd "$(dirname "$0")/.."

step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
fail() { printf '\033[31mError:\033[0m %s\n' "$*" >&2; exit 1; }

step "Checking the setup"
[[ -z $(git status --porcelain) ]] || fail "Commit or stash your changes first, so the release matches a commit."
git rev-parse -q --verify "refs/tags/$TAG" >/dev/null && fail "The tag $TAG already exists."
[[ -z $(git ls-remote --tags origin "refs/tags/$TAG") ]] || fail "The tag $TAG is already on GitHub."
IDENTITY=$(node -p 'require("./src-tauri/tauri.release.conf.json").bundle.macOS.signingIdentity')
security find-identity -v -p codesigning | grep -qF "\"$IDENTITY\"" ||
  fail "The certificate \"$IDENTITY\" isn't in your keychain."
[[ -f $UPDATER_KEY ]] || fail "The updater signing key $UPDATER_KEY is missing."
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=$(security find-generic-password -s "$UPDATER_KEY_PASSWORD_ITEM" -w) ||
  fail "The updater key's password isn't in your keychain (item \"$UPDATER_KEY_PASSWORD_ITEM\")."
xcrun notarytool history --keychain-profile "$NOTARY_PROFILE" >/dev/null ||
  fail "No notarization credentials. Save them with: xcrun notarytool store-credentials $NOTARY_PROFILE"
gh auth status >/dev/null 2>&1 || fail "Log in to GitHub first: gh auth login"

step "Setting the version to $VERSION"
node -e '
  const fs = require("fs");
  for (const file of ["package.json", "src-tauri/tauri.conf.json"]) {
    const json = JSON.parse(fs.readFileSync(file, "utf8"));
    json.version = process.argv[1];
    fs.writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
  }
' "$VERSION"
sed -i '' -E "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 >/dev/null # Updates Cargo.lock
# Nothing changes when a failed release is run again.
if [[ -n $(git status --porcelain) ]]; then
  git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
  git commit -q -m "chore: release $TAG"
fi

step "Building and signing"
# Tauri would notarize the app too if these were set, but it can't use a keychain profile.
unset APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_PATH
export TAURI_SIGNING_PRIVATE_KEY="$UPDATER_KEY" TAURI_SIGNING_PRIVATE_KEY_PASSWORD
pnpm tauri build --target universal-apple-darwin --bundles app --config src-tauri/tauri.release.conf.json

BUNDLE=src-tauri/target/universal-apple-darwin/release/bundle/macos
APP="$BUNDLE/qqnote.app"
OUT="src-tauri/target/release-$VERSION"
rm -rf "$OUT"
mkdir -p "$OUT"
codesign --verify --strict "$APP" || fail "The app's signature doesn't verify."

notarize() {
  local file=$1 result id status
  # Exits non-zero when Apple rejects the file; the status below says so.
  result=$(xcrun notarytool submit "$file" --keychain-profile "$NOTARY_PROFILE" --wait --output-format json) || true
  id=$(node -p 'JSON.parse(process.argv[1]).id' "$result") || fail "notarytool failed: $result"
  status=$(node -p 'JSON.parse(process.argv[1]).status' "$result")
  if [[ $status != Accepted ]]; then
    xcrun notarytool log "$id" --keychain-profile "$NOTARY_PROFILE" >&2 || true
    fail "Apple didn't notarize $(basename "$file"): $status"
  fi
}

step "Notarizing the app (usually a few minutes)"
ditto -c -k --keepParent "$APP" "$OUT/qqnote.zip"
notarize "$OUT/qqnote.zip"
xcrun stapler staple -q "$APP"
spctl --assess --type execute "$APP" || fail "Gatekeeper rejects the app."

step "Making the disk image"
DMG="$OUT/qqnote.dmg"
mkdir "$OUT/dmg"
ditto "$APP" "$OUT/dmg/qqnote.app"
ln -s /Applications "$OUT/dmg/Applications"
hdiutil create -quiet -volname qqnote -srcfolder "$OUT/dmg" -fs HFS+ -format UDZO -ov "$DMG"
codesign --sign "$IDENTITY" --timestamp "$DMG"
notarize "$DMG"
xcrun stapler staple -q "$DMG"
spctl --assess --type open --context context:primary-signature "$DMG" ||
  fail "Gatekeeper rejects the disk image."

step "Writing latest.json"
# The update is the app as Tauri signed it, from before stapling: the notarization covers it
# all the same, and re-signing the archive would drop the version the updater requires.
cp "$BUNDLE/qqnote.app.tar.gz" "$OUT/"
node -e '
  const fs = require("fs");
  const [version, repo, sigFile, out] = process.argv.slice(1);
  const update = {
    url: `https://github.com/${repo}/releases/download/v${version}/qqnote.app.tar.gz`,
    signature: fs.readFileSync(sigFile, "utf8"),
  };
  const manifest = {
    version,
    notes: `https://github.com/${repo}/releases/tag/v${version}`,
    pub_date: new Date().toISOString(),
    // One universal app for both.
    platforms: { "darwin-aarch64": update, "darwin-x86_64": update },
  };
  fs.writeFileSync(out, JSON.stringify(manifest, null, 2) + "\n");
' "$VERSION" "$REPO" "$BUNDLE/qqnote.app.tar.gz.sig" "$OUT/latest.json"

step "Ready to publish"
ls -lh "$DMG" "$OUT/qqnote.app.tar.gz" "$OUT/latest.json"
read -r -p "Publish qqnote $VERSION? Everyone running qqnote will update to it. [y/N] " answer
[[ $answer == [yY] ]] || fail "Not published. The files are in $OUT."

git tag -a "$TAG" -m "qqnote $VERSION"
git push -q origin HEAD "$TAG"
gh release create "$TAG" -R "$REPO" --verify-tag --title "qqnote $VERSION" --generate-notes \
  "$DMG" "$OUT/qqnote.app.tar.gz" "$OUT/latest.json"

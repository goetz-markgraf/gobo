#!/usr/bin/env bash
#
# release.sh – baut einen neuen gobo-Release und veröffentlicht ihn
# über GitHub Releases + aktualisiert das Homebrew-Tap.
#
# Nutzung:  ./scripts/release.sh [--force]
#
set -euo pipefail

# ---------------------------------------------------------------------------
# Konfiguration
# ---------------------------------------------------------------------------
GITHUB_USER="goetz-markgraf"
REPO_NAME="gobo"
TAP_NAME="homebrew-tap"
ARCH="aarch64-apple-darwin"

GITHUB_REPO="${GITHUB_USER}/${REPO_NAME}"
TAP_REPO="${GITHUB_USER}/${TAP_NAME}"
TAP_URL="git@github.com:${TAP_REPO}.git"

# Repo-Verzeichnis ermitteln (Skript liegt in scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
fi

# ---------------------------------------------------------------------------
# Hilfsfunktionen
# ---------------------------------------------------------------------------
die() { echo "FEHLER: $*" >&2; exit 1; }
info() { echo "› $*"; }
ok()   { echo "  ✓ $*"; }

cleanup() {
  [[ -n "${TMP_DIR:-}" && -d "${TMP_DIR:-}" ]] && rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# 1. Version aus Cargo.toml auslesen
# ---------------------------------------------------------------------------
read_version() {
  cd "$REPO_DIR"
  local ver
  ver=$(grep -E '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
  [[ -n "$ver" ]] || die "Keine version in Cargo.toml gefunden."
  echo "$ver"
}

# ---------------------------------------------------------------------------
# 2. Prüfen, ob Version gegenüber letztem Release verändert wurde
# ---------------------------------------------------------------------------
check_version_changed() {
  local current="$1"
  local latest
  latest=$(gh release list --repo "$GITHUB_REPO" --limit 1 \
            --json tagName --jq '.[0].tagName' 2>/dev/null || echo "")
  if [[ -z "$latest" ]]; then
    info "Noch kein vorheriger Release vorhanden – erstmaliger Release."
    return
  fi
  local latest_ver="${latest#v}"
  if [[ "$latest_ver" == "$current" ]]; then
    if [[ "$FORCE" -eq 1 ]]; then
      echo "WARNUNG: Version $current ist unverändert gegenüber letztem Release ($latest)."
      echo "         --force gesetzt, fahre trotzdem fort."
    else
      die "Version $current ist unverändert gegenüber letztem Release ($latest).
         Bitte erst version in Cargo.toml erhöhen, oder --force verwenden."
    fi
  else
    ok "Version $current (zuletzt: $latest)"
  fi
}

# ---------------------------------------------------------------------------
# 3. Arbeitsbaum sauber?
# ---------------------------------------------------------------------------
check_clean_tree() {
  cd "$REPO_DIR"
  if [[ -n "$(git status --porcelain)" ]]; then
    die "Arbeitsbaum ist nicht sauber. Bitte erst committen oder stashen."
  fi
  ok "Arbeitsbaum sauber."
}

# ---------------------------------------------------------------------------
# 4. Release-Build
# ---------------------------------------------------------------------------
build_binary() {
  cd "$REPO_DIR"
  info "Baue Release-Binary (aarch64-apple-darwin)…"
  cargo build --release
  [[ -f "target/release/gobo" ]] || die "Binary target/release/gobo fehlt nach Build."
  ok "Binary gebaut: $(file target/release/gobo | cut -d: -f2-)"
}

# ---------------------------------------------------------------------------
# 5. tar.gz erstellen
# ---------------------------------------------------------------------------
package_binary() {
  local version="$1"
  TMP_DIR=$(mktemp -d)
  mkdir -p "${TMP_DIR}/staging"
  cp "$REPO_DIR/target/release/gobo" "${TMP_DIR}/staging/gobo"
  ARCHIVE="${TMP_DIR}/gobo-${version}-${ARCH}.tar.gz"
  tar -czf "$ARCHIVE" -C "${TMP_DIR}/staging" gobo
  ok "Archiv: $(basename "$ARCHIVE") ($(du -h "$ARCHIVE" | cut -f1))"
}

# ---------------------------------------------------------------------------
# 6. Git-Tag anlegen und pushen
# ---------------------------------------------------------------------------
create_tag() {
  local version="$1"
  local tag="v${version}"
  cd "$REPO_DIR"
  if git rev-parse "$tag" >/dev/null 2>&1; then
    info "Tag $tag existiert bereits."
  else
    git tag "$tag"
    git push origin "$tag"
    ok "Tag $tag gepusht."
  fi
}

# ---------------------------------------------------------------------------
# 7. GitHub Release erstellen + Archiv hochladen
# ---------------------------------------------------------------------------
create_release() {
  local version="$1"
  local tag="v${version}"
  if gh release view "$tag" --repo "$GITHUB_REPO" >/dev/null 2>&1; then
    info "Release $tag existiert bereits – lade Archiv hoch."
    gh release upload "$tag" "$ARCHIVE" --repo "$GITHUB_REPO" --clobber
  else
    info "Erstelle GitHub Release $tag…"
    gh release create "$tag" \
      --repo "$GITHUB_REPO" \
      --title "gobo ${version}" \
      --generate-notes \
      "$ARCHIVE"
  fi
  ok "Release $tag veröffentlicht."
}

# ---------------------------------------------------------------------------
# 8. SHA256 berechnen
# ---------------------------------------------------------------------------
compute_sha() {
  SHA256=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
  ok "SHA256: $SHA256"
}

# ---------------------------------------------------------------------------
# 9. Homebrew-Tap aktualisieren
# ---------------------------------------------------------------------------
update_tap() {
  local version="$1"
  local tap_dir
  tap_dir=$(mktemp -d)
  info "Klone Tap-Repo $TAP_REPO…"
  git clone --quiet "$TAP_URL" "$tap_dir"
  cd "$tap_dir"

  mkdir -p Formula
  generate_formula "$version" > Formula/gobo.rb

  git add Formula/gobo.rb
  git commit --quiet -m "gobo ${version}"
  # Branch immer main heißen (sicher auch beim ersten Commit im leeren Tap-Repo)
  git branch -M main
  git push --quiet -u origin main
  ok "Tap aktualisiert: Formula/gobo.rb @ v${version} (Branch: main)"
}

# ---------------------------------------------------------------------------
# Formel als Vorlage erzeugen
# ---------------------------------------------------------------------------
generate_formula() {
  local version="$1"
  cat <<EOF
class Gobo < Formula
  desc "Simple shell text editor for one UTF-8 file"
  homepage "https://github.com/${GITHUB_USER}/${REPO_NAME}"
  url "https://github.com/${GITHUB_USER}/${REPO_NAME}/releases/download/v${version}/gobo-${version}-${ARCH}.tar.gz"
  sha256 "${SHA256}"
  version "${version}"
  license "GPL-3.0-or-later"

  depends_on arch: :arm64

  def install
    bin.install "gobo"
  end

  test do
    assert_match "A shell text editor", shell_output("#{bin}/gobo --help")
  end
end
EOF
}

# ---------------------------------------------------------------------------
# Hauptablauf
# ---------------------------------------------------------------------------
main() {
  echo "=== gobo release ==="
  command -v gh >/dev/null || die "gh CLI fehlt."
  command -v cargo >/dev/null || die "cargo fehlt."

  local version
  version=$(read_version)
  info "Version aus Cargo.toml: $version"

  check_version_changed "$version"
  check_clean_tree
  build_binary
  package_binary "$version"
  compute_sha
  create_tag "$version"
  create_release "$version"
  update_tap "$version"

  echo ""
  echo "=== Fertig ==="
  echo "  Installieren:  brew tap ${GITHUB_USER}/tap && brew install gobo"
  echo "  Update:        brew upgrade gobo"
}

main "$@"

#!/usr/bin/env bash
# The first command after cloning melies: installs the tools and the platform's C++ toolchain, stages
# the pinned FFmpeg and runs every check of CI's light tier; on request, the Kokoro files and the model
# tier too. Safe to run again: each step skips or quickly repeats what is already done. Bash only; on
# Windows run it from Git Bash. See the README, "Setup".
set -euo pipefail
cd "$(dirname "$0")"

usage() {
  cat <<'EOF'
Usage: ./new-folder-who-dis.sh [--skip-ci] [--with-models]

Sets up this clone of melies from nothing to a tested state, in order:
  1. scripts/install.sh: mise, rustup, the pinned Rust toolchain and tools,
     and the Git hooks
  2. the C++ toolchain Rust links with (Windows: the Visual Studio Build Tools'
     C++ workload and the Windows SDK, with winget; Debian/Ubuntu: the packages
     CI installs, with apt; macOS: the Xcode command-line tools)
  3. the pinned FFmpeg in tools/ (mise run stage -- --ffmpeg-only; about 114 MB
     on Windows, 41 MB on Linux), unless MELIES_TOOLS_DIR names another folder
     that holds it
  4. every check of CI's light tier (mise run ci): formatting, spelling, links,
     unused dependencies, licences, Clippy, cargo check and the tests that
     need FFmpeg only
  5. with --with-models: the pinned Kokoro model and voices in tools/speech
     (mise run stage; about 349 MB), unless MELIES_SPEECH_DIR names another
     folder that holds them
  6. with --with-models: the model tier (mise run test:model), which takes a
     long while on the CPU

Every step is safe to repeat: after a failure, fix what it reports and run
the script again. A first run takes a while (downloads, then compiling the
Rust workspace).

Options:
  --skip-ci      in step 4, only compile (mise run check); leave out step 6
  --with-models  add steps 5 and 6, which download about 349 MB; without it
                 nothing of the model is downloaded
  --help         show this help
EOF
}

skip_ci=false
with_models=false
for option in "$@"; do
  case "$option" in
    --skip-ci) skip_ci=true ;;
    --with-models) with_models=true ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $option" >&2
      echo "Run ./new-folder-who-dis.sh --help for the options." >&2
      exit 2
      ;;
  esac
done

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) platform=windows ;;
  Darwin) platform=macos ;;
  Linux) platform=linux ;;
  *)
    echo "Unsupported system: $(uname -s). melies is set up on Windows (Git Bash), macOS and Linux." >&2
    exit 1
    ;;
esac

started=$SECONDS
current_step=""
current_hint=""
summary=()

elapsed() {
  local seconds=$1
  printf '%dm %02ds' $((seconds / 60)) $((seconds % 60))
}

on_exit() {
  local status=$?
  if [ "$status" -ne 0 ] && [ -n "$current_step" ]; then
    echo >&2
    echo "Setup stopped at step \"$current_step\" (exit status $status), after $(elapsed $((SECONDS - started)))." >&2
    echo "Fix what the step reports above, then run ./new-folder-who-dis.sh again:" >&2
    echo "the steps already done are safe to repeat and finish quickly." >&2
    if [ -n "$current_hint" ]; then
      echo "$current_hint" >&2
    fi
  fi
}
trap on_exit EXIT

# step <title> <function> [<hint>]: runs one step under a heading and records how long it took; the
# hint is printed if the step fails.
step() {
  local title=$1 run=$2 step_started=$SECONDS
  current_step=$title
  current_hint=${3:-}
  echo
  echo "==> $title"
  "$run"
  local line
  line="$(printf '%-34s %s' "$title" "$(elapsed $((SECONDS - step_started)))")"
  summary+=("$line")
  echo "--> $title: done in $(elapsed $((SECONDS - step_started)))"
  current_step=""
}

skipped() {
  summary+=("$(printf '%-34s %s' "$1" "skipped ($2)")")
  echo
  echo "==> $1: skipped ($2)"
}

# winget and reg take Windows-style arguments: keep Git Bash from rewriting them as paths.
windows_command() {
  MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' "$@"
}

winget_install() {
  windows_command winget install --id "$@" --exact --accept-source-agreements --accept-package-agreements
}

# Tools installed in step 1 are not on this shell's PATH yet: add the places the install scripts put
# them, and on Windows the user and machine PATH as a new terminal would read them.
refresh_path() {
  local directory
  for directory in "$HOME/.local/bin" "$HOME/.cargo/bin"; do
    [ -d "$directory" ] && PATH="$directory:$PATH"
  done
  if [ "$platform" = windows ]; then
    local local_app_data user_profile registry_path
    local_app_data="$(cygpath -u "${LOCALAPPDATA:-$USERPROFILE/AppData/Local}")"
    user_profile="$(cygpath -u "$USERPROFILE")"
    [ -d "$user_profile/.cargo/bin" ] && PATH="$user_profile/.cargo/bin:$PATH"
    [ -d "$local_app_data/Microsoft/WinGet/Links" ] && PATH="$local_app_data/Microsoft/WinGet/Links:$PATH"
    for directory in "$local_app_data"/Microsoft/WinGet/Packages/jdx.mise_*/mise/bin; do
      [ -d "$directory" ] && PATH="$directory:$PATH"
    done
    registry_path="$(powershell.exe -NoProfile -Command \
      "[Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')" \
      | tr -d '\r')" || registry_path=""
    if [ -n "$registry_path" ]; then
      PATH="$PATH:$(cygpath -u -p "$registry_path")"
    fi
  fi
  export PATH
}

install_tools() {
  bash scripts/install.sh
  refresh_path
  local tool
  for tool in mise rustup git; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      echo "$tool was installed, but this shell cannot find it: open a new terminal and run this script again." >&2
      return 1
    fi
  done
}

# Windows: MSVC (vswhere) and the Windows 10/11 SDK: Rust's MSVC target links with them.
windows_prerequisites() {
  local program_files_x86 vswhere has_msvc=false has_sdk=false
  # CSIDL 42: Program Files (x86).
  program_files_x86="$(cygpath -u -F 42 2>/dev/null)" || program_files_x86=""
  [ -d "$program_files_x86" ] || program_files_x86="/c/Program Files (x86)"
  vswhere="$program_files_x86/Microsoft Visual Studio/Installer/vswhere.exe"

  if [ -x "$vswhere" ] && [ -n "$("$vswhere" -products '*' -latest \
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | tr -d '\r')" ]; then
    has_msvc=true
  fi
  local kits_root
  kits_root="$(windows_command reg query 'HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots' /v KitsRoot10 2>/dev/null \
    | tr -d '\r' | sed -n 's/^ *KitsRoot10 *REG_SZ *//p')" || kits_root=""
  [ -n "$kits_root" ] || kits_root="$program_files_x86/Windows Kits/10"
  local library
  for library in "$(cygpath -u "$kits_root")"/Lib/*/um/x64/kernel32.lib; do
    [ -f "$library" ] && has_sdk=true
  done
  echo "Microsoft C++ Build Tools: $has_msvc; Windows SDK: $has_sdk"

  if ! command -v winget >/dev/null 2>&1 && { ! $has_msvc || ! $has_sdk; }; then
    echo "winget is missing: install what is missing above by hand, then run this again." >&2
    return 1
  fi
  if ! $has_msvc; then
    # The C++ workload with its recommended parts brings the Windows SDK too. Visual Studio's installer
    # asks for administrator rights.
    echo "Installing the Visual Studio Build Tools with the C++ workload (a few GB; it asks for administrator rights)..."
    winget_install Microsoft.VisualStudio.BuildTools \
      --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
  elif ! $has_sdk; then
    echo "Installing the Windows SDK..."
    winget_install Microsoft.WindowsSDK.10.0.26100
  fi
}

# The packages the Checks workflow installs on Ubuntu (.github/workflows/checks.yml).
linux_packages=(build-essential pkg-config libssl-dev)

linux_prerequisites() {
  if ! command -v apt-get >/dev/null 2>&1 || ! command -v dpkg-query >/dev/null 2>&1; then
    echo "This is not Debian or Ubuntu: install your distribution's equivalents of: ${linux_packages[*]}" >&2
    echo "then run this script again." >&2
    return 1
  fi
  local missing=() package
  for package in "${linux_packages[@]}"; do
    if ! dpkg-query --show --showformat='${db:Status-Status}' "$package" 2>/dev/null | grep -q '^installed$'; then
      missing+=("$package")
    fi
  done
  if [ ${#missing[@]} -eq 0 ]; then
    echo "Every package is installed."
    return 0
  fi
  echo "Installing with apt (asks for your password): ${missing[*]}"
  sudo apt-get update
  sudo apt-get install --yes "${missing[@]}"
}

macos_prerequisites() {
  if xcode-select -p >/dev/null 2>&1; then
    echo "The Xcode command-line tools are installed: $(xcode-select -p)"
    return 0
  fi
  xcode-select --install || true
  echo "Finish installing the Xcode command-line tools in the window that opened, then run this script again." >&2
  return 1
}

cpp_toolchain() {
  case "$platform" in
    windows) windows_prerequisites ;;
    linux) linux_prerequisites ;;
    macos) macos_prerequisites ;;
  esac
}

# The folder a variable from mise.toml or the gitignored mise.local.toml names, as this shell reads it.
mise_folder() {
  local value
  value="$(mise exec -- printenv "$1" | tr -d '\r')" || return 1
  if [ "$platform" = windows ]; then
    cygpath -u "$value"
  else
    printf '%s\n' "$value"
  fi
}

# is_stage_folder <folder> <path>: is it <path> in this clone, where `mise run stage` stages? It does
# not have to exist yet. Anywhere else is a copy the owner of this clone chose, which stage never writes.
is_stage_folder() {
  local root="${1%/"$2"}"
  [ "$root" != "$1" ] && [ "$root" -ef . ]
}

has_tool() {
  [ -x "$1/$2" ] || [ -x "$1/$2.exe" ]
}

stage_ffmpeg() {
  local folder
  folder="$(mise_folder MELIES_TOOLS_DIR)"
  if is_stage_folder "$folder" tools; then
    mise run stage -- --ffmpeg-only
  elif has_tool "$folder" ffmpeg && has_tool "$folder" ffprobe; then
    echo "MELIES_TOOLS_DIR names $folder, which holds FFmpeg: nothing to download."
  else
    echo "MELIES_TOOLS_DIR names $folder, which holds no ffmpeg and ffprobe." >&2
    return 1
  fi
}

stage_kokoro() {
  local folder
  folder="$(mise_folder MELIES_SPEECH_DIR)"
  if is_stage_folder "$folder" tools/speech; then
    mise run stage
  elif [ -f "$folder/models/model.onnx" ]; then
    echo "MELIES_SPEECH_DIR names $folder, which holds the Kokoro model: nothing to download."
  else
    echo "MELIES_SPEECH_DIR names $folder, which holds no models/model.onnx." >&2
    return 1
  fi
}

compile_check() {
  mise run check
}

ci_checks() {
  mise run ci
}

model_tier() {
  mise run test:model
}

step "1. Tools (scripts/install.sh)" install_tools
step "2. C++ toolchain" cpp_toolchain
step "3. FFmpeg" stage_ffmpeg \
  "kuleshov pins FFmpeg builds for Windows and Linux. A copy from elsewhere goes in the gitignored
mise.local.toml, as MELIES_TOOLS_DIR (README, \"Setup\")."
if $skip_ci; then
  step "4. Compile only (mise run check)" compile_check
else
  step "4. CI checks (mise run ci)" ci_checks
fi
if $with_models; then
  step "5. Kokoro files (about 349 MB)" stage_kokoro \
    "A copy from elsewhere goes in the gitignored mise.local.toml, as MELIES_SPEECH_DIR (README, \"Setup\")."
else
  skipped "5. Kokoro files" "no --with-models"
fi
if ! $with_models; then
  skipped "6. Model tier" "no --with-models"
elif $skip_ci; then
  skipped "6. Model tier" "--skip-ci"
else
  step "6. Model tier (mise run test:model)" model_tier
fi

echo
echo "Summary"
for line in "${summary[@]}"; do
  echo "  $line"
done
echo "  Total: $(elapsed $((SECONDS - started)))"
echo
if $skip_ci; then
  echo "Done. The workspace compiles; nothing was tested. Run the checks with: mise run ci"
elif $with_models; then
  echo "Done. CI's light tier and the model tier pass. Coverage over both: mise run coverage"
else
  echo "Done. CI's light tier passes. For the model tier (about 349 MB): ./new-folder-who-dis.sh --with-models"
fi
echo "To use the pinned tools in every shell, add mise to it: https://mise.jdx.dev/getting-started.html"

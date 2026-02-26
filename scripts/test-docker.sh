#!/usr/bin/env bash
# Run snapshot tests inside a clean Docker container.
#
# Usage:
#   scripts/test-docker.sh            # run snapshot tests (default)
#   scripts/test-docker.sh --all      # run the full test suite
#   scripts/test-docker.sh --rebuild  # force a fresh image build first
#
# The container uses a read-only mount of the local source tree so that you
# can iterate quickly without rebuilding the image.  Snapshots are stored in
# the repo (tests/snapshots/) and compared inside the container, which means
# a mismatch will fail the run rather than silently writing new .snap.new
# files that you cannot see.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE="worktree-io-test"
DOCKERFILE="$PROJECT_ROOT/docker/Dockerfile.test"

# ── argument parsing ──────────────────────────────────────────────────────────
RUN_ALL=false
REBUILD=false
for arg in "$@"; do
    case "$arg" in
        --all)     RUN_ALL=true ;;
        --rebuild) REBUILD=true ;;
        *)
            echo "Unknown option: $arg" >&2
            echo "Usage: $0 [--all] [--rebuild]" >&2
            exit 1
            ;;
    esac
done

# ── build image ───────────────────────────────────────────────────────────────
BUILD_ARGS=()
if $REBUILD; then
    BUILD_ARGS+=(--no-cache)
fi
echo "Building $IMAGE…"
docker build "${BUILD_ARGS[@]}" -f "$DOCKERFILE" -t "$IMAGE" "$PROJECT_ROOT"

# ── choose test command ───────────────────────────────────────────────────────
if $RUN_ALL; then
    TEST_CMD=("cargo" "test" "--locked")
else
    TEST_CMD=("cargo" "test" "--locked" "--test" "snapshot_tests")
fi

# ── run tests ─────────────────────────────────────────────────────────────────
echo "Running: ${TEST_CMD[*]}"
docker run --rm "$IMAGE" "${TEST_CMD[@]}"

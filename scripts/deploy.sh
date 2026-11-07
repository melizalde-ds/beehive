#!/usr/bin/env bash
# Build Beehive and ship it to the Lightsail box. Safe to rerun.
# Usage: scripts/deploy.sh <host>     e.g. scripts/deploy.sh 44-1-2-3.sslip.io
set -euo pipefail
HOST=${1:?usage: scripts/deploy.sh <host>}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
CACHE=${XDG_CACHE_HOME:-$HOME/.cache}/beehive-deploy
SSH=(ssh -o StrictHostKeyChecking=accept-new "ubuntu@$HOST")

echo "1/4 Build the server (Docker: this machine's glibc is newer than the box's)"
mkdir -p "$CACHE/cargo" "$CACHE/target"
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$ROOT:/src:ro" -v "$CACHE/cargo:/cargo" -v "$CACHE/target:/target" \
  -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target -w /src/server \
  rust:1-bookworm cargo build --release --locked

echo "2/4 Build the dashboard"
(cd "$ROOT/web" && pnpm build)

echo "3/4 Server settings (first deploy only; secrets come from .env, never from the repo)"
if ! "${SSH[@]}" test -f /etc/beehive.env; then
  set -a; . "$ROOT/.env"; set +a
  printf '%s\n' \
    "REDO_MODE=real" \
    "REDO_BASE=${REDO_BASE:-https://api.getredo.com/v2.2}" \
    "REDO_STORE_ID=${REDO_STORE_ID:?missing in .env}" \
    "REDO_API_SECRET=${REDO_API_SECRET:?missing in .env}" \
    "BEEHIVE_JWT_SECRET=$(openssl rand -hex 32)" \
    "BEEHIVE_WEB_URL=https://$HOST" \
    "BEEHIVE_DEMO_SEED=1" \
    "PORT=3000" \
    | "${SSH[@]}" "sudo sh -c 'umask 077; cat > /etc/beehive.env'"
fi

echo "4/4 Upload and restart"
STAGE="$CACHE/stage"
rm -rf "$STAGE" && mkdir -p "$STAGE"
cp "$CACHE/target/release/beehive" "$ROOT/deploy/beehive.service" "$ROOT/deploy/setup.sh" "$STAGE/"
sed "s/BEEHIVE_HOST/$HOST/" "$ROOT/deploy/Caddyfile" > "$STAGE/Caddyfile"
cp -r "$ROOT/web/dist" "$STAGE/web"
tar -C "$STAGE" -cz . | "${SSH[@]}" \
  "rm -rf /tmp/beehive-up && mkdir /tmp/beehive-up && tar -xz -C /tmp/beehive-up && sudo bash /tmp/beehive-up/setup.sh"

echo "Live: https://$HOST  (agent: BEEHIVE_URL=https://$HOST/api node agent/agent.ts --reset)"

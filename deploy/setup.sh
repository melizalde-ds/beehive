#!/usr/bin/env bash
# Runs on the box as root, from the folder scripts/deploy.sh uploaded. Safe to rerun.
set -euo pipefail
cd "$(dirname "$0")"

command -v caddy >/dev/null || { apt-get update -q; apt-get install -y -q caddy; }
id beehive &>/dev/null || useradd --system --no-create-home --shell /usr/sbin/nologin beehive

mkdir -p /srv/beehive
install -m 755 beehive /srv/beehive/beehive
rm -rf /srv/beehive/web && cp -r web /srv/beehive/web
install -m 644 beehive.service /etc/systemd/system/beehive.service
install -m 644 Caddyfile /etc/caddy/Caddyfile

systemctl daemon-reload
systemctl enable beehive
systemctl restart beehive
systemctl reload caddy || systemctl restart caddy

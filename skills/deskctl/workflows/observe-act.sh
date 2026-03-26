#!/usr/bin/env bash
# observe-act.sh - main desktop interaction loop
# usage: ./observe-act.sh <selector> [action] [action-args...]
# example: ./observe-act.sh 'title=Firefox' click
# example: ./observe-act.sh 'class=terminal' type "ls -la"
set -euo pipefail

SELECTOR="${1:?usage: observe-act.sh <selector> [action] [action-args...]}"
ACTION="${2:-click}"
shift 2 2>/dev/null || true

# 1. observe - snapshot the desktop, get current state
echo "--- observe ---"
deskctl snapshot --annotate --json | head -1
deskctl get active-window

# 2. wait - ensure target exists
echo "--- wait ---"
deskctl wait window --selector "$SELECTOR" --timeout 10

# 3. act - perform the action on the target
echo "--- act ---"
case "$ACTION" in
  click)    deskctl click "$SELECTOR" ;;
  dblclick) deskctl dblclick "$SELECTOR" ;;
  focus)    deskctl focus "$SELECTOR" ;;
  type)     deskctl focus "$SELECTOR" && deskctl type "$@" ;;
  press)    deskctl focus "$SELECTOR" && deskctl press "$@" ;;
  hotkey)   deskctl focus "$SELECTOR" && deskctl hotkey "$@" ;;
  close)    deskctl close "$SELECTOR" ;;
  *)        echo "unknown action: $ACTION"; exit 1 ;;
esac

# 4. verify - snapshot again to confirm result
echo "--- verify ---"
sleep 0.5
deskctl snapshot --json | head -1

#!/usr/bin/env bash
set -euo pipefail

deskctl doctor
deskctl snapshot --annotate
deskctl get active-window
deskctl wait window --selector "${1:-focused}" --timeout "${2:-5}"

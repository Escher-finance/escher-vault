#!/usr/bin/env bash
set -e
nix develop .# --command "${@:-bash}"

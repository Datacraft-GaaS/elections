#!/usr/bin/env bash
# Initialize SoftHSMv2 slots matching the eel production layout.
# Run once on a developer machine before `just -f environments/dev/kind/justfile apply-all`.
#
# Requires: softhsm2 installed (via Nix: nix-shell -p softhsm2, or brew install softhsm on macOS).
# PKCS#11 module: see README.md for platform-specific paths.

set -euo pipefail

SOFTHSM_CONF="${SOFTHSM2_CONF:-${HOME}/.softhsm2/softhsm2.conf}"
TOKEN_DIR="${SOFTHSM_TOKEN_DIR:-${HOME}/.softhsm2/tokens}"

# eel slot layout
declare -A SLOTS=(
  [eel-sth]="eel-sth-so-pin:eel-sth-user-pin"
  [eel-declaration]="eel-decl-so-pin:eel-decl-user-pin"
  [eel-kit-ca]="eel-kitca-so-pin:eel-kitca-user-pin"
  [eel-credential-ca]="eel-credca-so-pin:eel-credca-user-pin"
  [eel-code-signing]="eel-codesign-so-pin:eel-codesign-user-pin"
  [eel-cycle-kek]="eel-kek-so-pin:eel-kek-user-pin"
)

echo "==> Ensuring SoftHSM config dir exists"
mkdir -p "$(dirname "$SOFTHSM_CONF")" "$TOKEN_DIR"

if [[ ! -f "$SOFTHSM_CONF" ]]; then
  cat > "$SOFTHSM_CONF" <<EOF
directories.tokendir = ${TOKEN_DIR}
objectstore.backend = file
log.level = INFO
slots.removable = false
EOF
  echo "    Created $SOFTHSM_CONF"
fi

export SOFTHSM2_CONF="$SOFTHSM_CONF"

echo "==> Initializing slots"
for label in "${!SLOTS[@]}"; do
  IFS=':' read -r so_pin user_pin <<< "${SLOTS[$label]}"
  if softhsm2-util --list-tokens 2>/dev/null | grep -q "Label: *${label}"; then
    echo "    Slot '$label' already exists, skipping."
  else
    softhsm2-util --init-token --free \
      --label "$label" \
      --so-pin "$so_pin" \
      --pin "$user_pin"
    echo "    Initialized slot: $label"
  fi
done

echo ""
echo "==> Slot listing:"
softhsm2-util --list-tokens

echo ""
echo "Done. Set SOFTHSM2_CONF=$SOFTHSM_CONF in your shell or .env files."
echo "Run 'cargo run --release -p hsm-init -- init-slots' to generate keys in each slot."

#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# bootstrap.sh: Import EEL Keycloak realms and configure service accounts.
# Requires: kcadm.sh on PATH, curl, jq, OPENBAO_ADDR and OPENBAO_TOKEN env vars.

set -euo pipefail

# -----------------------------------------------------------------------
# Constants
# -----------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REALMS_DIR="${SCRIPT_DIR}/../realms"
KC_BASE_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KC_REALM="master"
OPENBAO_MOUNT="${OPENBAO_MOUNT:-secret}"
OPENBAO_PATH_PREFIX="${OPENBAO_PATH_PREFIX:-eel/keycloak}"

# 18 backend services
SERVICES=(
	biometric-edge-svc
	bvr-svc
	abis-svc
	vprs-svc
	bvi-svc
	rts-svc
	portal-svc
	crms-svc
	transparency-svc
	ecc-svc
	boundary-svc
	queue-svc
	inventory-svc
	governance-svc
	ai-platform-svc
	field-mobile-svc
	training-svc
	support-admin-svc
)

REALMS=(
	eel-staff
	eel-party
	eel-observer
	eel-media
	eel-public
	eel-county
)

REALM_FILES=(
	"${REALMS_DIR}/staff.json"
	"${REALMS_DIR}/party.json"
	"${REALMS_DIR}/observer.json"
	"${REALMS_DIR}/media.json"
	"${REALMS_DIR}/public.json"
	"${REALMS_DIR}/county.json"
)

# -----------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------
log() {
	echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"
}

# Fetch a secret value from OpenBao via curl.
# Usage: bao_get <path> <key>
bao_get() {
	local BAO_PATH="$1"
	local BAO_KEY="$2"
	curl -sf \
		-H "X-Vault-Token: ${OPENBAO_TOKEN}" \
		"${OPENBAO_ADDR}/v1/${OPENBAO_MOUNT}/data/${OPENBAO_PATH_PREFIX}/${BAO_PATH}" \
		| jq -r ".data.data.${BAO_KEY}"
}

# -----------------------------------------------------------------------
# Step 1: Authenticate kcadm to Keycloak master realm.
# Admin credentials are stored in OpenBao at eel/keycloak/admin.
# -----------------------------------------------------------------------
log "Step 1: Authenticating to Keycloak at ${KC_BASE_URL}"

KC_ADMIN_USER="$(bao_get admin username)"
KC_ADMIN_PASS="$(bao_get admin password)"

kcadm.sh config credentials \
	--server "${KC_BASE_URL}" \
	--realm "${KC_REALM}" \
	--user "${KC_ADMIN_USER}" \
	--password "${KC_ADMIN_PASS}"

log "  authenticated as ${KC_ADMIN_USER}"

# -----------------------------------------------------------------------
# Step 2: Import realm JSON files.
# Each import is idempotent: if the realm exists, kcadm will update it.
# -----------------------------------------------------------------------
log "Step 2: Importing realm JSON files"

for REALM_FILE in "${REALM_FILES[@]}"; do
	REALM_NAME="$(jq -r '.realm' "${REALM_FILE}")"
	log "  importing realm: ${REALM_NAME} from ${REALM_FILE}"
	kcadm.sh create realms \
		-f "${REALM_FILE}" \
		--merge 2>/dev/null \
		|| kcadm.sh update "realms/${REALM_NAME}" -f "${REALM_FILE}"
	log "  done: ${REALM_NAME}"
done

# -----------------------------------------------------------------------
# Step 3: Apply WebAuthn passwordless policy to eel-staff realm.
# Policy JSON is at config/webauthn-policy.json.
# -----------------------------------------------------------------------
log "Step 3: Applying WebAuthn policy to eel-staff"

WEBAUTHN_POLICY_FILE="${SCRIPT_DIR}/webauthn-policy.json"
kcadm.sh update "realms/eel-staff" \
	-f "${WEBAUTHN_POLICY_FILE}"

log "  WebAuthn policy applied"

# -----------------------------------------------------------------------
# Step 4: Create service account users for the 18 backend services in eel-staff.
# Passwords are fetched per-service from OpenBao at eel/keycloak/svc/<service-name>.
# -----------------------------------------------------------------------
log "Step 4: Creating service account users in eel-staff"

for SVC in "${SERVICES[@]}"; do
	SVC_USER="sa-${SVC}"
	SVC_PASS="$(bao_get "svc/${SVC}" password)"

	# Create or update the user
	EXISTING_ID="$(kcadm.sh get "users" \
		-r eel-staff \
		-q "username=${SVC_USER}" \
		| jq -r '.[0].id // empty')"

	if [[ -z "${EXISTING_ID}" ]]; then
		log "  creating service account: ${SVC_USER}"
		kcadm.sh create users \
			-r eel-staff \
			-s "username=${SVC_USER}" \
			-s "enabled=true" \
			-s "serviceAccountClientId=${SVC}" \
			-s "realmRoles=[\"SERVICE_ACCOUNT\"]"
	else
		log "  service account already exists: ${SVC_USER} (id=${EXISTING_ID})"
	fi

	# Reset password
	kcadm.sh set-password \
		-r eel-staff \
		--username "${SVC_USER}" \
		--new-password "${SVC_PASS}" \
		--temporary false

	log "  password set for ${SVC_USER}"
done

# -----------------------------------------------------------------------
# Step 5: Wire event listeners to Kafka for all realms.
# The eel-audit-kafka listener SPI must be deployed before running this step.
# -----------------------------------------------------------------------
log "Step 5: Wiring event listeners to eel-audit-kafka"

for REALM_NAME in "${REALMS[@]}"; do
	log "  setting event listeners for realm: ${REALM_NAME}"
	kcadm.sh update "realms/${REALM_NAME}" \
		-r master \
		-s 'eventsListeners=["jboss-logging","eel-audit-kafka"]' \
		-s 'eventsEnabled=true' \
		-s 'adminEventsEnabled=true' \
		-s 'adminEventsDetailsEnabled=true'
	log "  done: ${REALM_NAME}"
done

# -----------------------------------------------------------------------
# Step 6: Apply WebAuthn required policy via REST for high-privilege clients.
# chair-portal and iab-portal have authenticationFlowBindingOverrides set
# to browser-webauthn-flow in the realm JSON, so no further REST call is needed.
# This step verifies the flow exists by querying it.
# -----------------------------------------------------------------------
log "Step 6: Verifying browser-webauthn-flow exists in eel-staff"

FLOW_COUNT="$(kcadm.sh get authentication/flows \
	-r eel-staff \
	| jq '[.[] | select(.alias=="browser-webauthn-flow")] | length')"

if [[ "${FLOW_COUNT}" -eq 0 ]]; then
	log "  WARNING: browser-webauthn-flow not found in eel-staff"
	log "  The realm JSON import should have created it. Check import output."
	exit 1
fi

log "  browser-webauthn-flow confirmed present"

# -----------------------------------------------------------------------
# Done
# -----------------------------------------------------------------------
log "Bootstrap complete."
log "Realms imported: ${REALMS[*]}"
log "Service accounts created: ${#SERVICES[@]} services"
log "Audit Kafka topic: eel.audit.keycloak.v1"

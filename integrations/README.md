# integrations/

External-system adapters (50+ upstreams) consumed by eel services. Python adapters wrapped by Rust facades. See ADRs 0004, 0006, 0009, 0010, 0019, 0022, 0023.

## Layout

Each adapter follows a tripartite structure:

- `adapter/` - Python package (pyproject.toml + src/) wrapping vendor SDK or REST client. Implements sync/async service classes, Pydantic models for request/response, auth handling, PII field masking.
- `double/` - Canned-response mock for CI and dev. Runs as standalone Python service (localhost:8081 default). Exposes chaos knobs (latency, error injection, duplicate frames).
- `contract-tests/` - Consumer/provider Pact contracts or pytest fixtures. Verifies schema, auth headers, error codes, rate-limit behavior.

## Adapters by tier

### Tier A (WP-2.5: Wave 2 trust core)

| Adapter | Upstream Party | Protocol | Auth | Data Class | Breaker Policy | Contract Test Path |
|---------|---|---|---|---|---|---|
| iprs | IPRS (Integrated Population Registration System) | REST/SOAP | SAML+mTLS | PII-max | open/half 5s/2req | integrations/iprs/contract-tests |
| mrz | ICAO 9303 MRZ reader SDK | SDK (local) | device-cert | PII-high | n/a (local) | integrations/mrz/contract-tests |
| telco-sms | Safaricom/Airtel/Telkom SMS GW | REST | OAuth2 | PII-low | open 30s/5req | integrations/telco-sms/contract-tests |
| telco-ussd | Safaricom/AT USSD GW | TCP/USSD-GW | session-key | PII-low | open 10s/3req | integrations/telco-ussd/contract-tests |
| telco-vpn | Safaricom Leased Line / WireGuard | IPsec+WireGuard | PSK+cert | none | always-on | integrations/telco-vpn/contract-tests |
| legacy-abis | Incumbent AFIS vendor SOAP API | SOAP | mTLS+WS-Sec | PII-max | open 60s/1req | integrations/legacy-abis/contract-tests |
| legacy-android-shim | Legacy field Android APK gRPC shim | gRPC | device-cert | PII-high | open 10s/5req | integrations/legacy-android-shim/contract-tests |
| hsm-pkcs11 | Thales Luna 7 / Utimaco (SoftHSMv2 dev) | PKCS11 | slot+PIN | crypto-key | n/a (library) | integrations/hsm-pkcs11/contract-tests |
| nist-beacon | NIST Randomness Beacon v2 | REST (public) | none | public | n/a | integrations/nist-beacon/contract-tests |

### Tier B (WP-4.12: Wave 4 operations)

| Adapter | Upstream Party | Protocol | Auth | Data Class | Breaker Policy | Contract Test Path |
|---------|---|---|---|---|---|---|
| mpesa-daraja | Safaricom M-Pesa Daraja API | REST | OAuth2 | PII-medium | open 30s/5req | integrations/mpesa-daraja/contract-tests |
| kra-itax | KRA iTax SOAP gateway | SOAP | mTLS+PIN | PII-high | open 60s/2req | integrations/kra-itax/contract-tests |
| ifmis | Integrated Financial Management IS | REST | OAuth2 | PII-medium | open 30s/3req | integrations/ifmis/contract-tests |
| nssf | National Social Security Fund API | REST | mTLS | PII-high | open 60s/2req | integrations/nssf/contract-tests |
| nhif-shif | NHIF/SHA health insurance API | REST | mTLS | PII-high | open 60s/2req | integrations/nhif-shif/contract-tests |
| whatsapp | WhatsApp Business Cloud API | REST+Webhooks | Bearer | PII-medium | open 30s/5req | integrations/whatsapp/contract-tests |
| gov-printer | Government Printer PDF/print API | REST+FTP | API-key | PII-low | open 120s/1req | integrations/gov-printer/contract-tests |
| kbc-sftp | KBC SFTP file drop | SFTP | SSH-key | PII-low | open 300s/1req | integrations/kbc-sftp/contract-tests |
| knads | Kenya National Archives | REST | API-key | PII-medium | open 60s/3req | integrations/knads/contract-tests |
| datacite | DataCite DOI registration API | REST | BasicAuth | public/low | open 30s/3req | integrations/datacite/contract-tests |
| ecourt | eCourt e-filing REST API | REST/SOAP | mTLS | PII-medium | open 60s/2req | integrations/ecourt/contract-tests |
| ppdt | Political Parties Disputes Tribunal | REST | OAuth2 | PII-medium | open 60s/2req | integrations/ppdt/contract-tests |
| mck | Media Council of Kenya | REST | API-key | PII-low | open 60s/3req | integrations/mck/contract-tests |
| ncic-dci-odpp | NCIC/DCI/ODPP clearance API | REST | mTLS | PII-max | open 120s/1req | integrations/ncic-dci-odpp/contract-tests |
| cbk-frc | CBK/FRC financial intelligence | REST | mTLS | PII-high | open 60s/2req | integrations/cbk-frc/contract-tests |
| kmd | Kenya Meteorological Dept API | REST | API-key | public | open 30s/5req | integrations/kmd/contract-tests |
| airline-cargo | Kenya Airways/IATA cargo manifest | REST+EDI | OAuth2 | PII-medium | open 60s/2req | integrations/airline-cargo/contract-tests |
| kra-eclearance | KRA e-Clearance (diaspora ballot import) | REST | mTLS+PIN | PII-high | open 60s/2req | integrations/kra-eclearance/contract-tests |
| pantone | Pantone colour-match API | REST | API-key | none | open 30s/5req | integrations/pantone/contract-tests |
| social-platforms | Twitter/X, Facebook, YouTube, TikTok APIs | REST | OAuth2 | PII-medium | open 30s/5req | integrations/social-platforms/contract-tests |
| litellm | LiteLLM proxy (62.169.25.77:4000) | REST (OpenAI-compat) | API-key | PII-low | open 30s/3req | integrations/litellm/contract-tests |
| nllb | NLLB-200 translation service | REST | Bearer | PII-medium | open 30s/5req | integrations/nllb/contract-tests |
| power-bi | Microsoft Power BI Embedded | REST | OAuth2 (AAD) | PII-low | open 30s/5req | integrations/power-bi/contract-tests |
| odpc-breach | ODPC breach notification API | REST | mTLS+signed-payload | PII-high | open 60s/2req | integrations/odpc-breach/contract-tests |
| kebs-warranty | KEBS/OEM warranty registration | REST | API-key | PII-low | open 30s/5req | integrations/kebs-warranty/contract-tests |
| ipfs | IPFS/Kubo node (libp2p) | HTTP+libp2p | peer-key/mTLS | public | open 30s/3req | integrations/ipfs/contract-tests |
| satellite | VSAT/LEO satellite IP link | UDP/REST | PSK | none | always-on/fallback | integrations/satellite/contract-tests |
| crs | Civil Registration System (NRB/RB) | REST | mTLS | PII-max | open 60s/2req | integrations/crs/contract-tests |
| nrb | National Registration Bureau | REST | mTLS | PII-max | open 60s/2req | integrations/nrb/contract-tests |
| eacc | Ethics & Anti-Corruption Commission | REST | mTLS | PII-high | open 60s/2req | integrations/eacc/contract-tests |
| kps-kdf-unhcr | Kenya Prison Service/KDF/UNHCR special voter | REST | mTLS | PII-high | open 60s/2req | integrations/kps-kdf-unhcr/contract-tests |
| printer-mes | Ballot printer MES (De La Rue/Enschede) | REST | API-key | PII-low | open 120s/1req | integrations/printer-mes/contract-tests |
| card-manufacturer | Voter card manufacturer (Thales/IDEMIA) | REST+EDI | mTLS | PII-high | open 120s/1req | integrations/card-manufacturer/contract-tests |

### Stubs (empty, placeholder MoU pending)

| Adapter | Status |
|---------|--------|
| cbk | Central Bank of Kenya (general) |
| frc | Financial Reporting Centre |
| ivr | IVR call-centre platform |
| kebs | Kenya Bureau of Standards (general) |
| kra | KRA general (non-iTax, non-eClearance) |
| odpc | ODPC general (non-breach) |
| telco-whatsapp | Telco-managed WhatsApp fallback |

### Reports subs (RPT-B, owned by RPT-C agent)

| Adapter | Note |
|---------|------|
| gov-printer/gazette-report-poster | automated gazette PDF push |
| kbc-sftp/report-sftp-uploader | automated KBC SFTP push |

## Related docs

- ADR-0004: Event bus architecture
- ADR-0006: PII enforcement (field-level encryption, redaction, audit logs)
- ADR-0009: Circuit breaker resilience patterns
- ADR-0010: Adapter contract testing (Pact, pytest-httpserver)
- ADR-0019: HSM key rotation and backup
- ADR-0022: Legacy system migration (ABIS, Android APK sunset)
- ADR-0023: Satellite / offline fallback strategy

Individual adapter READMEs: see integrations/[name]/README.md for each adapter.

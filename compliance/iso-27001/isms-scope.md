# ISMS Scope Statement

Document reference: IEBC-ISMS-SCOPE-001
Version: 1.0
Date: [YYYY-MM-DD]
Approved by: [DG_NAME], Director-General, IEBC

---

## 1. Organisation

The Independent Electoral and Boundaries Commission (IEBC) is a constitutional commission established under Article 88 of the Constitution of Kenya 2010. IEBC is responsible for the continuous registration of citizens as voters, the regulation of political parties, the settlement of electoral disputes, and the conduct of elections and referenda.

The eel (IEBC Integrated Election Management System) is the software platform designed, built, and operated to support all IEBC electoral functions. eel is developed under contract reference [CONTRACT_REF] pursuant to GCC 15.4.

---

## 2. Scope Boundaries

### 2.1 Included in scope

The following are within the ISMS scope:

- All eel software services:
  - transparency-svc, bvr-svc, abis-svc, bvi-svc, biometric-edge-svc, portal-svc, ecc-svc, queue-svc, inventory-svc, ai-platform-svc, field-mobile-svc, vprs-svc, crms-svc, governance-svc, training-svc, support-admin-svc
- All eel shared packages: packages/eel-*/
- All eel tools: tools/ directory
- The data centre infrastructure hosting eel: primary DC, mirror DC (hot standby), and 5 edge sites
- Developer workstations with MDM profiles from which eel code is developed, built, and deployed
- The CI/CD pipeline (Gitea, Nix flake, ArgoCD, image registry)
- The authentication and authorisation infrastructure: Keycloak (auth.lindela.io), SpiceDB, MinIO (auth.lindela.io)
- The observability infrastructure: Grafana, Prometheus, Loki (mail.lindela.io)
- All personal data processed by eel, as classified in ADR-0016 and the DPA toolkit
- All eel configuration, secrets management (OpenBao), and key management (HSM, LiteLLM proxy)

### 2.2 Excluded from scope

The following are outside the ISMS scope, though they are interfaced with eel:

- IPRS (Integrated Population Registration System) operated by the Department of Immigration
- NRB (National Registration Bureau) systems
- CRS (Civil Registration Service) systems
- eCourt system operated by the Judiciary
- IFMIS operated by the National Treasury
- KRA iTax system
- Telco networks and infrastructure
- Physical ballot papers, seals, and non-digital election materials
- IEBC headquarters physical security (governed by separate physical security controls)

Interfaces to excluded systems are governed by contracts (MoUs) with the respective agencies and are represented as adapter layers within the eel codebase (integrations/ directory).

---

## 3. Assets in Scope

### 3.1 Information assets

| Asset category | Examples | Highest data tier |
|---|---|---|
| Voter personal data | Biometric templates, National ID, biographical data | A (max) |
| Voter operational data | Polling station assignment, turnout status | B |
| Election results data | Form 34A/B/C, tallied results, Merkle roots | C (public, signed) |
| Audit and evidence | Transparency log, audit events | D |
| Operational data | Logs, metrics, traces | D |
| Configuration and secrets | Environment configs, TLS certificates, HSM key handles | D |
| Source code and SBOMs | eel git repository, signed SBOMs | D |

### 3.2 Hardware assets

- Kit devices (Android, legacy Android): approximately [KIT_COUNT] units
- HSM appliances: primary and mirror
- Server hardware at primary DC, mirror DC, and edge sites (Contabo EU fleet)
- Developer workstations: [WORKSTATION_COUNT] (MDM-managed)

### 3.3 Software assets

All eel software components as listed in compliance/ipr-handover/source-inventory.yaml.

---

## 4. Locations

| Location | Role | In scope |
|---|---|---|
| Primary DC: [DC_ADDRESS] | All production services, primary PostgreSQL, primary Kafka | Yes |
| Mirror DC: [MIRROR_DC_ADDRESS] | Hot standby, PG hot standby (144.91.112.190), Kafka mirror | Yes |
| Edge site 1-5: [EDGE_ADDRESSES] | Kafka edge relay, kit synchronisation, local result capture | Yes |
| auth.lindela.io (62.84.181.55) | Keycloak, SpiceDB, MinIO | Yes |
| ml/vault (62.169.25.77) | LiteLLM, OpenBao, Ollama, NLLB | Yes |
| mail/monitoring (217.76.53.144) | Stalwart mail, Grafana, Prometheus, Loki | Yes |
| search/tools (84.247.181.100) | SearXNG, Firecrawl, Docling, Soketi | Yes |
| Developer workstations | Code development, build, review | Yes (MDM) |
| IPRS, NRB, CRS systems | External government systems | No |

---

## 5. Interested Parties

| Party | Relationship | Information security interests |
|---|---|---|
| Kenya voters | Data subjects | Confidentiality and integrity of personal data; accuracy of registration |
| ODPC | Regulatory | DPA compliance, breach notification |
| Elections (Technology) Regulations certifier | Regulatory | Independent security certification |
| Political parties and candidates | Stakeholders | Integrity and fairness of election results |
| International and domestic observers | Stakeholders | Transparency and auditability |
| IEBC commissioners and staff | Internal | Operational continuity, security of the platform |
| Supplier/contractor | Contract party | Delivery of IPR per GCC 15.4, warranty obligations |
| NCIC (National Cohesion and Integration Commission) | Regulatory | Hate speech and incitement monitoring (M17) |
| ODPP, DCI, KDF, KPS | National security | Fraud and security incident response |
| Judiciary and OAG | Legal oversight | Evidence integrity, petitions and audit |

---

## 6. Interfaces and Dependencies

| Interface | Direction | Protocol | Sensitivity |
|---|---|---|---|
| IPRS biometric verification | Inbound to eel | VPN + mTLS | Tier A |
| NRB/CRS continuous register feed | Inbound to eel | VPN + mTLS | Tier A |
| Telco VPN for kit transmission | Outbound from kit | IPsec VPN + TLS | Tier B |
| M-Pesa B2C for payroll | Outbound from M55 | HTTPS + HMAC | Tier B |
| KRA iTax statutory returns | Outbound from M55 | HTTPS | Tier A |
| NSSF / NHIF/SHIF returns | Outbound from M55 | HTTPS | Tier A |
| eCourt evidence submission | Outbound from M40 | HTTPS + signed payload | Tier A |
| KBC/Gov Printer result submission | Outbound from M10 | SFTP + signature | Tier C |

---

## 7. Applicable Legislation and Regulations

| Instrument | Relevance |
|---|---|
| Constitution of Kenya 2010, Article 38 | Right to vote; basis for voter data processing |
| Elections Act, Cap 7 | Core electoral law; §4, §5, §26, §32, §44, §68, §74, §88, §109 |
| Elections (Technology) Regulations 2017 | Technology certification, security, audit trail requirements |
| Kenya Data Protection Act 2019 | PII processing, DPIA, breach notification, SAR/DSAR, automated decisions |
| Computer Misuse and Cybercrimes Act 2018 | Cybercrime offences, incident reporting |
| National Cohesion and Integration Act 2008 | Hate speech monitoring obligations |
| Employment Act, Cap 226 | Staff data, payroll, records retention |
| GCC 15.4 | Government ICT procurement; IPR handover requirements |
| ISO 27001:2022 | ISMS framework |
| ASVS 4.0 | Application security verification requirements (L3 for election-critical modules) |

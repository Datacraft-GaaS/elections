# Copyright and Moral Rights Assignment

Government Common Conditions (GCC) 15.4: Intellectual Property Rights
Document reference: IEBC-COPYRIGHT-ASSIGN-001
Version: 1.0
Date: [YYYY-MM-DD]

---

## 1. Parties

**Assignor (Supplier):** [SUPPLIER_NAME], a company incorporated under the laws of [JURISDICTION] with registration number [REG_NO], whose registered office is at [ADDRESS] ("the Supplier").

**Assignee (IEBC):** Independent Electoral and Boundaries Commission, a constitutional commission established under Article 88 of the Constitution of Kenya 2010, having its principal offices at Anniversary Towers, University Way, Nairobi, Kenya ("IEBC").

---

## 2. Background

The Supplier has developed the eel (IEBC Integrated Election Management System) Custom Software listed in Schedule 1 pursuant to contract reference [CONTRACT_REF] dated [CONTRACT_DATE] between the Supplier and IEBC. GCC 15.4 requires that the intellectual property rights in Custom Software vest in IEBC on creation.

---

## 3. Assignment of Economic Rights

With effect from the date of creation of each artefact listed in Schedule 1 (or, if earlier, the date of this deed), the Supplier assigns to IEBC:

3.1 All copyright and related rights in each Custom Software artefact listed in Schedule 1, including but not limited to the rights to:
   a. reproduce the Custom Software in any form;
   b. prepare derivative works based on the Custom Software;
   c. distribute copies of the Custom Software;
   d. perform and display the Custom Software;
   e. sub-licence the rights to third parties.

3.2 All rights assigned herein are assigned absolutely, worldwide, for the full term of copyright and any extensions thereof, with the right to bring proceedings for infringement in respect of past, present, and future acts.

3.3 The Supplier warrants that it is the sole author and owner of the Custom Software, free from any encumbrance, and has the full right and authority to make this assignment.

---

## 4. Moral Rights

4.1 To the extent permitted by law, the Supplier and each individual developer who contributed to the Custom Software (whether as an employee of the Supplier or as a contractor or subcontractor) irrevocably waives in favour of IEBC all moral rights in the Custom Software, including the right of attribution (paternity) and the right of integrity.

4.2 Where applicable local law does not permit waiver of moral rights, the Supplier agrees not to assert those rights against IEBC, its successors, or licensees.

4.3 IEBC acknowledges that this waiver does not affect the Supplier's right to be identified as the original developer of the Custom Software for historical, archival, or attribution purposes in non-commercial contexts.

---

## 5. Retained Rights

5.1 The Supplier retains no rights to use the Custom Software for any purpose other than:
   a. performance of its obligations under contract [CONTRACT_REF] during the contract period;
   b. maintenance and support obligations during any agreed warranty or support period;
   c. reference to the Custom Software in the Supplier's portfolio or marketing materials with IEBC's prior written consent.

5.2 Standard Software (as defined in `compliance/ipr-handover/software-classification.yaml`) is not covered by this assignment. Licence terms for Standard Software are as listed in that document.

---

## 6. Custom Software Artefacts (Schedule 1)

The following categories of artefacts are Custom Software for the purposes of this deed. Specific artefacts are listed in `compliance/ipr-handover/source-inventory.yaml`.

| Category | Repository path pattern |
|---|---|
| eel microservices | services/*/  |
| eel shared packages | packages/eel-*/  |
| eel tools | tools/asvs-check/, tools/wcag-audit/, tools/completeness/, tools/dr-failover/, tools/appset-gen/, tools/iam-seed/, tools/kit-simulator/ |
| eel configuration | platform/, environments/, config/ |
| eel Kubernetes manifests | services/*/deploy/ |
| eel Flutter apps | services/*/apps/*/ |
| eel OpenAPI contracts | packages/openapi-contracts/ |
| eel authz schemas | packages/authz-schema/ |
| eel event schemas | packages/event-schemas/ |
| eel documentation | docs/ |
| eel compliance documents | compliance/ |

---

## 7. Developer and Contractor Obligations

7.1 The Supplier shall ensure that all employees, contractors, and subcontractors who contribute to the Custom Software have assigned their rights in their contributions to the Supplier (or directly to IEBC) before commencement of work.

7.2 The Supplier shall maintain records of all contributors and their assignment agreements, and shall make those records available to IEBC on request.

7.3 The Supplier shall not engage any subcontractor to develop Custom Software without IEBC's prior written consent and without ensuring that the subcontractor is bound by equivalent IP assignment obligations.

---

## 8. Source Code Warranty

8.1 The Supplier warrants that:
   a. The Custom Software does not infringe the intellectual property rights of any third party.
   b. The Custom Software does not incorporate any code whose licence would impose obligations on IEBC beyond those disclosed in `compliance/ipr-handover/software-classification.yaml`.
   c. The Custom Software is free from any malicious code, backdoors, or undisclosed functionality.
   d. The source code delivered under this deed is complete and sufficient to build and deploy the Custom Software from source, using the instructions in `compliance/ipr-handover/gcc-15-4-handover-template.md` section 4.

8.2 The Supplier shall indemnify IEBC against any losses, costs, or damages arising from a breach of the warranties in clause 8.1.

---

## 9. Escrow Release Conditions

9.1 In the event of an escrow release as defined in `compliance/ipr-handover/gcc-15-4-handover-template.md` section 6.2, IEBC is authorised to use, modify, and maintain the Source Code for the purposes of operating the eel platform, without restriction and without further payment to the Supplier.

9.2 IEBC may engage a third party to provide maintenance and support services using the Source Code released from escrow.

---

## 10. Execution

This deed is executed as a deed by the parties on the date written above.

**Signed by [SUPPLIER_NAME]:**

| Field | Value |
|---|---|
| Authorised signatory name | [NAME] |
| Title | [TITLE] |
| Signature | [SIGNATURE] |
| Date | [YYYY-MM-DD] |
| Witness name | [WITNESS_NAME] |
| Witness signature | [WITNESS_SIGNATURE] |

**Signed for and on behalf of the Independent Electoral and Boundaries Commission:**

| Field | Value |
|---|---|
| Authorised signatory name | [NAME] |
| Title | [TITLE, e.g., Director-General] |
| Signature | [SIGNATURE] |
| Date | [YYYY-MM-DD] |
| Witness name | [WITNESS_NAME] |
| Witness signature | [WITNESS_SIGNATURE] |
| IEBC common seal | [AFFIXED] |

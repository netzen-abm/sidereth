# SIDERETH — Evidence-Grounded Administrative Correspondence Workflow

**Status:** Proposed product/workflow contract — not an implemented end-user feature  
**Scope:** Notice replies, document-evidence requests, RTI applications, and follow-up/appeal preparation  
**First tracked issue:** [#181](https://github.com/netzen-abm/sidereth/issues/181)  
**Decision principle:** Preserve the citizen's useful template structure; separate factual record, verified law, inference and requested relief; do not manufacture legal effects.

## 1. Purpose

This contract adapts the supplied **“Official Administrative Reply & Demand for Documentary Evidence Template”** into a reusable, source-grounded SIDERETH workflow.

The template is useful as a checklist for addressing an administrative notice, asking for the asserted authority and calculation, requesting relevant records, recording objections, preserving rights and delivery evidence, and planning a follow-up. It must not be copied verbatim as a universally authoritative legal instrument.

This is a **document-preparation and evidence-management workflow**, not a new rule engine, legal interpreter, automatic legal decision-maker, or autonomous dispatch service. It does not decide whether a levy is valid, whether the recipient owes money, whether a right has been waived, or which remedy will succeed.

The user remains the decision-maker. A high-impact legal assertion must be source-grounded, marked when uncertain, and reviewed by the user and, where appropriate, a qualified professional before use.

## 2. Separate document tracks

A single administrative issue may need more than one document. Do not combine routes whose recipients, legal purpose, scope and deadlines differ.

### Track A — Reply or representation to an issued notice

Purpose: respond to a specific notice, clarify the position, identify factual or legal disputes, request the basis for the demand and relevant records, and request an appropriate written response.

Core contents, selected according to the facts:
1. Authority, department and delivery/notice references.
2. Subject, fee/levy, amount and calculation as stated in the notice.
3. A concise statement of receipt and the matters being disputed or clarified.
4. The stated enabling Act, rule, notification, order, contract or tariff, distinguishing what the authority actually cited from what the sender has independently verified.
5. Specific requests for the relevant existing records, including the order/notification and applicable schedule, calculation or assessment record, competent-authority approval where relevant, exemption/appeal rules and reasons relied upon.
6. Fact-specific questions and objections.
7. A request for a written, reasoned response and, if appropriate under the applicable procedure, a request to defer coercive action or extend time pending clarification.
8. A neutral reservation of remedies actually available under applicable law.
9. Attachments, signature, contact details and dispatch method.

A request to respond within **14 days** may be included as a requested response period when useful, but it must be labelled as the sender's requested timeframe—not generated as a statutory deadline unless the applicable law or notice establishes that deadline.

### Track B — Right to Information (RTI) application

Purpose: seek information or existing records held by or under the control of the relevant public authority through the applicable RTI procedure.

This is a separate application addressed to the correct CPIO/SPIO or other designated channel. It should request identifiable existing records, inspection, notes/extracts, copies, certified copies or available electronic records as appropriate. It must apply the correct public authority, Central/State rules, fees, submission route, exemptions and appeal procedure.

Do not use the RTI form to demand that an officer create a new study, formulate a legal opinion, answer hypothetical questions, decide the constitutionality of a levy, or endorse the applicant's interpretation. Ask for the existing record containing that material, if one exists. The Department of Personnel and Training's official guide explains that RTI access concerns information held by or under the control of a public authority and does not generally require the PIO to create information or draw a conclusion for the applicant.

For Central/State and authority-specific procedures, load the appropriate verified procedure rather than assuming one national fee, postal address, form or payment method.

### Track C — Follow-up, grievance, first appeal or other remedy

Purpose: prepare the next document only after identifying what happened, the applicable route and the relevant trigger date.

It may include:
- a request for status or acknowledgement;
- a reminder under a procedure that supports one;
- a grievance to a competent authority;
- an RTI first appeal or later appeal where statutory criteria and timing are met;
- a notice-specific appeal/review/objection where the governing law provides one;
- a referral for legal review of possible litigation.

A missed response date may trigger a reminder to assess available remedies. It must **not** automatically mark the demand invalid, the recipient in agreement, the claim waived, or the sender free from a statutory obligation.

## 3. How the supplied template should be used

Preserve its substantive sequence where relevant, but render sections conditionally rather than forcing every article and accusation into every case.

| Supplied section | SIDERETH rendering rule |
|---|---|
| Header, reference, recipient and C.C. | Populate only from user-confirmed notice/contact details and verified official directory data. Never invent an officer, email address, notice number or copy recipient. |
| Preamble and standing | Use a short factual statement of the sender's capacity and connection to the matter. Constitutional standing language is not a substitute for identifying the applicable statutory procedure. Do not automatically assert that every user is directly affected or has standing for every remedy. |
| Demand for documentary evidence | Ask for specific records and, where relevant, their certified copies. Separate an RTI application from the notice reply when each has a different procedural basis. |
| Foundational questions | Generate targeted questions about the exact statutory provision, competent authority, jurisdiction, calculation, applicable rate, process, exemptions and review route. Do not automatically assert that lack of a contract defeats a statutory levy. |
| Questions by constitutional article | Include only where tied to a concrete, sourced issue. Treat the sender's argument as an argument, not a verified court holding or final legal conclusion. |
| Time limit and “tacit agreement” | Preserve an explicit requested response date when useful, but never create silence-as-consent or self-executing legal effects by default. Record non-response as an event and prompt review of next procedural steps. |
| Coercive-action consequences | State the observed or threatened action, request its exact legal basis, and reserve remedies under applicable law. Do not automatically accuse officials of trespass, tort, extortion, intimidation or criminal conduct. |
| Withdrawal of consent | Do not insert blanket or irrevocable withdrawal of consent as default boilerplate against an asserted statutory fee. Consent and statutory obligation are not interchangeable. |
| Authentication and signature | Use a normal declaration, signature and delivery record unless a verified law or prescribed form specifically requires an affidavit, oath, attestation or particular e-filing method. |
| Dispatch | Prepare an export-ready version and a dispatch checklist. Sending/filing is a separate, explicit user-approved action. Preserve submission receipts, tracking results and any acknowledgement. |

The user's original section order and wording remain available as source material. Generated changes must be versioned and inspectable; SIDERETH must not silently overwrite or misrepresent the supplied text.

## 4. Legal-source rules: version, jurisdiction and procedural route

Every legal proposition must be evaluated against at least:
- jurisdiction and relevant public authority;
- source type and issuing authority;
- exact section/rule/order or official procedure;
- publication, effective-from and, if applicable, effective-to date;
- the event date to which the source applies;
- retrieval time and source version/hash where supported;
- verification status;
- claim classification and the exact supporting source reference.

A source title alone is not enough. A result from a search snippet, secondary website, previous draft, or AI output must not be silently upgraded to verified law.

Use the existing `LegalSource`, `LegalProposition`, `VerificationStatus` and `PropositionType` vocabulary in `src/legal_source.rs`, and the existing `LegalSourceRegistry` in `src/legal_source_registry.rs`. These already distinguish verified rules, official procedures, user facts, inferences, uncertainty, disputed interpretations and propositions requiring professional review. Do not build a second legal-source or citation model unless a concrete gap is demonstrated.

### Legal drafting guardrails for this template

1. **Silence, acquiescence and “self-executing notice.”** Never say that failure to respond to this letter automatically means agreement, consent, waiver, invalidity of a levy, or extinguishment of liability. In *Bichitrananda Behera v. State of Orissa* (Supreme Court of India, 11 October 2023, 2023 INSC 902), the Court discusses acquiescence in an equitable context involving knowledge, conduct, assent and delay. It does not establish a universal rule that a unilateral letter can turn every subsequent silence into an agreement. The effect of conduct, delay, limitation or waiver depends on the applicable law and facts. Use a neutral formulation: “I request a written response by [date] and reserve the remedies available to me under applicable law. If no response is received, I will consider the procedure and remedies available on the facts.”
2. **Bharatiya Sakshya Adhiniyam, 2023 (BSA), sections 74–76.** Section 74 classifies public and private documents. Section 75 provides for a certified copy of a public document where the person has a right to inspect it and the legal fee is paid. Section 76 addresses proof by production of certified copies. These provisions should not be stated as a blanket, independent right to obtain every internal note, Cabinet record, legal opinion, personal record or other requested file. The applicable right of access, exemptions and procedure must first be established, often using RTI or the specific record-access law.
3. **Indian Evidence Act, 1872.** The BSA repealed the Indian Evidence Act, subject to BSA section 170's savings for specified applications, trials, inquiries, investigations, proceedings or appeals pending immediately before commencement. Do not add old IEA section 76 as a general “analogous” citation. Use it only after establishing that the savings provision or another applicable rule preserves that route for the matter.
4. **RTI route and timing.** The Right to Information Act, 2005 provides for inspection and copies of existing records. Section 7(1) ordinarily sets a 30-day decision period after receipt of an RTI request, with a 48-hour proviso where the information concerns a person's life or liberty; section 19 establishes the appeal route and timeframes. These are not a blanket 14-day deadline for an ordinary notice reply. Confirm the relevant authority, current rules, fee, trigger date and any special circumstances before calculating a due date.
5. **Current RTI personal-information provision.** Digital Personal Data Protection Act, 2023 section 44(3) amends RTI Act section 8(1)(j). The commencement notification G.S.R. 843(E), dated 13 November 2025, brought section 44(3) into force on publication. The amended clause refers to information relating to personal information. RTI Act section 8(2)'s public-interest override remains relevant. Do not reuse old section 8(1)(j) wording from a pre-amendment guide as though it were current law; inspect the operative text for the request date and apply other relevant exemptions and severability rules.
6. **Constitutional provisions.** Article 265 says no tax shall be levied or collected except by authority of law. Whether a particular demand is a tax, fee, service charge or other levy is a legal classification question requiring the applicable enactment, rules, notifications and facts. Articles 13, 14, 19, 21, 32, 226 and 300A must not be included as an automatic list or presented as proving that a fee is invalid. Article 51A(h) states the citizen's duty to develop scientific temper, humanism and the spirit of inquiry and reform; it does not, by itself, create an unrestricted right to every requested record or an opt-out from a statutory obligation.
7. **Criminal, tort and constitutional litigation claims.** Do not auto-generate allegations of extortion, criminal intimidation, trespass, tort liability or misuse of office because an authority seeks payment or threatens an enforcement measure. Specific offences and remedies have legal elements and depend on fact-specific assessment. Prefer a neutral reservation of rights and route potentially serious allegations to qualified review.
8. **Affidavits and perjury declarations.** Do not insert “under penalty of perjury”, an affidavit demand, or an assertion that an officer must swear to the completeness of every response unless a prescribed legal procedure supports it. Requesting certified copies and requesting an affidavit are distinct.
9. **Email acknowledgement.** IT Act section 12 addresses acknowledgement of receipt of electronic records in specified circumstances; it does not create a universal duty for every public office to acknowledge every ordinary email. Request acknowledgement operationally and preserve the dispatch evidence.
10. **No-contract / consent wording.** Do not treat “No consent–No contract” as a universal defence to a statutory charge. Do not label an amount legally invalid merely because the user has not consented to a contract unless that conclusion is independently supported for the particular obligation.

These controls are legal issue-spotting and source-management requirements, not a determination about any particular user's fee, notice or remedy.

## 5. Mapping to existing SIDERETH responsibilities

Reuse current contracts; do not create parallel legal abstractions.

| Responsibility | Existing SIDERETH boundary | Intended use in this workflow |
|---|---|---|
| Source notice, draft, sent copy, reply | `src/document.rs` — `Document` / `DocumentVersion` | Keep each original and version distinct; capture document type, version, integrity status, privacy class and references. |
| Legal materials and propositions | `src/legal_source.rs`, `src/legal_source_registry.rs` | Store source version, jurisdiction, effective period, verification status and provenance for each legal claim. |
| Administrative route and deadlines | `src/procedure.rs` | Use versioned procedure steps with source references; keep notice deadlines, RTI deadlines and appeal deadlines distinct. |
| Original notice and dispatch evidence | `src/evidence.rs` plus existing evidence trust/repository/query contracts | Preserve the original notice, enclosures, sent letter, postal receipts, delivery tracking, email metadata, acknowledgements and authority response as separate records. |
| Source/transform history | `src/provenance.rs` | Record who supplied a fact, which source supported a legal statement, what text was generated/changed, and which operation created the version. |
| Protected access and user control | Existing authorization and privacy/safety contracts | Control viewing, source upload, external processing, export and dispatch as distinct actions with purpose and scope. |
| Atomic persistence | Existing persistence API and `UnitOfWork` contracts | Commit related state changes through current transaction boundaries, not a new correspondence-specific transaction manager. |
| Tooling or AI assistance | Existing Tool Gateway and intelligence boundaries, if used | AI may propose drafts or identify missing inputs; it cannot create legal authority, fabricate a source, mark its own claim as verified, bypass authorization or send automatically. |

**Current implementation caveat:** these are architectural reuse mappings. The existing domain primitives are present, but this repository has not yet demonstrated a complete correspondence UI, automatic template renderer, RTI portal integration, current-law service, deadline worker or dispatch service. Do not describe this workflow as implemented until the individual acceptance tests and end-to-end evidence exist.

## 6. Data model and claim provenance

Each draft should retain structured inputs instead of burying the entire workflow in a single text field.

Minimum case inputs:
- issuing authority and department;
- public authority and jurisdiction;
- notice/letter reference and date;
- recorded receipt/delivery date and evidence;
- subject, demand amount, calculation and due date exactly as stated;
- statute/rule/order/notification cited by the authority;
- user's stated facts and the source for each fact;
- records sought and why each is relevant;
- selected document track;
- intended outcome and requested response date, if any;
- supporting original documents;
- current status and next-action date.

Each generated claim should be classified as one of:
- verified rule;
- official procedure;
- user-supplied fact;
- inference;
- uncertainty;
- disputed interpretation;
- professional-review-required.

Each legal claim should have a traceable source reference and verification status. When a source does not support the claim, the system must remove it, narrow it, or mark it for review—not invent a footnote.

A generated document must be reproducible from the reviewed inputs, template version, source versions and generation version. Editing a legal paragraph changes the draft version and records the edit; it never changes the preserved original notice or sent letter.

## 7. Workflow lifecycle and review gates

Suggested lifecycle:

`Draft → InputsValidated → SourcesChecked → FactsChecked → ReviewRequired | ReadyForUserReview → UserApproved → Exported → Sent (user-recorded) → DeliveryEvidenceRecorded → ResponseReceived | DeadlinePassed → FollowUp/AppealReview → Closed`

Rules:
- Missing notice, authority, jurisdiction or critical dates must remain visibly missing. Do not fabricate them.
- A source verification failure prevents a legal proposition from being labelled verified.
- High-risk legal claims must block “ready to send” until resolved by the user or qualified review.
- User approval of a draft and authorization to send it are separate decisions.
- Exporting is not dispatch. A transport integration, if later added, requires explicit user authorization at the moment of sending and must produce a durable result or an explicit unknown state.
- The application must distinguish requested reply dates from statutory deadlines.
- A deadline passing creates a review reminder, never an automatic admission, waiver, invalidity or case outcome.
- The user can choose not to include a proposed question or citation. Removing a paragraph cannot delete its underlying source records.
- If the source law changes, a draft should show that its earlier citation may have been superseded; do not silently rewrite a previously sent document.

## 8. Privacy, security and evidence integrity

Apply `docs/architecture/PRIVACY-AND-SAFETY-BY-DESIGN.md`:
- Collect only records necessary for the user's purpose.
- Keep sensitive identifiers, addresses, phone numbers, personal records and case details private by default.
- Make external AI processing and dispatch optional, explicit user choices.
- Preserve source files separately from OCR, summaries, extracted text or AI-generated derivatives.
- Restrict source records and drafts using canonical authorization; do not expose raw storage adapters directly to an agent or presentation surface.
- Record access/export/dispatch events and their relevant source references where required.
- Use content hashes as integrity checks only. A hash does not prove who issued a document, whether its contents are legally authentic, or whether the cited law applies.
- Apply explicit retention/deletion rules and preserve records where a valid legal/evidence-preservation obligation requires it.
- Never include secrets or irrelevant personal data in generated notices or logs.

## 9. Official-source seed list for a legal-source registry

These are starting points for verification, not an exhaustive legal analysis. Record each source as a versioned source and recheck its operative text for the target date/jurisdiction before using it.

1. **Right to Information Act, 2005 — official DoPT consolidated PDF:**  
   https://dopt.gov.in/sites/default/files/RTI%20Act%202005%20%28Updated%29.PDF  
   **Official DoPT RTI guide:** https://rti.dopt.gov.in/RTICorner/Guideonrti.pdf  
   Use to classify requests for existing records, certified copies, fees, public-authority route, response periods, exemptions and appeals. Confirm the current text of section 8(1)(j) against the later DPDP commencement notification.
2. **Bharatiya Sakshya Adhiniyam, 2023 — official India Code PDF:**  
   https://www.indiacode.nic.in/indiacode/bitstream/123456789/20063/1/aa202347.pdf  
   Sections 74–76 concern public/private documents, certified copies of public documents that a person has a right to inspect, and proof by certified copies. Section 170 contains repeal and savings. The Act commenced on 1 July 2024 under the cited commencement notification.
3. **Digital Personal Data Protection Act, 2023 — official MeitY PDF:**  
   https://www.meity.gov.in/static/uploads/2024/02/Digital-Personal-Data-Protection-Act-2023-1.pdf  
   Section 44(3) amends RTI Act section 8(1)(j).
4. **DPDP Act commencement — Gazette of India, G.S.R. 843(E), 13 November 2025:**  
   https://egazette.gov.in/WriteReadData/2025/267647.pdf  
   The notification appoints the Gazette publication date for commencement of section 44(3). The workflow must treat the amended RTI text as the operative version after commencement.
5. **Constitution of India — official India Code copy:**  
   https://upload.indiacode.nic.in/showfile?actid=AC_DD_63_859_00001_00001_1553767976751&filename=constitution_of_india.pdf&type=actfile  
   Use the operative text, including Articles 51A(h), 265, 32, 226 and 300A where relevant. Do not infer that listing an article establishes a claim.
6. **Information Technology Act, 2000 — official India Code copy:**  
   https://upload.indiacode.nic.in/showfile?actid=AC_CEN_45_76_00001_200021_1517807324077&filename=a2000-21.pdf&type=actfile  
   Section 12 concerns acknowledgement of receipt of electronic records; avoid an unsupported assertion of a universal duty to acknowledge routine email.
7. **Bichitrananda Behera v. State of Orissa, Supreme Court of India, 11 October 2023, 2023 INSC 902 — official judgment:**  
   https://api.sci.gov.in/supremecourt/2017/12085/12085_2017_12_1501_47683_Judgement_11-Oct-2023.pdf  
   Use only for the proposition the judgment actually supports about acquiescence, conduct and delay in context. Do not turn it into a universal “silence equals consent” rule.

For state/municipal levies, additional local-government statutes, rules, schedules, orders, official notices and court rulings will usually be necessary. The seed list does not establish the validity or invalidity of any particular levy.

## 10. Acceptance criteria

This workflow is not ready to be called implemented until automated tests establish:

1. Notice reply, RTI application and appeal/follow-up create separate document types and track the correct route-specific recipient, source rules and deadlines.
2. Missing or conflicting facts remain unfilled and prevent finalization when material.
3. Every legal claim carries a source and claim classification, or is flagged as unsupported/review-required.
4. The pre-amendment RTI section 8(1)(j) text is not used as the current law for dates after commencement of DPDP Act section 44(3).
5. BSA sections 74–76 are not presented as a universal access entitlement for all requested public records.
6. Old IEA section 76 is not inserted unless the BSA savings provision or another current legal basis is established for that specific matter.
7. The “silence equals tacit agreement”, “self-executing instrument”, blanket “no consent/no contract”, unqualified perjury declaration, universal affidavit demand and automatic criminal accusations are blocked as default legal effects.
8. A requested 14-day reply period is not confused with the RTI statutory response period. Due-date calculations are traceable to a verified procedure and trigger event.
9. Source notice, generated draft, approved version, sent version, response and delivery records remain separate versioned records with provenance; originals are never silently overwritten.
10. Export and dispatch are separate; no letter is automatically sent without explicit user authorization.
11. Privacy, access control, retention, source-version changes and safe failure have tests.
12. The solution reuses existing SIDERETH boundaries and introduces no new compiler, interpreter, JIT, event loop, generalized rule engine, package split, or correspondence-specific transaction manager without a separately demonstrated need.
13. Exact nine active branches are retained; any branch retirement follows archive, archive verification, unique-commit and dependency review, then explicit disposition.
14. The exact PR head passes Foundation validation and Security/Supply Chain; the final main commit receives a fresh required-check and branch-count verification.

## 11. First implementation milestone

1. Land this reviewed workflow contract.
2. Implement a deterministic field/claim checklist and route classification using existing domain contracts.
3. Add tests for RTI-vs-notice separation, source-backed claims, operative source dates, prohibited default boilerplate, and retention of the original notice.
4. Add only the smallest rendering function required to produce a draft with visible unresolved placeholders. Treat visual presentation/UI as a separate surface, not a second domain model.

Do not start with AI-generated legal prose, external form submission or automated mailing. The initial proof is that the right type of document is selected, its facts and cited sources are traceable, and unsupported legal effects cannot pass silently into a ready-to-send draft.

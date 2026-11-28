# Accessible enrollment — future Ant Kiss proposal

Recorded: **2026-11-26**.
Status: **PROPOSAL FOR FUTURE DESIGN; NOT IMPLEMENTED.**
Related work: MB-05, MB-08, MB-09, MB-10 and MB-12 in
[MASTER_PLAN.md](../MASTER_PLAN.md). Decisions D-02, D-05 and D-06 remain open.
Next coding task remains MB-03E. This record changes no current admission rule,
wire format, funds, population counter, device support or progress percentage.

## 1. User's proposal to preserve

The user wants Mirror to accommodate people who cannot provide the standard
biometric readings, including someone without the designated finger or without
hands. The proposed approach is:

- An accessibility/disability label or record with an explanation of the limitation.
- Review by a committee of **three qualified people** before adding that exception.
- Deliberate controls on subsequent changes or removal, so the exception cannot be
  casually toggled or exploited to obtain another identity.
- Future implementation within the human-supervised Ant Kiss process.

Three is a proposed committee size, not an agreed approval threshold. The user has
not selected unanimity, two-of-three voting, reviewer qualifications, appointment
rules or a public medical-data field.

### Latest clarification — preferred initial combinations, 2026-11-26

The user is relying on the combined evidence; no individual ingredient is expected
to establish uniqueness by itself. Preserve the following preferred starting designs:

| Route | Proposed combination |
|---|---|
| Standard | ECG + one designated pinky fingerprint + human-supervised Ant Kiss |
| Fingerprint unavailable | ECG + a description of the observed capture limitation, such as no hands or unavailable pinky + human-supervised Ant Kiss and the proposed exceptional review |

The earlier possible left-pinky selection is not yet a frozen capture specification.
The user believes these combinations can be sufficient for an initial launch and
suggests possibly adding more fingerprints, even all fingers, as the population
grows. This is a design preference and sufficiency hypothesis, not measured launch
acceptance. No ten-finger requirement, upgrade trigger, approval quorum or actual
launch authorization is introduced. Keep the description minimal and private as
recommended below. Evaluate both complete procedures without assuming either equal
performance or that the accessibility route must be inadequate.

## 2. Proposed interpretation — an alternative verification route

An accommodation records why the ordinary capture procedure cannot be followed.
Observed details can support a reviewer's consistency checks and choice of procedure.
They do not themselves distinguish a unique person: multiple people can share the
same limitation. Evaluate the combined procedure; do not require each ingredient
to solve uniqueness independently. The alternative route still needs a defined
duplicate-enrollment check and continuity with any existing membership.

The user's proposed alternative is ECG plus observed capture limitations and human
review. A different designated finger where usable can also be evaluated, but is
not a selected substitute. Keep other usable required modalities under the chosen
profile. No additional biometric, medical certificate or government identity
document is selected. The ECG capture arrangement must work for the actual applicant;
performance from finger-contact recordings does not establish performance for
other contact positions. See the acquisition studies in the
[device draft's evidence section](DEVICE_INPUT_CONTRACT.md#9-engineering-evidence-for-this-draft).

Human supervision can witness capture and provide recognition/context; reviewers
who have not met someone before still need the shared duplicate-checking process.
Details such as "no hands" must not become an exclusive search filter: descriptions
can differ and physical circumstances can change. Uncertain or conflicting evidence
needs a defined review outcome. For example, a possible fingerprint duplicate must
not automatically become a new person merely because the ECG differs.

Unsupported or unusable input enters an explicit unresolved/retry/review pathway;
it must never be silently translated into a successful no-duplicate result. A
supported exception profile and its evidence must be specified before real admission.

## 3. Privacy recommendation — minimize the record

Prefer a **private accommodation record** containing only the functional information
needed to arrange the alternative procedure. For example: "designated finger
unavailable; an alternative capture route is required." A full diagnosis or
description of the person's body is not automatically necessary.

Whether any public indicator is required remains a design question. Prefer the
ordinary membership outcome with minimal authorized evidence metadata wherever
the protocol permits it. Even a generic public exception flag could reveal or
suggest sensitive circumstances. A future interface must not promise that the
exception is hidden without a demonstrated privacy mechanism; current V2 metadata
is not anonymous.

Descriptions, evidence, consent and access records belong in the protected
off-chain process under a defined access/retention/correction policy. A hash of a
short predictable disability description is not an adequate privacy substitute.
Do not publish descriptions or medical records on-chain, in GitHub, logs or examples.
This proposal authorizes no collection of actual personal information.

## 4. Committee and controlled changes — design questions

The proposed committee reviews the accommodation and the appropriate alternative
verification procedure. It must not receive unlimited power over the person's
membership or funds. The later design needs to settle:

| Question | Still to be decided |
|---|---|
| Qualifications | Relevant verification/accessibility training and how eligibility is established; no automatic medical-profession requirement |
| Selection | Assignment, conflicts of interest, independence, availability and replacement; three identities alone do not prevent collusion |
| Decision rule | Approval threshold, rejected/uncertain cases, reasons, deadlines and appeal |
| Evidence | Minimum private explanation and supported alternative procedure; neither self-declaration nor committee size alone proves uniqueness |
| Authority and binding | Signed decisions tied to the applicant, fresh session, applicable profile and exact requested action |
| Change control | Who may request, approve and audit creation, correction, reassessment or removal; no casual unilateral toggle |
| Privacy | Who sees which information, retention/deletion rules, and any unavoidable public metadata |

Controls must remain compatible with correcting mistakes, changed circumstances
and appeals. "Hard to change casually" must not become an irreversible inaccurate
label. Updating an accommodation is separate from suspending membership or access
to funds. A minimal authorized audit trail need not contain a permanent public
medical explanation.

## 5. Identity continuity and ordinary participation

Proposed invariants for the later design:

- One person keeps the same identity continuity when an accommodation is created,
  changed or removed, or when the verification profile changes.
- Normal and exception routes share the same duplicate-checking scope. They are
  not separate namespaces that allow the same person to enroll once in each.
- A changed route cannot reset enrollment/claim history, count a person twice in
  population, create another bootstrap entitlement or multiply participation.
- An accommodation should not itself increase or reduce ordinary voting, spending
  or reward rights. Existing unresolved economic and consensus policies remain open.
- Existing members who later cannot use the original capture method need a
  continuity/recovery procedure, not a new-person enrollment shortcut.

These are recommendations for review, not new runtime guarantees.

## 6. Future implementation and acceptance

- **MB-05:** design accommodation requests, reviewer assignment, decisions,
  reassessment and appeals with D-02.
- **MB-08:** represent unavailable modalities explicitly and support approved
  alternative capture profiles without guessing or bypassing required evidence.
- **MB-09:** protect minimum explanations and evidence; check duplicates across
  normal/exception profiles under the approved D-05/D-06 design.
- **MB-10:** provide an accessible applicant/verifier workflow, clear outcomes,
  correction requests and understandable status.
- **MB-12:** evaluate actual alternative procedures under their tested conditions;
  synthetic scenarios alone do not establish biological uniqueness.

For each complete proposed route, evaluate the same person returning with a fresh
wallet on another day to different reviewers, genuine new applicants, coordinated
reviewer abuse, and attempts across standard/exception profiles. Measure missed
repeat enrollments and wrongful duplicate flags separately, along with acquisition
failures and review outcomes. Combined performance must be measured; do not multiply
component error rates under an untested independence assumption. Launch criteria
and scope remain to be agreed from actual evidence under D-06.

Later strengthening should use explicit profile versions, preserve the same identity
and claim history, and compare across old/new/exception routes. Additional fingers
are an option to evaluate, not a default collection escalation or a solution for
someone who cannot provide fingerprints. Review performance as the registry grows;
do not wait for a guessed population threshold when failures already occur. A later
upgrade cannot retroactively establish that all earlier admissions were unique.

Future tests should include missing designated finger, unavailable fingerprint
capture, unusable required modalities, a pre-existing member changing route,
concurrent normal/exception enrollments, repeated exception requests, colluding or
unauthorized reviewers, replayed decisions, wrongful labeling/correction, appeals,
privacy leaks and attempted duplicate claims. These are planned tests, not results.

See [device input draft](DEVICE_INPUT_CONTRACT.md),
[decision record](DECISIONS.md), and [current status](../CURRENT_STATUS.md).

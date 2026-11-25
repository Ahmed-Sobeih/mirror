# Device input contract — planning draft v0.2

Status: **DRAFT REQUIREMENTS; no adapter or hardware support implemented here.**
Owner task: MB-08 in [MASTER_PLAN.md](../MASTER_PLAN.md).
Hardware acceptance: MB-12. This file is a compatibility checklist, not an order
for equipment or a promise that an arbitrary consumer device can export readings.
The blockchain receives authorized membership events, not this private capture
payload. No example below requires uploading real readings to GitHub or chat.

Updated 2026-11-26 to record the user's preference for flexible equipment and a
minimal capture profile. The proposed one-finger-plus-ECG profile, possibly using
the left little finger, remains a proposal for evaluation. This document revision
changes no runtime API, consensus encoding or supported-device claim.

## 1. Device-neutral boundary

A combined watch or separate fingerprint/ECG devices are both allowed. Their
adapters provide observations to a local Mirror enrollment application. The
application performs normalization, protection and session binding. An off-chain
service assists duplicate checking; assigned humans review; nodes enforce the
signed admission rules. No device determines identity or creates MRY by itself.

The user's intended flexibility concerns manufacturer, model and transport. A
documented import or adapter can normalize usable observations without embedding
vendor-specific behavior in consensus. The blockchain receives the authorized
enrollment outcome and permitted evidence commitments/approvals. The fingerprint
image/template and ECG waveform remain in the protected off-chain workflow.
Human supervision witnesses the procedure; it does not by itself establish
cross-sensor compatibility, successful population-wide search or honest reviewers.

Initial transports can be a documented file export or a local USB/SDK connection;
additional transports require tests. File import is not automatically evidence
of witnessed live capture. A device that exposes only its internal match result
is not a substitute for the records required by the selected comparison backend.

## 2. Logical envelope (exact wire schema is an MB-08 deliverable)

| Field group | Required meaning |
|---|---|
| Version | Envelope, adapter and capture-profile versions |
| Session | Fresh unpredictable enrollment challenge, applicant credential/request reference, verifier/session authorization, network/policy context |
| Modality | Explicit FINGERPRINT or ECG; never infer it from a filename |
| Provenance | Device model/firmware/SDK and capture mode; distinguish device-attested, application-attested and operator-witnessed claims |
| Time | Capture start/duration, clock source and challenge expiry; local timestamps alone do not prove freshness |
| Payload | Strict encoding identifier, declared dimensions/counts, bounded byte length and integrity binding |
| Quality | Acquisition errors, quality measurements and algorithm versions; no universal invented success threshold |
| Protection | Local authorization, encrypted/authenticated transfer, storage/key-version reference; do not publish decrypting keys |

Capture provenance and device identifiers can themselves be sensitive. Minimize
retention and public exposure. Device signatures, when available, do not prove
biological uniqueness; host signatures alone do not prove the sensor was genuine.

A profile must define minimum/maximum readings, byte sizes, durations, timeouts,
units, finger positions and quality rules before its adapter can be accepted.
Missing required metadata is an error, not permission to guess values.

## 3. Fingerprint observations

Require an export usable by the selected matching implementation: supported image
format with dimensions, bit depth and resolution, OR a documented interoperable
template with exact format/version. Record the specified finger position, capture
attempt and relevant quality metadata. The backend must define comparison support
across sensors and versions; proprietary template bytes are not presumed compatible.

The enrollment profile must consistently specify which fingers are collected and
how repeated captures and failures are treated. The adapter cannot choose a
convenient different finger to bypass duplicate checks. A local phone/device unlock
boolean is not an export of the applicant's fingerprint record.

### Proposed minimal profile — one anatomical finger plus ECG

The user prefers to investigate one finger, possibly the left little finger,
instead of assuming collection of every fingerprint. No ten-finger requirement
has been selected. The exact position, number of repeat captures, matcher and
performance criteria remain D-06 decisions; no evidence currently validates this
particular combined profile for Mirror's intended population.

Several quality-controlled captures of the same anatomical finger can still form
a one-finger profile. If a fixed position is selected, witnesses must check that
position and bind both modalities to the same applicant and fresh session. A
person must not freely substitute another finger to obtain another enrollment.
Missing/unreadable/injured fingers need a defined reviewed exception and continuity
procedure; absence of a comparable record must not be treated as a new person.
Alternative profiles or later profile changes require explicit cross-profile
duplicate-checking and migration rules, not separate identity namespaces.

One-to-many duplicate search is distinct from verifying an already claimed
account. Evaluate missed repeat enrollments, false duplicate flags, acquisition
failures and cross-device/session behavior at the intended operating scale.
Single-finger capture is a data-minimization proposal, not an established accuracy
or anonymity guarantee. References below motivate this evaluation.

No exact hash is treated as a biometric identity: repeated physical measurements
are compared by the chosen matcher. Randomized commitments bind evidence records
without advertising a reusable public fingerprint digest.

## 4. ECG observations

Require an actual electrical waveform, not only heart-rate/BPM, a pulse graph,
an optical PPG series, or a screenshot. Provide samples plus channel/lead/contact
configuration, sample rate, count/duration, numeric encoding, physical units,
calibration/gain/baseline where applicable, filtering and quality/error metadata.

WFDB is a candidate import format, not a mandatory device choice. Its header
specification describes sampling, units and calibration fields [R3 in the master
plan]. Mirror should require the metadata needed for its chosen profile even when
an underlying file format permits defaults. No software guess of unknown voltage
units, missing samples or contact configuration may count as a valid capture.

Both electrical contacts must measure the applicant under the tested procedure.
Human review of the enrollment is separate from medical interpretation: Mirror
makes no diagnostic claim from the recording.

Evaluate supported recording arrangements across separate sessions and time
intervals. ECG studies [D4, D5] show that performance can depend on acquisition
position and evaluation interval. Their results do not validate Mirror's proposed
fingerprint/ECG combination or supply a universal threshold. The joint interpretation
of the two modalities remains to be specified; never assume their errors are
independent or that one automatically compensates for an unusable other modality.

## 5. Adapter operations and failures

Implement versioned equivalents of: capabilities, begin(session), capture(modality),
read/result, cancel, and erase-temporary-data. A retry remains bound to its session;
it cannot create an extra person, approval or grant. Support disconnect/reconnect
without cross-linking two applicants' recordings.

Explicit failure classes include unsupported device/profile, permission denied,
capture unavailable, unusable quality, malformed/truncated/oversized input,
expired/cancelled challenge, substituted session and transport failure. None may
be translated into "no duplicate" or active membership.

Reject malformed numeric values and resource-exhausting metadata before allocating
unbounded memory. Do not allow uploaded file paths or payload URLs to read arbitrary
host files or contact arbitrary internal services. Isolate vendor SDKs and parsers
from the consensus process. Never log payload bytes, keys or unredacted identifiers.

## 6. Matching boundary

A versioned off-chain matching provider returns a bounded private report and a
minimal signed receipt. Bind results to the applicant request, protected evidence,
model/policy versions, required modalities, population snapshot, freshness and
review status. Define NO_MATCH_WITHIN_TESTED_RULES, POSSIBLE_DUPLICATE,
INCONCLUSIVE and ERROR as different outcomes. Human reviewers do not receive an
entire population's records simply because they have been assigned a case.

The private report must not be inserted verbatim into a public transaction.
Only fields permitted by the MB-02 admission format may cross the on-chain boundary.
Authorization of receipt issuers does not establish a mathematical guarantee that
the claimed comparison was correct; disclose and test its trust assumptions.

A result-only interface to the chain is compatible with this boundary. The
authorized private workflow still needs a comparable reference population and
tested matching inputs. A scanner's local unlock/match flag does not establish
that the applicant is absent from the network register. Capture failure, unavailable
search or disagreement between required modalities must remain explicit unresolved
outcomes until the approved review policy resolves them. No automatic admission
fallback is implied by human supervision.

## 7. Supplier/device compatibility questions — before purchase

Ask for model, supported operating systems, documented SDK/export formats,
sample files and schema descriptions, licenses/redistribution conditions,
fingerprint positions/resolution/template interoperability, ECG waveform export
with calibration and lead metadata, offline operation, security/provenance features
and capture-error reporting. Verify whether a vendor cloud/account is mandatory.
Do not promise support before inspecting those materials. No device procurement
or manufacturer contact is authorized by this planning task.

## 8. Required evidence

Software conformance: two differently encoded synthetic sources; parser and
normalization test vectors; invalid metadata/units, session replay, wrong modality,
resource limits, disconnect/cancel and privacy-leak tests. Synthetic captures must
be unmistakably labelled and cannot activate real memberships.

Device conformance: exact tested hardware/firmware/SDK/profile and repeat captures
across sessions and devices; calibrated comparison results and error rates; human
review handling; consent/retention/security review. Keep real recordings off public
CI and Git. Gate A can establish the software interface; only MB-12 can establish
support and recognition performance for actual supplied devices.

## 9. Engineering evidence for this draft

These sources inform evaluation; they do not certify Mirror, mandate government
identity documents or settle its admission rules.

- [D1] [NIST, contactless-to-contact interoperability evaluation (2020)](https://www.nist.gov/publications/interoperability-assessment-2019-contactless-contact-fingerprint-capture):
  measured performance differed across capture arrangements; multiple-finger
  combinations improved results in the evaluated contactless setting.
- [D2] [NIST, FpVTE 2003 results (2004)](https://www.nist.gov/publications/fingerprint-vendor-technology-evaluation-2003-summary-results-and-analysis-report):
  fingerprint quality and number of fingers affected accuracy in that evaluation.
  This historical study supplies no current error rate for a left-pinky profile.
- [D3] [NIST SP 800-63A-4, identity-proofing requirements](https://pages.nist.gov/800-63-4/sp800-63a/ial-general/):
  distinguishes one-to-one verification from one-to-many searches and calls for
  operationally representative evaluation and manual review before duplicate-based
  enrollment rejection within its scope.
- [D4] [Ramos et al., ECG acquisition-protocol experiment (2021)](https://www.mdpi.com/1424-8220/21/14/4645):
  a 20-participant experiment examined acquisition locations and separate sessions.
  Its results support testing compatibility rather than assuming it.
- [D5] [Islam et al., Heartprint multisession dataset (2022)](https://www.mdpi.com/2306-5729/7/10/141):
  evaluated repeat ECG recordings over substantial intervals; baseline results
  depended on the session protocol. They are not a universal ECG accuracy bound.

# OATHYARD Deterministic Physical-Fidelity Reduced Model Architecture

Status: proposed architecture/specification; not implementation evidence.
Date: 2026-06-30T13:57:07Z
Task: `t_217698df` / `PHYS-DETERMINISTIC-REDUCED-MODEL-ARCHITECTURE-001`

This document answers the owner correction that the current OATHYARD combat model is far below the For Honor / Elden Ring-class physical-fidelity bar. It specifies the missing deterministic reduced physical model layer that must exist before renderer, engine, capture, import, PBR, animation, or asset-pipeline work may claim it is solving biomechanics, tissue/flesh, material response, deformation, or armor/weapon contact fidelity.

This document does not implement the model, adopt Bevy, adopt Unreal, adopt Warp/Newton/PhysX/Chrono, import vendor assets, claim high-fidelity visuals, claim owner acceptance, or claim public-demo/release readiness.

## 1. Source boundary and precedence

Controlling sources reviewed, in canon order:

1. `docs/design/GAME_CANON.md`:
   - OATHYARD identity and high-fidelity target: lines 7-15.
   - Fixed 120 Hz deterministic integer/fixed-point truth: lines 41-50.
   - authoritative combat truth definition and inclusion list: lines 52-66.
   - excluded presentation/offline systems and layer taxonomy: lines 68-80.
   - frozen/deterministic/hashed/replayable/cross-platform truth-promotion gates: lines 83-97.
   - current 16-joint body graph: lines 101-124.
   - contact truth path and no-HP health model: lines 134-148.
   - weapon/armor physical requirements: lines 149-156.
   - replay as authoritative evidence: lines 157-160.
2. `docs/design/DEMO_SCOPE.md`:
   - current verified slice is deterministic local duel foundation only: lines 3-14.
   - current presentation evidence is local verification only: lines 29-35.
3. `ACCEPTANCE_MAP.md`:
   - non-negotiable invariants: lines 20-30.
   - high-fidelity gate is not passed: lines 32-46.
   - Bevy/wgpu is selected only as the next renderer spike path, not completion: line 42.
   - local vs public/store gates remain separated: lines 48-103.
4. `AGENTS.md`:
   - determinism rules and forbidden shortcuts, especially fixed truth, loud replay failure, no HP/arbitrary stats, no renderer physics as truth, and no Unreal/Godot/browser/dependency shortcuts for this slice.
5. Parent task `t_7c42a44f` artifact:
   - `artifacts/kanban/physical_fidelity_reset/20260630T133000Z/physical_fidelity_gap_matrix.md`.
   - Finding: current OATHYARD truth is deterministic scalar scaffolding; biomechanics, muscles/tendons/ligaments, tissue/flesh layers, armor-piece deformation, fracture, stress/strain, contact manifolds, and deep material simulation are nonexistent model layers.
6. Parent task `t_e47e5eca` artifact:
   - `artifacts/kanban/phys_reference_sim_research_biomech_tissue_material/t_e47e5eca_source_table.md`.
   - Finding: UE Chaos Flesh/Cloth/ML systems and Warp/Newton/PhysX/Chrono are reference-authoring or runtime-presentation sources only. They are not acceptable authoritative truth unless reduced into OATHYARD-owned fixed-point/hash/replay/cross-platform-verified data by a separate ADR.
7. Current source inspection:
   - `src/lib.rs:79-113` defines `TRUTH_HZ = 120`, current schemas, readiness flags false, and deterministic contact order string.
   - `src/lib.rs:995-1127` stores `FighterState` with exactly 16 `JointState` entries plus scalar balance/momentum/grip/torso/torque/recovery/action-validity fields.
   - `src/lib.rs:1130-1172` builds the current 16 sparse canonical joints.
   - `src/lib.rs:1220-1422` defines current non-HP capability stop conditions, scalar `CapabilityDelta`, and scalar `ContactTrace`.
   - `src/lib.rs:6377-6484` computes current frame costs from scalar Body/Equipment/State/Momentum/Injury factors.
   - `src/lib.rs:6608-6621` sorts contact packets by `frame, attacker, defender, action, target, direction`.
   - `src/lib.rs:6627-6864` resolves contact with scalar reach/coverage/impulse and branch-table material/anatomy strings.
   - `src/lib.rs:6886-7050` serializes current trace JSON with schema, hashes, costs, contacts, capability deltas, and cause chains.
   - `src/content.rs:5-40` defines current weapon and armor scalar profiles.
   - `src/content.rs:426-502` hashes current weapon/armor/fighter/arena content tables.

Conclusion from sources: the existing deterministic envelope is valid and must be preserved. The missing piece is an OATHYARD-owned reduced biomechanics/tissue/material truth model inside that envelope.

## 2. Architecture thesis

OATHYARD should not attempt to run a full FEM/MPM/SPH/DEM/cloth/deformable solver inside live combat truth. The live truth layer must remain a reduced, deterministic, integer/fixed-point model at 120 Hz.

The bridge from high-fidelity reference simulation to game truth is:

```text
offline reference authoring
  -> frozen reference fixture with solver/version/scene/command/seed/units/output hashes
  -> reduction script with explicit integer rounding and hypothesis verdict
  -> OATHYARD reduced tables/curves/fixtures with content hashes
  -> 120 Hz deterministic truth solve
  -> replay/trace/content/table/state hashes
  -> read-only presentation deformation packets for Bevy/Unreal/native renderer
```

Only the OATHYARD reduced tables and OATHYARD deterministic solver may become authoritative combat truth. External solver states, engine physics, renderer deformation, animation events, cloth runtime, neural inference, DCC cache output, or GPU results must not decide live or replayed contact, injury, capability, action legality, frame cost, end state, content hash, or replay hash.

## 3. Non-negotiable truth contract

Every gameplay-affecting value in this architecture is combat truth and must obey all of these constraints:

- Fixed 120 Hz tick.
- Integer/fixed-point storage and arithmetic only.
- No hidden RNG.
- No wall-clock input.
- No gameplay floats.
- No unordered iteration affecting state.
- No renderer/UI/audio/VFX/camera/engine writeback into truth.
- Every source table, reduced curve, asset metadata table, initial state, contact result, state delta, and replay-relevant derived value is versioned and hash-covered.
- Replay re-runs deterministic truth and fails loudly before use on schema, hash, unit, ordering, overflow, missing-table, stale-version, or byte-mismatch errors.
- External solver disagreement is evidence for a falsifying fixture or table revision, not permission to copy external runtime state into truth.

### 3.1 Units and numeric representation

All truth schema fields must include unit suffixes. Proposed canonical units:

| Domain | Unit suffix | Storage |
| --- | --- | --- |
| time | `_tick`, `_frame`, `_frames` | `u32` unless explicitly bounded lower |
| distance / length | `_mm`, `_um` | signed/unsigned integers by field |
| area | `_mm2` | `u32`/`i32` |
| mass | `_g` | `u32`/`i32` |
| inertia | `_g_mm2` or legacy `_g_cm2` | integer, converted at table load if legacy |
| normalized ratios | `_permille` | `0..=1000` unless signed delta |
| fine normalized strain | `_ppm` | parts per million integer |
| force | `_mN` | millinewton integer |
| torque | `_mN_mm` | integer |
| impulse | `_mN_ms` | integer |
| stress / pressure | `_kPa` | integer |
| energy proxy | `_mJ` | integer |
| angle | `_mrad` | milliradian integer |
| velocity | `_mm_per_s` | integer |
| angular velocity | `_mrad_per_s` | integer |

Rounding policy:

- Reference-source floats are never stored directly in truth tables.
- Reduction from offline reference data must choose one explicit mode per field: `floor`, `ceil`, `nearest_even`, or `toward_zero`.
- Runtime arithmetic must use widened intermediates (`i128` equivalent) where multiplication can overflow the target type.
- Schema load must fail loudly on out-of-range values unless a field explicitly declares a clamp policy.
- Gameplay-affecting clamp/saturation policies must be stated per field and tested; silent overflow is forbidden.

## 4. Layered data model

The proposed architecture has four schema layers. Only layer 3 is live authoritative truth.

### 4.1 `offline_reference_authoring`

Purpose: build and falsify reduced model hypotheses from external or analytic sources.

Allowed inputs:

- Chrono/Warp/Newton/PhysX reference scenes.
- UE Chaos Flesh/Cloth/ML Deformer/Physics Asset/Control Rig studies.
- Small analytic closed-form fixtures.
- Hand-authored deterministic mini-scenes.
- Captured solver traces, geometry caches, deformation caches, stress/strain samples, material impact studies.

Required fixture fields:

| Field | Meaning |
| --- | --- |
| `fixture_schema` | e.g. `oathyard.reference_fixture.v1` |
| `solver_id` | `chrono`, `warp`, `newton`, `physx`, `ue_chaos_flesh`, `analytic`, etc. |
| `solver_version` | exact version/build/doc date where available |
| `source_scene_hash` | hash of scene/source inputs |
| `command` | exact command or UI/export recipe recorded as text |
| `seed` | explicit seed or `none`; hidden stochasticity fails fixture admission |
| `units` | source units and canonical conversion mapping |
| `timestep` | source timestep/substep info; reference only |
| `hardware_backend` | CPU/GPU/driver where relevant |
| `raw_output_hash` | hash of solver/cache output |
| `reduction_script_hash` | hash of the script/tool that quantizes to OATHYARD tables |
| `rounding_policy` | explicit per-field rounding policy |
| `reduced_table_hash` | hash of OATHYARD reduced result |
| `hypothesis_id` | reduced-model hypothesis this fixture supports/falsifies |
| `verdict` | `supports`, `falsifies`, `inconclusive`, or `calibration_only` |

Fixtures cannot be read by live truth until a reduced table derived from them is frozen, unit-tagged, hashed, and accepted by an OATHYARD truth ADR.

### 4.2 `asset_metadata_contract`

Purpose: connect source assets to deterministic truth IDs without letting presentation assets become hidden truth.

Asset metadata may contain both truth-bearing and presentation-only fields, but the split must be explicit.

Common required fields:

| Field | Required for | Meaning |
| --- | --- | --- |
| `asset_id` | all assets | stable deterministic ID |
| `asset_kind` | all assets | `fighter`, `armor_piece`, `weapon`, `arena`, `material`, `rig`, etc. |
| `source_files` | all production assets | source paths and hashes under `assets_src/` |
| `provenance` | all production assets | repo-owned/provenance/license status |
| `author_toolchain` | all production assets | tool/version/command/export route |
| `runtime_exports` | runtime presentation assets | generated asset paths and hashes |
| `truth_metadata_hash` | truth-bearing assets | hash of truth metadata fields only |
| `presentation_metadata_hash` | presentation-only assets | hash of visual/runtime fields only |
| `units` | truth-bearing assets | unit map for every numeric field |
| `layer_classification` | all assets | `TRUTH`, `ASSET_METADATA`, `PRESENTATION`, or `offline_research_authoring` |
| `truth_mutation` | presentation assets | must be `false` |
| `readiness_flags` | all production assets | external readiness remains false until evidenced |

Rule: if changing a metadata field can alter contact, material/anatomy solve, capability deltas, action legality, frame costs, end state, replay data, or hashes, that field is combat truth and must be in `truth_metadata_hash` and replay/table hash coverage.

### 4.3 `runtime_authoritative_truth`

Purpose: deterministic reduced physical state and solve at 120 Hz.

This layer is OATHYARD-owned. Proposed schema family:

| Schema | Purpose |
| --- | --- |
| `oathyard.phys_body_graph.v1` | expanded skeletal/body-region hierarchy and mappings to existing 16 joints |
| `oathyard.phys_material_table.v1` | material/tissue/armor/weapon response curves |
| `oathyard.phys_asset_truth_metadata.v1` | truth-bearing asset metadata contract |
| `oathyard.phys_initial_state.v1` | initial body/armor/weapon/material/deformation state |
| `oathyard.phys_contact_packet.v1` | deterministic reduced contact packet/manifold |
| `oathyard.phys_state_delta.v1` | capability/material/deformation/fracture/durability deltas |
| `oathyard.phys_replay_extension.v1` | replay hash coverage and fail-loud schema extension |

These names are proposed for review. They must not be added to source constants until the spec is reviewed.

### 4.4 `runtime_presentation`

Purpose: Bevy/wgpu, Unreal, native renderer, animation, cloth, ML Deformer, Control Rig, PBR, VFX, audio, cameras, and fight-film consume post-hash truth packets.

Presentation may output deformation meshes, blendshape weights, cloth secondary motion, material masks, IK poses, camera cues, particles, sound events, UI text, and screenshots. It may not output authoritative contact, injury, material response, action validity, cost, capability, end state, or hashes.

## 5. Expanded body and skeletal hierarchy

The current 16-joint graph remains the compatibility spine for existing actions, reports, and presentation mapping. The reduced physical-fidelity model adds a body-region graph and anatomical truth nodes around it.

### 5.1 Compatibility rule

The existing canonical joints remain stable compatibility anchors:

```text
0 root
1 spine_lower
2 spine_upper
3 neck_head
4 shoulder_r
5 elbow_r
6 wrist_r
7 shoulder_l
8 elbow_l
9 wrist_l
10 hip_r
11 knee_r
12 ankle_r
13 hip_l
14 knee_l
15 ankle_l
+ grip_r, grip_l frames
```

New truth nodes may be added only with stable IDs and an explicit mapping back to the existing anchors. Existing replay v1 consumers must fail loudly rather than silently reinterpret a v2 physical replay as v1.

### 5.2 Proposed body graph IDs

The first reduced model should target roughly 48-64 truth nodes, not a full anatomical mesh. It must be detailed enough to encode meaningful skeletal, tissue, armor, and contact effects while remaining testable.

Required top-level regions:

| Region group | Required child regions |
| --- | --- |
| head/neck | skull, face/jaw, neck_front, neck_back |
| thorax | sternum, rib_cage_front, rib_cage_back, left_ribs, right_ribs, thoracic_spine |
| abdomen/pelvis | abdomen_front, abdomen_back, pelvis, sacrum, left_hip_socket, right_hip_socket |
| right arm | clavicle_r, scapula_r, shoulder_r, upper_arm_r, elbow_r, forearm_r, wrist_r, hand_r, grip_r |
| left arm | clavicle_l, scapula_l, shoulder_l, upper_arm_l, elbow_l, forearm_l, wrist_l, hand_l, grip_l |
| right leg | thigh_r, knee_r, shin_r, ankle_r, foot_r |
| left leg | thigh_l, knee_l, shin_l, ankle_l, foot_l |
| organs/critical internals | lung_l, lung_r, heart_zone, liver_zone, gut_zone, spine_cord_zone |

Each region row must include:

| Field | Meaning |
| --- | --- |
| `region_id` | stable canonical string |
| `parent_region_id` | stable parent or `none` |
| `legacy_joint_anchor_id` | current 16-joint anchor for compatibility |
| `side` | `center`, `left`, `right` |
| `mass_g` | reduced region mass |
| `center_offset_mm` | offset from anchor in integer mm |
| `extent_mm` | reduced capsule/box/ellipsoid dimensions |
| `collision_proxy_kind` | `capsule`, `box`, `sphere`, `convex_reduced`, or `none` |
| `skeletal_segment_ids` | bones/joints supporting the region |
| `tissue_stack_id` | tissue/flesh layer stack |
| `capability_links` | capability channels affected by damage here |
| `presentation_region_id` | render/rig/deformation hook region |

### 5.3 Skeletal segments and joint constraints

The reduced skeleton is not a ragdoll solver. It is a deterministic constraint and capability model.

Required skeletal segment fields:

| Field | Meaning |
| --- | --- |
| `segment_id` | stable bone/segment ID |
| `parent_segment_id` | deterministic hierarchy |
| `proximal_region_id` / `distal_region_id` | attached body regions |
| `rest_length_mm` | integer reduced length |
| `mass_g` | reduced mass |
| `inertia_g_mm2` | reduced inertia |
| `joint_limit_flex_mrad_min/max` | flex/extend range |
| `joint_limit_abduct_mrad_min/max` | side range where relevant |
| `joint_limit_twist_mrad_min/max` | twist range where relevant |
| `load_path_ids` | ligaments/tendons/muscles transmitting force |
| `fracture_curve_id` | material/fracture response curve |
| `capability_loss_map_id` | capability effects if impaired |

The truth solver should not integrate free-floating rigid bodies in the first implementation. Instead, action labels produce deterministic pose envelopes and load estimates over this graph; contacts produce reduced impulses/stress/strain; state deltas update joint/segment capability and allowed future actions.

## 6. Muscles, tendons, ligaments, and capability loss

The current model has scalar `balance_permille`, `grip_r_permille`, `grip_l_permille`, `torso_rotation_permille`, `torque_permille`, recovery frames, and action-validity booleans. The new model should explain those scalars through explicit tissue/biomechanical channels.

### 6.1 Capability channels

Proposed canonical capability channels:

| Channel | Existing scalar compatibility | Meaning |
| --- | --- | --- |
| `stance_support` | `balance_permille` | ability to support weight and recover stance |
| `right_grip_closure` | `grip_r_permille` | right hand weapon retention/control |
| `left_grip_closure` | `grip_l_permille` | off-hand/shield/control retention |
| `torso_rotation` | `torso_rotation_permille` | ability to rotate trunk for cuts/thrusts/guards |
| `weapon_torque_control` | `torque_permille` | ability to apply/control torque through weapon |
| `recovery_timing` | `recovery_slowdown_frames` | additional frames imposed by injury/load |
| `right_leg_drive` | future | step/lunge/pivot support |
| `left_leg_drive` | future | step/lunge/pivot support |
| `right_shoulder_load` | future | overhead/cut/guard viability |
| `left_shoulder_load` | future | shield/grapple/guard viability |
| `breathing_capacity` | future | fatigue/stagger/recovery coupling |
| `pain_shock_control` | future | transient stagger/guard failure coupling |

No channel is HP. Channels are physical ability constraints that modify action legality, frame cost, recovery, and stop conditions.

### 6.2 Muscle group rows

Required fields:

| Field | Meaning |
| --- | --- |
| `muscle_group_id` | stable ID: e.g. `right_forearm_flexors`, `left_quad`, `obliques_r`, `neck_stabilizers` |
| `origin_region_ids` / `insertion_region_ids` | reduced attachment regions |
| `joint_action` | `flexion`, `extension`, `rotation`, `grip`, `brace`, etc. |
| `max_force_mN` | reduced force capability |
| `fatigue_permille` | deterministic current fatigue/load loss |
| `strain_ppm` | current strain state |
| `tear_state` | `intact`, `strained`, `partial_tear`, `ruptured` |
| `pain_weight_permille` | contribution to pain/shock channel |
| `capability_channel_weights` | weighted map to channels above |
| `action_labels_supported` | action labels whose cost/validity read this row |

### 6.3 Tendon and ligament rows

Required fields:

| Field | Meaning |
| --- | --- |
| `connector_id` | stable ID |
| `connector_kind` | `tendon` or `ligament` |
| `region_a` / `region_b` | attached regions/segments |
| `rest_length_mm` | integer rest length |
| `max_tension_mN` | reduced tension threshold |
| `laxity_ppm` | accumulated laxity/stretch |
| `injury_state` | `intact`, `sprained`, `partial_tear`, `ruptured` |
| `stability_channel` | capability channel affected |
| `invalidated_actions` | optional action labels invalidated at severe state |

Ligament/tendon injury must affect capabilities through explicit channel mappings. Example: `right_wrist_ligament_tear` can reduce `right_grip_closure` and `weapon_torque_control`; it does not subtract hit points.

## 7. Tissue/flesh layer schema

Tissue/flesh is modeled as a reduced layered stack per body region, not as live high-resolution soft-body simulation.

### 7.1 Layer kinds

Required layer kinds:

- `skin`
- `fat`
- `fascia`
- `muscle`
- `tendon`
- `ligament`
- `bone`
- `cartilage`
- `organ`
- `vascular`
- `nerve`

The first implementation may omit some region/layer combinations, but the schema must support them and fail loudly when a referenced required layer is absent.

### 7.2 Tissue layer row

Required fields:

| Field | Meaning |
| --- | --- |
| `tissue_layer_id` | stable ID |
| `region_id` | owning body region |
| `layer_kind` | from layer kinds above |
| `depth_order` | deterministic outer-to-inner integer |
| `thickness_um` | reduced thickness |
| `density_mg_per_mm3` | integer density if used |
| `stiffness_curve_id` | material curve for compression/tension/shear |
| `slash_curve_id` | material curve for edge/slash |
| `pierce_curve_id` | material curve for point penetration |
| `blunt_curve_id` | material curve for blunt transfer |
| `bleed_presentation_hook_id` | presentation-only visual/audio hook, not truth unless promoted |
| `damage_state` | truth state if gameplay-affecting |

### 7.3 Tissue damage state

Minimum truth states:

| State | Meaning | Gameplay effect allowed |
| --- | --- | --- |
| `intact` | no persistent impairment | none |
| `bruised` | blunt compression response | pain/recovery/temporary capability loss |
| `cut` | edge discontinuity in outer tissue | pain/presentation/wetness hook; possible grip/recovery impact by region |
| `punctured` | pierce path through layer | region-specific capability loss |
| `torn` | tissue/connective partial failure | reduced force or stability |
| `crushed` | severe compression | balance/recovery/breathing channel loss |
| `fractured` | bone/cartilage failure | action invalidation/channel loss |

Blood, wetness, swelling, discoloration, and gore visuals are presentation hooks unless a reduced state explicitly affects capability, action validity, frame cost, or end condition.

## 8. Armor layers, straps, gaps, and fasteners

Armor is truth when it changes contact, impulse transfer, deflection, binding, detachment, action cost, or capability. Armor visuals are presentation when they only alter rendered meshes/material masks after truth hashes.

### 8.1 Armor piece row

Required fields:

| Field | Meaning |
| --- | --- |
| `armor_piece_id` | stable ID |
| `loadout_id` | loadout/fighter association |
| `piece_kind` | `helmet`, `gorget`, `breastplate`, `mail_shirt`, `gambeson`, `vambrace`, `gauntlet`, `greave`, etc. |
| `covered_region_ids` | body regions potentially covered |
| `coverage_patch_ids` | deterministic coverage/gap map references |
| `layer_stack_id` | armor material layers |
| `mass_g` | truth mass |
| `inertia_g_mm2` | truth inertia |
| `attachment_ids` | straps/fasteners/anchors |
| `binding_feature_ids` | edges, hooks, protrusions that affect bind |
| `deformation_state_id` | state row |
| `detachment_state` | `attached`, `loosened`, `partially_detached`, `detached` |
| `presentation_mesh_ids` | post-hash display assets |

### 8.2 Armor layer row

Required fields:

| Field | Meaning |
| --- | --- |
| `armor_layer_id` | stable ID |
| `piece_id` | owning piece |
| `depth_order` | outer-to-inner order |
| `material_id` | metal/mail/leather/cloth/wood/bone/etc. |
| `thickness_um` | integer thickness |
| `hardness_permille` | reduced hardness |
| `toughness_permille` | reduced durability/resistance |
| `flex_permille` | reduced bending/flex |
| `deflection_curve_id` | curve for oblique contacts |
| `penetration_curve_id` | curve for pierce/cut |
| `blunt_transfer_curve_id` | curve for blunt/compression |
| `deformation_curve_id` | curve for dents/tears/bending |

### 8.3 Coverage/gap map

Coverage cannot remain four scalar target zones. The reduced model needs deterministic coverage patches.

Required fields:

| Field | Meaning |
| --- | --- |
| `patch_id` | stable ID |
| `region_id` | covered body region |
| `local_grid_u` / `local_grid_v` | small integer patch coordinates or range |
| `coverage_permille` | coverage probability is forbidden; this is deterministic area coverage/occlusion score |
| `edge_exposure_permille` | gap/edge exposure score |
| `normal_mrad` | reduced outward orientation if used |
| `gap_kind` | `none`, `seam`, `joint`, `visor`, `armpit`, `elbow_inside`, `knee_back`, `strap_gap`, etc. |
| `fastener_dependency_ids` | gaps affected by loose/broken fasteners |

The contact solver must use deterministic geometry/patch lookup, not RNG. Same inputs must choose the same coverage/gap result and serialize the selected patch/gap into contact packets.

### 8.4 Strap/fastener row

Required fields:

| Field | Meaning |
| --- | --- |
| `fastener_id` | stable ID |
| `fastener_kind` | `strap`, `buckle`, `lace`, `rivet`, `hinge`, `chain_link`, `toggle` |
| `anchor_region_or_piece_a/b` | attachment endpoints |
| `tension_mN` | current reduced tension |
| `max_tension_mN` | failure threshold |
| `slip_permille` | looseness/slip state |
| `failure_state` | `intact`, `loosened`, `torn`, `broken`, `unlatched` |
| `coverage_patch_effects` | deterministic changes to gaps/coverage |
| `detachment_effects` | piece detachment rules |

## 9. Weapon contact geometry and durability

The current weapon scalar profile is retained as compatibility metadata but is insufficient for physical fidelity.

### 9.1 Weapon truth row

Required fields:

| Field | Meaning |
| --- | --- |
| `weapon_id` | stable ID |
| `weapon_kind` | sword, axe, spear, shield, mace, billhook, etc. |
| `length_mm` | existing-compatible length |
| `mass_g` | mass |
| `center_of_mass_from_primary_grip_mm` | center of mass |
| `inertia_g_mm2` | moment of inertia |
| `grip_frame_ids` | deterministic grip frames |
| `contact_feature_ids` | edge/point/blunt/hook features |
| `material_layer_ids` | metal/wood/leather/etc. layers where relevant |
| `durability_state_id` | weapon state row |
| `presentation_mesh_ids` | display assets |

### 9.2 Contact feature row

Required fields:

| Field | Meaning |
| --- | --- |
| `feature_id` | stable ID |
| `weapon_id` | owning weapon |
| `feature_kind` | `edge`, `point`, `blunt_face`, `hook`, `guard`, `haft`, `pommel`, `shield_rim`, `shield_face` |
| `local_start_mm` / `local_end_mm` | deterministic segment or local reduced coordinates |
| `radius_um` | edge/point/blunt radius |
| `contact_area_mm2_min/max` | reduced contact area bounds |
| `alignment_window_mrad` | orientation range for effective contact |
| `material_id` | response material |
| `attack_label_affinity` | allowed action labels |
| `durability_curve_id` | damage response |
| `bind_curve_id` | hook/bind/friction behavior |

### 9.3 Weapon state row

Required fields:

| Field | Meaning |
| --- | --- |
| `weapon_state_id` | stable ID |
| `weapon_id` | owning weapon |
| `edge_integrity_permille` | edge durability/sharpness |
| `point_integrity_permille` | point durability |
| `haft_integrity_permille` | handle/shaft durability |
| `bend_mm` | reduced deformation |
| `crack_length_mm` | reduced crack state |
| `looseness_permille` | head/guard/strap looseness |
| `state_kind` | `intact`, `dulled`, `bent`, `cracked`, `loosened`, `broken` |
| `capability_effects` | effects on action validity/cost/control |

Weapon damage only becomes truth when it changes contact, cost, action validity, capability, end condition, or hashes. Pure scratches are presentation masks.

## 10. Material response curves

The material/anatomy branch table must be replaced or wrapped by reduced integer response curves.

### 10.1 Curve row

Required fields:

| Field | Meaning |
| --- | --- |
| `curve_id` | stable ID |
| `curve_kind` | `compression`, `tension`, `shear`, `slash`, `pierce`, `blunt`, `deflection`, `fracture`, `fatigue`, `bind`, `friction` |
| `source` | `hand_authored`, `reference_reduced`, `analytic`, etc. |
| `source_fixture_hashes` | optional non-authoritative reference fixtures |
| `input_units` | canonical unit map |
| `output_units` | canonical unit map |
| `segments` | ordered integer piecewise segments |
| `rounding_policy` | runtime/reduction rounding policy |
| `range_policy` | loud fail vs clamp/saturate |
| `table_hash` | content hash |

### 10.2 Segment row

Each segment must be monotonic by input range and sorted by `(input_min, input_max, segment_id)`.

| Field | Meaning |
| --- | --- |
| `input_min` / `input_max` | integer range |
| `slope_num` / `slope_den` | rational slope |
| `offset` | integer offset |
| `output_min` / `output_max` | optional clamp bounds |
| `state_transition` | optional damage/deformation state transition |
| `capability_delta_map_id` | optional capability effects |

No lookup may depend on hash-map iteration order. Duplicate or overlapping segments fail schema load.

### 10.3 Response outputs

A material/tissue/armor/weapon response may output:

- `deflected_permille`
- `absorbed_impulse_mN_ms`
- `transmitted_impulse_mN_ms`
- `penetration_depth_um`
- `slash_depth_um`
- `dent_depth_um`
- `bend_delta_mm`
- `crack_delta_mm`
- `tear_delta_permille`
- `fastener_tension_delta_mN`
- `tissue_damage_state_delta`
- `capability_delta_map_id`
- `presentation_event_id`

Only the state/capability parts are truth. Presentation events are read-only outputs after hashes.

## 11. Stress, strain, and impulse abstractions

The reduced model should not pretend to be full continuum mechanics. It should compute deterministic low-dimensional abstractions that can be tested against reference fixtures.

### 11.1 Contact packet inputs

A reduced contact packet must include, at minimum:

| Field | Meaning |
| --- | --- |
| `schema` | `oathyard.phys_contact_packet.v1` |
| `turn` / `frame` | truth timing |
| `attacker` / `defender` | fighter IDs |
| `action` / `direction` / `target_region_id` | committed action context |
| `attacker_pose_envelope_id` | deterministic action pose envelope |
| `defender_pose_envelope_id` | deterministic defense/stance envelope |
| `weapon_feature_id` | selected weapon contact feature |
| `body_region_id` | selected body region |
| `armor_piece_id` / `coverage_patch_id` | selected armor/gap, if any |
| `contact_normal_mrad` | reduced normal/orientation bucket |
| `relative_velocity_mm_per_s` | reduced velocity proxy |
| `lever_arm_mm` | torque arm |
| `contact_area_mm2` | reduced contact area |
| `normal_impulse_mN_ms` | computed impulse component |
| `shear_impulse_mN_ms` | computed shear component |
| `torque_impulse_mN_mm_ms` | computed torque component |
| `stress_kPa` | reduced stress proxy |
| `strain_ppm` | reduced strain proxy |
| `ordering_key` | serialized deterministic ordering key |
| `input_table_hashes` | hashes for body/material/asset tables used |

### 11.2 Deterministic formulas

The exact formulas must be implemented later, but the formula contract is fixed here:

- Inputs are integers or rational integer pairs only.
- Intermediate products use widened signed integers.
- Division has explicit rounding policy.
- Contact selection is sorted by stable IDs and frame order.
- If two candidate contacts tie, the serialized ordering key decides, not insertion order.
- Contact packets serialize every selected body/armor/weapon/material ID needed to reproduce the solve.
- Stress/strain values are abstractions for table lookup, not claims of real SI-accurate continuum simulation.

### 11.3 Deterministic solve order

Required order per frame:

1. Validate committed actions and action legality from previous truth state.
2. Build deterministic pose envelopes from action labels, direction, body graph, and capability channels.
3. Generate candidate weapon/body/armor contacts using reduced geometry.
4. Sort candidates by `frame, attacker, defender, action_order, target_region_order, weapon_feature_id, body_region_id, armor_patch_id`.
5. Select bounded contacts by deterministic priority and schema limit.
6. Apply armor coverage/gap/deflection/binding solve.
7. Apply weapon/material response curves.
8. Apply tissue/flesh/body-region response curves.
9. Merge capability deltas by deterministic channel order.
10. Apply deformation/fracture/durability/fastener state deltas.
11. Recompute action validity, cost modifiers, stop conditions.
12. Serialize trace/replay fields and update state hash.

## 12. Deformation, fracture, and durability state

Gameplay-affecting deformation is truth. Visual deformation that only changes rendered meshes after hashes is presentation.

### 12.1 State classes

Required truth state classes:

| Class | Examples |
| --- | --- |
| body tissue state | bruise/cut/puncture/torn/crushed by layer/region |
| bone/cartilage state | hairline fracture/fracture/dislocation/stability loss |
| muscle/tendon/ligament state | strain/partial tear/rupture/laxity |
| armor deformation state | dent/bend/tear/pierce/loose/detached |
| weapon durability state | dulled/bent/cracked/broken/loose |
| fastener state | loosened/torn/broken/unlatched |
| arena/footing state | optional later: slick/debris/obstacle/contact damage |

### 12.2 State row fields

| Field | Meaning |
| --- | --- |
| `state_id` | stable row ID |
| `owner_kind` | `body_region`, `tissue_layer`, `armor_piece`, `weapon_feature`, `fastener`, etc. |
| `owner_id` | owning row |
| `state_kind` | class-specific enum |
| `severity_permille` | deterministic severity |
| `deformation_mm` | bend/dent/displacement where relevant |
| `crack_length_mm` | fracture/crack extent where relevant |
| `tear_permille` | tear/damage extent |
| `durability_permille` | remaining durability |
| `capability_effect_map_id` | effects on channels |
| `action_validity_effects` | explicit action labels affected |
| `presentation_hook_id` | visual/audio hook after hash |

### 12.3 Merge policy

Multiple contacts in the same frame must merge deterministically:

- Sort deltas by the same contact order key.
- Apply state changes in sorted order.
- Clamp or fail according to field policy.
- Serialize pre-state, delta, and post-state hashes for replay debugging.
- If two deltas target the same exclusive state transition, use declared severity/order rules and serialize the chosen rule.

## 13. Replay schema, hash coverage, and loud failure

The physical-fidelity model must extend replay/trace evidence before it is allowed to affect gameplay.

### 13.1 Replay extension fields

Proposed `oathyard.phys_replay_extension.v1` fields:

| Field | Meaning |
| --- | --- |
| `phys_schema` | schema version |
| `phys_model_hash` | hash of body graph + material tables + solve config |
| `phys_asset_truth_metadata_hash` | hash of truth-bearing asset metadata |
| `reference_fixture_index_hash` | hash of reference fixture index used to derive tables, if any |
| `initial_phys_state_hash` | initial body/armor/weapon/deformation state hash |
| `per_turn_phys_contact_hashes` | ordered hashes of physical contact packets |
| `per_turn_phys_delta_hashes` | ordered hashes of state deltas |
| `final_phys_state_hash` | final reduced physical state hash |
| `legacy_state_hash_bridge` | mapping from new state to old final_state_hash fields during transition |
| `failure_policy` | loud failure rules |

### 13.2 Loud failure conditions

Replay/schema verification must fail loudly if any of these occur:

- Missing physical schema version when a physical replay extension is expected.
- Unknown schema version.
- Missing or mismatched body/material/asset/reference table hash.
- Unit suffix missing or inconsistent.
- Noncanonical ordering of tables, contacts, deltas, or state rows.
- Duplicate stable IDs.
- Overlapping material curve segments.
- Invalid range/clamp policy.
- Integer overflow or out-of-range reduced value.
- Unknown body region, tissue layer, armor piece, weapon feature, fastener, or capability channel.
- Presentation packet tries to declare `truth_mutation:true` or includes a writeback field.
- Replay byte mismatch, trace mismatch, final state hash mismatch, content hash mismatch, or physical state hash mismatch.
- Reference fixture hash referenced by a truth table is absent or stale.

### 13.3 Hash boundary

Hash-covered truth inputs must include:

- Body graph rows.
- Tissue layer rows.
- Muscle/tendon/ligament rows.
- Capability channel mapping rows.
- Armor piece/layer/coverage/fastener rows.
- Weapon geometry/material/durability rows.
- Material response curves and segment rows.
- Reduction policies and source fixture hashes for reduced tables.
- Initial fighter/loadout/body/armor/weapon/deformation state.
- Scenario commits after the commit boundary.
- Per-turn contact packets and state deltas.

Presentation-only outputs must include source truth hashes in their manifests but must not be read by replay truth.

## 14. Asset metadata contract

Production source assets must be able to feed truth metadata and presentation metadata separately.

### 14.1 Fighter asset contract

Fighter source assets must provide or link:

- High-fidelity presentation mesh/rig/skin weights, presentation-only until truth promotion.
- Canonical truth-joint mapping to existing 16 anchors plus expanded body-region IDs.
- Body-region map and reduced collision/contact proxies.
- Tissue stack references per region.
- Muscle/tendon/ligament group references and default capability baselines.
- Damage/deformation mask IDs for presentation hooks.
- Armor attachment anchor IDs.
- Source/provenance/toolchain/runtime export hashes.

### 14.2 Armor asset contract

Armor assets must provide:

- Separate piece IDs.
- Coverage/gap patch maps.
- Layer stacks and material curve references.
- Strap/fastener/hinge/rivet/lace rows.
- Mass/inertia truth values.
- Deformation/durability state definitions.
- Collision/contact regions.
- Presentation mesh/material/deformation hooks.
- Source/provenance/toolchain/runtime export hashes.

### 14.3 Weapon asset contract

Weapon assets must provide:

- Grip frames.
- Edge/point/blunt/hook/guard/shaft/shield contact feature rows.
- Reduced geometry and contact area ranges.
- Mass distribution and inertia values.
- Material layer references.
- Durability/fracture/deformation state definitions.
- Presentation mesh/material/deformation hooks.
- Source/provenance/toolchain/runtime export hashes.

### 14.4 Arena/footing contract

Arena assets must provide at least:

- Footing surface material IDs.
- Collision/obstacle reduced geometry.
- Slip/friction response curves if gameplay-affecting.
- Debris/terrain state only if promoted to truth.
- Lighting/camera/weather as presentation-only unless separately promoted.

The first implementation may postpone footing/debris beyond explicit surface/friction metadata, but the schema must reserve a deterministic path rather than route footing through renderer physics.

## 15. Presentation deformation hooks for Bevy and Unreal

### 15.1 Common post-hash packet

All renderer/engine integrations consume the same read-only physical presentation packet.

Proposed schema: `oathyard.phys_presentation_packet.v1`.

Required fields:

| Field | Meaning |
| --- | --- |
| `schema` | packet schema |
| `source_replay_schema` | replay schema used |
| `scenario_id` | scenario |
| `truth_hz` | 120 |
| `content_hash` | content hash |
| `phys_model_hash` | body/material/table hash |
| `replay_json_hash` | replay artifact hash |
| `trace_json_hash` | trace artifact hash |
| `final_state_hash` | authoritative final hash |
| `final_phys_state_hash` | physical extension final hash, if present |
| `generated_after_replay_verify` | must be true |
| `truth_mutation` | must be false |
| `body_region_states` | read-only region transforms/state buckets |
| `skeletal_anchor_states` | read-only legacy + expanded anchors |
| `contact_events` | read-only contact/material/anatomy events |
| `deformation_events` | read-only deformation/fracture/durability deltas |
| `material_mask_events` | read-only visual material/wetness/wear hooks |
| `capability_events` | read-only channel deltas/stop conditions |
| `presentation_region_map_hash` | hash of mapping to renderer skeleton/meshes |
| `forbidden_writeback_fields` | explicit list of fields renderer may not emit back |

### 15.2 Bevy/wgpu hook

Bevy is the selected V1 renderer spike path in `docs/decisions/0009-production-renderer-selection.md`, but this spec does not add a Bevy dependency.

A future Bevy presentation adapter should:

- Load the post-hash packet and production visual manifest.
- Map `body_region_states` to Bevy transforms/bones/animation graph inputs.
- Map `deformation_events` to blendshape weights, skeletal constraints, material masks, mesh morphs, or shader uniforms.
- Map `material_mask_events` to PBR damage/wetness/dirt/wear masks.
- Map `capability_events` to presentation-only animation state such as stagger/collapse/recovery.
- Emit capture/frame timing manifests with `truth_mutation:false`.
- Prove replay JSON, trace JSON, contact packets, costs, capability deltas, end condition, content hash, final hash, and physical state hash are byte-identical with Bevy path enabled/disabled.

### 15.3 Unreal hook

Unreal is not adopted and is explicitly blocked behind owner/toolchain/license/ADR gates. The hook exists only so the truth model has a clean presentation boundary if a future Unreal evaluation occurs.

A future Unreal adapter should:

- Consume the same post-hash packet, likely through JSON/USD/glTF custom extras or an import bridge.
- Drive Control Rig/FBIK, ML Deformer, Chaos Flesh/Cloth, material masks, and camera systems only after truth hashes exist.
- Keep UE Physics Asset/Chaos collision outputs presentation-only unless a separate truth ADR promotes a reduced exported table.
- Serialize Unreal capture manifests with engine version, project ID, packet hash, replay hash, final hash, and `truth_mutation:false`.
- Prove byte-identical truth artifacts with Unreal enabled/disabled before any visual output is treated as valid presentation evidence.

No Unreal runtime physics, cloth, flesh, ML deformer, Control Rig, animation notify, collision event, or Blueprint/C++ state may decide OATHYARD truth.

## 16. Reference-simulation reduction plan

The reduced truth model should be falsified against high-value reference cases, not invented from vibes.

### 16.1 Initial reference fixture families

Smallest useful fixture families:

1. Edge-on-mail over gambeson:
   - Expected reduced output: edge mostly deflected/absorbed, blunt transfer to ribs/torso, recovery/torso capability loss.
2. Spear point through armor gap at weapon arm:
   - Expected reduced output: selected gap patch, penetration/tissue injury, right grip and weapon torque loss.
3. Blunt strike against plate over torso:
   - Expected reduced output: dent/deformation state, blunt transfer curve, balance/recovery loss without arbitrary damage number.
4. Hook/billhook catch on strap or shield rim:
   - Expected reduced output: fastener tension, binding/torque/grip loss, possible loosened/detached state.
5. Knee/leg strike causing stance collapse risk:
   - Expected reduced output: leg drive and stance_support channel loss.
6. Tendon/ligament stress in wrist/ankle under torque:
   - Expected reduced output: tendon/ligament strain state and action validity/cost effect.
7. Soft tissue compression/cut/puncture over layered flesh:
   - Expected reduced output: tissue layer damage states and presentation hooks.

### 16.2 Fixture admission gate

A fixture can influence truth tables only if all pass:

- Source scene is stored or reproducible from source-controlled text.
- Solver/version/build/toolchain is recorded.
- Command/seed/timestep/units/backend are recorded.
- Raw output hash is recorded.
- Reduction script hash is recorded.
- Reduced integer table hash is recorded.
- Hypothesis verdict is stated.
- Resulting truth table passes deterministic replay/hash tests.

If any are missing, the fixture remains research notes only.

## 17. Smallest implementation and test plan

No implementation should start until this spec is reviewed. After review, implement in the smallest independently verifiable sequence.

### Phase 0: Review and freeze the spec boundary

Files:

- Review: `docs/design/PHYSICAL_FIDELITY_ARCHITECTURE.md`.
- Do not modify Rust source.

Acceptance:

- Code/design reviewer confirms required schema areas are present.
- Reviewer confirms no renderer/engine/capture/import work is treated as the solution.
- Reviewer records any schema renames or scope cuts before code starts.

### Phase 1: Add schema-only fixtures and validators

Goal: fail loudly on malformed body/material/asset schemas without changing duel outcomes.

Proposed future files:

- `content/physical/body_graph.v1.oyphys`
- `content/physical/material_curves.v1.oyphys`
- `content/physical/asset_truth_metadata.v1.oyphys`
- `src/physical_schema.rs`
- `tests/physical_schema_negative.rs`

Verification:

- `cargo test --locked physical_schema`
- Negative tests for duplicate IDs, missing units, overlapping curve segments, bad ranges, unknown region references, and stale hashes.
- `./tools/audit_truth.sh`

### Phase 2: Add expanded body graph compatibility bridge

Goal: map expanded body regions to existing 16 joints without changing current replay semantics until enabled.

Verification:

- Existing replay v1 still verifies unchanged.
- Expanded graph hash is deterministic and stable across runs.
- Unknown/ambiguous legacy joint mappings fail loudly.
- `cargo test --locked body_graph`
- `./tools/replay_verify.sh artifacts/verify_a/replay.json` after a current duel run.

### Phase 3: Add reduced material curve evaluator in isolation

Goal: pure integer curve lookup/evaluation with no integration into combat yet.

Verification:

- Unit tests for monotonic segment order, exact rounding, range failure, clamp policy, and overflow handling.
- Fixture tests from analytic small cases.
- No gameplay float usage.
- `cargo test --locked material_curve`
- `./tools/audit_truth.sh`

### Phase 4: Add physical contact packet prototype behind explicit schema gate

Goal: generate additional physical contact evidence from existing duels without replacing the current branch-table outcome.

Verification:

- Current replay/trace/final hash unchanged when the prototype packet is presentation/evidence-only.
- Contact packet ordering is deterministic.
- Packet includes selected body/armor/weapon IDs, table hashes, and stress/strain/impulse abstractions.
- `./tools/presentation_truth_isolation.sh`
- focused replay equality checks.

### Phase 5: Promote one narrow physical solve path to truth by ADR

Goal: replace one current branch-table path only after schema, fixtures, negative tests, and replay hash coverage are ready.

Candidate: edge-on-mail/gambeson blunt transfer, because current code already has a scalar `mail_absorbed_edge_with_blunt_transfer` branch.

Verification:

- RED test shows current scalar branch cannot express required armor/tissue state.
- GREEN implementation uses body/armor/tissue/material curves and serializes state deltas.
- Replay schema extension is present and fails loudly on mismatch.
- Current canonical gates pass fresh:
  - `./tools/build.sh`
  - `./tools/test.sh`
  - `cargo build --locked`
  - `cargo test --locked`
  - `./tools/verify.sh`

### Phase 6: Add post-hash presentation packet for Bevy/Unreal/native renderer

Goal: make physical state consumable by presentation without writeback.

Verification:

- Packet generated only after replay verification.
- Packet contains content/replay/trace/final/physical hashes.
- Presentation-on/off truth artifacts are byte-identical.
- `truth_mutation:false` manifests are produced.
- No Bevy/Unreal dependency is added unless a separate renderer/backend ADR explicitly approves it.

## 18. Review checklist before implementation

A reviewer should reject implementation if any answer is missing:

- Does every gameplay-affecting field have a unit suffix and hash coverage?
- Is every external solver output classified as offline reference or presentation unless separately promoted?
- Are existing 16 joints preserved as compatibility anchors?
- Does the body-region hierarchy include tissue, skeletal, muscle/tendon/ligament, and capability links?
- Are armor gaps/straps/fasteners represented as deterministic state, not renderer-only art?
- Is weapon contact geometry represented beyond scalar edge/blunt/pierce/hook values?
- Are material response curves integer/rational and overlap-free?
- Are stress/strain/impulse abstractions explicitly reduced proxies rather than fake high-resolution physics?
- Are deformation/fracture/durability states separated into truth vs presentation?
- Does replay fail loudly on schema/hash/unit/order/table mismatches?
- Can Bevy/Unreal consume post-hash packets without truth writeback?
- Does the plan avoid HP, arbitrary damage numbers, DPS, crit chance, armor points, speed/damage bonuses, or hidden stat boosts?
- Does the plan preserve current public-demo/release/owner/legal/store readiness false?

## 19. Explicit non-goals

This spec does not:

- Implement any Rust code.
- Add or approve a Bevy dependency.
- Add or approve an Unreal dependency.
- Add Unity, Godot, browser-first frameworks, vendored blobs, network services, telemetry, installers, or release packaging.
- Promote Chaos Flesh, Chaos Cloth, ML Cloth, ML Deformer, Physics Assets, Control Rig, Warp, Newton, PhysX, Chrono, FEM, MPM, SPH, DEM, XPBD/PBD, or any external solver to authoritative truth.
- Claim For Honor or Elden Ring quality has been achieved.
- Claim high-fidelity production renderer completion.
- Claim production asset completion.
- Claim owner visual acceptance.
- Claim public-demo readiness or release-candidate readiness.
- Add HP, hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- Use canned animation, animation notifies, renderer collision, or pre-decided hit results as gameplay truth.
- Copy reference-game names, assets, silhouettes, factions, UI, animations, lore, characters, textures, music, or proprietary mechanics.

## 20. Gate on downstream renderer/asset/import work

Renderer/engine/capture/import/PBR/animation work may proceed as presentation research or scaffolding only if it states that this physical-fidelity model is still absent or incomplete. It may not claim to solve the owner correction until at least these are true:

1. This spec or successor ADR is reviewed and accepted.
2. Schema validators exist for physical body/material/asset truth metadata.
3. At least one reduced physical solve path is hash/replay covered and fails loudly on mismatch.
4. Presentation packet isolation proves Bevy/Unreal/native rendering cannot mutate truth.
5. Canonical verification gates pass fresh on the current tree.

Until then, OATHYARD's current model remains deterministic scalar scaffolding, not the target biomechanics/tissue/material/deformation system.

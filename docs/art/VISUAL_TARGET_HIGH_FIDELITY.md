# OATHYARD High-Fidelity Visual Target

Status: Target accepted; current evidence remains candidate/failing baseline.
Date: 2026-06-30

## Quality references

Elden Ring and For Honor are quality/readability references only. OATHYARD must not copy names, assets, silhouettes, characters, factions, music, UI, animations, textures, camera shots, or proprietary mechanics.

## OATHYARD-specific target

OATHYARD must read as an original dark-fantasy judicial duel:

- verdict ring and oath/witness stone;
- witness positions and judicial staging;
- worn blood-washed stone with cuts, scuffs, wetness, dirt, ash, and maintenance props;
- weapon staging and loadout identity;
- layered armor, straps, fasteners, cloth/leather/mail/plate material separation;
- physically plausible, size-varied weapons with readable edge/point/blunt/hook/contact functions;
- threatening arena scale and cinematic lighting;
- fog/dust/mist/weather/wetness hooks;
- best-of-five duel presentation, planning timeline, contact, injury/capability consequence, replay browser, and fight-film proof.

## Current gate state

```text
candidate_asset_preview: true for existing candidate native 3D capture metadata only
production_asset_ready: false
in_engine_visual_ready: false
high_fidelity_ready: false
public_demo_visual_ready: false
owner_visual_accepted: false
```

## Pass criteria

A production visual gate passes only when all are true:

- provenance/licensing status exists for every asset;
- source and runtime files exist;
- asset validation passes;
- production assets load in a native renderer/engine;
- OATHYARD arena exists;
- production lighting/materials/shadows/reflections/GI approximation or equivalent are visible;
- fighter, weapon, armor, arena, gameplay, replay, and fight-film screenshots exist at 1920x1080+;
- gameplay captures show planning, contact, damage/material response, injury/capability consequence, replay, and fight-film;
- visual benchmark and gap reports exist;
- presentation truth isolation passes;
- owner visual acceptance is obtained or explicitly pending/false.

## Explicit non-evidence

Rodin turntables, isolated asset previews, legacy local debug captures, low-poly glTF, untextured meshes, primitive silhouettes, and metadata-only reports cannot satisfy high-fidelity visual completion.

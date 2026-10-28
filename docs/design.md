# Quadra: design notes

The why behind decisions, and the results of any tests that informed them. The bible says what was decided; this says why. Newest first.

## M1 setup answers (2026-10-27)
- **Decision:** Apache-2.0 licence; public GitHub repo "Quadra"; our own tool names from the start with Max's name in tooltips; trade mark check before the first public release.
- **Why:** Apache-2.0 matches Cycles and has a patent grant (MIT and GPL-3.0 rejected). The repo is needed for releases and the update check. Own names now avoid a big rename later, and tooltips keep Max users oriented. The trade mark check does not block building.

## How doole reviews (2026-10-27)
- **Decision:** doole checks each step as it lands, rather than one review at the end of a milestone.
- **Why:** doole's choice; more control over a large project. Claude stops after each step with exact test steps.

## Open source (2026-10-27)
- **Decision:** Quadra is free and open source. The exact licence is picked at M1 start.
- **Why:** doole's choice. The Cycles route allows any licence, so nothing in the engine forces one.
- **Evidence:** `docs/research/architecture.html` section 9.2.

## Milestone order: viewport first (2026-10-27)
- **Decision:** follow the research plan M1 to M6. M1 starts with a wgpu viewport prototype that must show 10M triangles at 60 FPS.
- **Why:** the viewport is the biggest unknown; no public benchmark shows wgpu at that size. Proving it first means a failure changes the plan before much is built. A crude end-to-end skeleton first was rejected because it would leave that risk untested.
- **Evidence:** `docs/research/architecture.html` sections 3 and 6; `docs/research/rust-stack.html` section 2.

## Main window look: mockup A (2026-10-27)
- **Decision:** the main window is a near copy of the 3ds Max 2026 default dark UI (`docs/mockups/a-familiar.html`), with our own icons, at about 85% of the reference UI scale.
- **Why:** Max muscle memory is the first pillar; switchers should find every panel where they expect it. Mockups B (one big view, search first) and C (workspace tabs and ribbon) were rejected.
- **Evidence:** colours and positions measured from the labelled interface image in Autodesk's 3ds Max 2026 help (Interface Overview). Colours: UI #444444, viewports #353535, perspective gradient #262626 to #4c4c4c, toggles #5f8ac1, rollouts #515151, buttons #656565, track bar #2a2a2a, active view border #a58c3c, selection outline #f2dc3a.

## Engine route: standalone Cycles (2026-10-27)
- **Decision:** a native Rust app linking standalone Cycles through our own C++ wrapper. Blender is never linked.
- **Why:** keeps the app free of GPL, no Blender or Python start-up cost, and Cycles' session API supports live viewport updates. Linking bpy or forking Blender was rejected (forces GPL, heavy start-up). The cost is owning the wrapper and losing Blender add-ons.
- **Evidence:** `docs/research/blender-engine.html`.

## Name: Quadra (2026-10-27)
- **Decision:** renamed from "4dMax" to "Quadra".
- **Why:** "Max" risks confusion with Autodesk's trade mark. "Quadra" still needs a UK IPO and EUIPO trade mark check before public release (Apple used it in the 1990s).

## egui 0.36, not 0.37 (2026-10-28)
egui_dock 0.21.1, the newest release, needs egui 0.36. egui 0.36 still uses wgpu 30, so stepping down one minor version keeps docking with no other cost. Move to 0.37 when egui_dock supports it. Source: https://crates.io/crates/egui_dock/0.21.1/dependencies

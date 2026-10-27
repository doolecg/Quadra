# Quadra: systems

What each system is, where it lives and how it connects to the others. Update when a system is added or changes. Nothing is built yet; every system below is planned (from `docs/research/architecture.html` sections 1 and 2) and gets its real "Where" when its first code lands.

## quadra_ui
- **What it does:** docked main window (mockup A layout), command panel, explorers, editors, settings page.
- **Where:** not built (planned crate `quadra_ui`).
- **Talks to:** sends commands to quadra_cmd; reads the scene for display.

## quadra_input
- **What it does:** keymap (480 bindings from `docs/research/keymap.json`), mouse combos, quad menus, rebinding.
- **Where:** not built (planned crate `quadra_input`).
- **Talks to:** turns keys and clicks into actions on quadra_cmd.
- **Saved:** the user's keymap file.

## quadra_cmd
- **What it does:** command bus, unlimited undo/redo, macro recording.
- **Where:** not built (planned crate `quadra_cmd`).
- **Talks to:** every edit from UI, input and scripts passes through it into quadra_scene.

## quadra_scene and quadra_stack
- **What it does:** node tree, layers, selection sets; modifier stack as a dependency graph evaluated in parallel with a cache per modifier.
- **Where:** not built (planned crates `quadra_scene`, `quadra_stack`, with `quadra_geom`, `quadra_modifiers`, `quadra_create`).
- **Talks to:** hands evaluated meshes to quadra_viewport and quadra_render.
- **Saved:** the scene, as USD.

## quadra_viewport
- **What it does:** real-time display in wgpu: four views, shading modes, gizmos, picking, snaps, grids.
- **Where:** not built (planned crate `quadra_viewport`).
- **Talks to:** reads evaluated meshes from quadra_stack.

## quadra_render (Cycles bridge)
- **What it does:** node materials, render setup, frame buffer; sends scene changes to Cycles.
- **Where:** not built (planned crates `quadra_material`, `quadra_render`, C++ `cycles_bridge`).
- **Talks to:** diffs the scene and sends only changes to Cycles through cxx.

## quadra_anim, quadra_sim, quadra_script, quadra_plugin, quadra_io
- **What it does:** animation and rigging; particles and physics; Python API; native plugin ABI; USD/FBX/OBJ/glTF, autosave and crash journal.
- **Where:** not built (planned crates of the same names).
- **Talks to:** all go through quadra_cmd and quadra_scene.

## quadra_app
- **What it does:** the `quadra` program; starts the app. Today it prints the version and licence notice.
- **Where:** `crates/quadra_app`.
- **Talks to:** will start quadra_ui and quadra_viewport.

## quadra_vp_proto
- **What it does:** M1 measuring tool. Draws a generated 10M-triangle scene with wgpu and logs frame times. Not part of the app.
- **Where:** `crates/quadra_vp_proto`.
- **Talks to:** nothing; standalone.

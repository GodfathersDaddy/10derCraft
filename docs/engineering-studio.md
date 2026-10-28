# Engineering extension: native tabs and reference studies

The extension retains the original SolidCraft application bar, CAD tabs, toolbar,
browser, viewport and navigation. Additional tabs host MBS, FEM/ROM, interfaces,
worksheet, variants, studies and load-path tools. Selecting a tab only changes the native ribbon. The central CAD viewport and original side panels remain unchanged. Engineering forms and block editors open in closable native-style windows only after an explicit tool action. There is no persistent engineering inspector. The worksheet button opens SolidCraft's original parameter dialog. The earlier separate studio shell
was rejected and removed from current source. Its historical screenshots are not
current UI evidence.

`--studio` creates an original generic robot using ordinary engine commands in an
empty document. Normal launches also expose the extra tabs. No commercial CAD
geometry or artwork is included.

## Implemented source behavior

- Mechanical port drafts separate frame pose, geometric support and load transfer.
- Captured CAD picks are provisional locators, not persistent topology or qualified
  FEM boundaries. Point forces and pressure loads cannot act as full joint ports.
- Representation previews retain port IDs. No ROM is generated.
- The common editor distinguishes mechanical links, directed signals and planned
  CAD/FEM/ROM artifact dependencies. The robot graph remains a draft.
- `studio.beam.evaluate` computes an independent rectangular cantilever reference.
- CAD length bindings use existing dimensioned parameters. Cross-sections and loads
  are not automatically inferred from CAD geometry.
- `studio.job.create` returns a JSON job for the optional external reference worker.
- `studio.job.accept` checks the exact job snapshot and rejects changed CAD length
  bindings. Importing results never modifies CAD geometry.

The worker lives outside the Rust application repository in the parent `simulation/`
research directory. It uses installed Exudyn and SciPy packages through public APIs.
Rust does not link these packages or require Python at application startup. The
external helper uses the registered commands through authenticated local control;
the existing MCP server exposes the same registry.

## Model limits

The beam uses mm, N, MPa and kg/m³; Exudyn receives SI quantities. The Exudyn model
is a single tip mass and analytic beam end stiffness, without distributed beam
inertia. It is not a FEM/CMS ROM or flexible robot. SciPy performs local analytic
section sizing with dimension bounds, deflection and applicability constraints.

Assembly simulation, persistent engineering configuration, automatic rigid grouping,
qualified surface couplings, FEM, CMS and automatic optimization feedback to CAD
remain future work. Engineering state is session-only; the CAD format is unchanged.
External-result schema acceptance is not a physical validation of those results.

## Upstream integration

Base: `d8eb65d26e36af74fc573c094f3106d0626fbcb6` (20 October 2026).
Remote: `upstream`; branch: `tendercraft/studio-prototype`. Nothing is pushed.

Most extension code is in `engine/src/studio.rs`, `engine/src/studio_beam.rs`,
`engine/src/cmd/studio.rs` and `ui-egui/src/engineering_studio.rs`. Glue touches the
registry/session, native toolbar, central panel and optional demo flag. No existing
geometry kernel implementation is replaced.

After local validation and a backup/commit, integrate updates in a separate branch:

```powershell
git fetch upstream
git switch -c integration/upstream-update
git merge upstream/main
# Resolve conflicts, build and verify original CAD plus extension.
git switch tendercraft/studio-prototype
git merge integration/upstream-update
```

This describes a local update, not publication permission. No commit is made before
required project gates pass. The previous broad suite remains paused; only bounded
new checks are authorized. Actual checks are recorded in the parent output folder.

SolidCraft keeps its product name, licenses and third-party notices. This extension
introduces no solver source, commercial assets or new icons.

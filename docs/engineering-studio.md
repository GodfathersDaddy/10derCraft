# Engineering extension: native tabs and reference studies

The 10derCraft extension retains the native SolidCraft CAD shell and navigation.
The project tabs are Start, Parameter, CAD, FEM and MKS. MKS now opens a full-page
native rigid-block view with background-rendered pictures of the exact member
bodies and placements. FEM opens a ribbon above the selectable CAD viewport;
Results switches to an integrated, navigable deformation view. Selecting these
tabs does not open a floating editor. Explicit interface/property actions can
still open temporary forms. Parameter uses the embedded local worksheet.

For prepared robot projects, `robot.fem.solve` computes the existing locked-actuator
two-beam model. The read-only `robot.fem.snapshot` command exports a current result
as PNG, with display amplification and camera recorded in its response. The same
scene builder supplies the UI: cubic Hermite bending, axial displacement and
cross-section rotation deform axially subdivided circular beam display meshes.
Upstream beam displacement/rotation is superposed on the downstream beam.
The scalar colours show unamplified displacement magnitude; non-beam CAD is an
undeformed translucent reference. This is not solved solid FEM or flexible MKS.

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

The paragraph above describes the older independent reference-study worker.
The native suite separately has persistent project ports, rigid grouping, assembly
dynamics and the limited robot beam FEM. Qualified CAD-surface FEM boundaries,
general solid meshing/solving, CMS and flexible MKS feedback remain future work.
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

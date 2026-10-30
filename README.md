# 10derCraft

Parametric CAD with a local calculation worksheet, assemblies, multibody simulation and engineering studies. This research prototype extends [SolidCraft](https://github.com/tangency-labs/solidcraft) while retaining its CAD workspace and adding Parameter, CAD, FEM and MKS tabs.

Based on SolidCraft 0.27.1, upstream commit `ef5985f841a1ff993ef1deb39e2962968b0a5e95`. Original authorship, licences and attribution are retained. The original documentation is in [docs](docs/); the [upstream introduction](docs/solidcraft-upstream-readme.md) is preserved separately.

Future updates can follow the retained upstream history: [integration workflow](docs/10dercraft/UPSTREAM_AKTUALISIEREN.md).

## Start on Windows

Install Rust and Visual Studio C++ build tools as described in [building](docs/building.md). Run `powershell -NoProfile -ExecutionPolicy Bypass -File Build-10derCraft.ps1`, then open **Start-10derCraft.cmd**. Choose projects under **Beispiele** on the home page. The starter also accepts a model file as an argument. This is a source checkout; generated binaries and installed runtimes are not committed.

For local simulation, install Python and create its separate environment:

```powershell
python -m venv simulation/.venv
.\simulation\.venv\Scripts\python.exe -m pip install -r simulation/requirements.txt
```

For the local Mathcad-style worksheet backend, install .NET 10 SDK and run `powershell -NoProfile -ExecutionPolicy Bypass -File Build-Calcpad-Local.ps1`. This fetches a pinned MIT-licensed CalcpadCE source revision and builds a short-lived local helper. No calculation server is used. See [integration details](integrations/calcpad-local/README.md).

## Included projects

- Editable planetary first stage: native involute equation geometry, calculation parameters and assembly.
- Robot arm: parameterized CAD, large-motion example, independent motion sources, sensors, joint-force plots and a dense drag-and-drop workflow.
- Pendulum: ground, revolute joint, arm and end weight, with 1001 stored output points.
- Additional calculation-to-CAD, mechanical-interface, mass-property and variant examples.

The [example catalogue](10derCraft.examples.json) contains paths; [checksums](examples/10dercraft-manifest.json) identify the included project files. Calculations are saved inside projects; optional standalone worksheet documents are under [examples/calculations](examples/calculations/). A simple parameterized plate with FEM is deferred.

## MKS workflow and video

CAD bodies and joints define the physical assembly. In MKS, drag a motion source onto a joint, edit it, confirm its connection, attach a sensor and connect its measurement output to Scope. Start simulation to calculate and play the 3D movement. Changed physical inputs require a new calculation. Display interpolation does not manufacture new force results.

**Video speichern** in the MKS ribbon exports a current simulation as AVI/MJPEG in the background. Default: 640 × 360 pixels, 30 fps, fixed fitted camera. The export interpolates joint-relative poses and keeps the source result unchanged. It refuses existing destinations; choose a new filename. Limits: 601 frames, 150 million total pixels, 128 MiB. PNG and animated GIF remain separate. MP4 and audio are not included.

This is a research prototype, not a complete Simscape or Modelica implementation. Supported dynamics are bounded open rigid-body trees with revolute/prismatic joints. Robot FEM is a locked-actuator beam model; general solid FEM, flexible MKS, contact and arbitrary controllers remain further work. Upstream analytic drawing projection currently reports native involute surfaces as unsupported.

## Research and verification

[Integration report](docs/10dercraft/RELEASE_2026-10-30.md), [research log](docs/10dercraft/DISSERTATION_FORSCHUNGSLOG.md), [MKS library status](docs/10dercraft/MKS_BIBLIOTHEK_STATUS.md), [CAD–MKS workflow](docs/10dercraft/MKS_CAD_WORKFLOW.md) and [software roadmap](docs/10dercraft/SUITE_FAHRPLAN.md) describe evidence and limits.

Engine operations are registered commands and accessible through the existing control/MCP path. Video uses `mks.result.video`; calculation references and CAD updates use the same engine as the UI.

Full project gate before commits: `cargo xtask ci`. The integration report records actual checks; a successful build alone is not a physical validation of every example.

This snapshot was finalized with focused checks at the owner’s request; the complete native test suite is still pending. Static gates, full Clippy, Wasm, doctests, project opening and native robot/video workflows were checked. See the [verification scope](docs/10dercraft/evidence/focused-release.json).

## Licence

See [MIT](LICENSE-MIT), [Apache-2.0](LICENSE-APACHE), [ATTRIBUTION.md](ATTRIBUTION.md) and [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). Scientific Python packages and CalcpadCE are optional separate processes, with their own licences. Personal input documents, local credentials, caches and installed toolchains are excluded.

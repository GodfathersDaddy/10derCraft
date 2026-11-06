# 10derCraft

Parametric CAD with a local calculation worksheet, assemblies, multibody simulation and engineering studies. This research prototype extends [SolidCraft](https://github.com/tangency-labs/solidcraft) while retaining its CAD workspace and adding Parameter, CAD, FEM and MKS tabs.

Based on SolidCraft 0.27.1, upstream commit `ef5985f841a1ff993ef1deb39e2962968b0a5e95`. Original authorship, licences and attribution are retained. The original documentation is in [docs](docs/); the [upstream introduction](docs/solidcraft-upstream-readme.md) is preserved separately.

Future updates can follow the retained upstream history: [integration workflow](docs/10dercraft/UPSTREAM_AKTUALISIEREN.md).

## Start on Windows

Windows development snapshots are published under [Releases](https://github.com/GodfathersDaddy/10derCraft/releases). Extract the entire ZIP to a writable folder and open **Start-10derCraft.cmd**. The complete portable package includes the GUI, CLI/MCP, examples, local Calcpad helper and a separate Python runtime with Exudyn, NumPy and SciPy. Its `BUILD_INFO.json` records the source commit and verification scope. Development snapshots are unsigned prereleases.

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
- Solid-FEM/ROM flexible arm, two-flexible-arm crane and the detailed six-axis robot with two hollow-profile ROMs, explicit rigid attachments and saved numerical results.
- Robot with mounted catalogue gripper motor, adapter plate and native standard fasteners.
- Assembled 34-part Advent calendar with radial drawer joints and a removable top.

The [example catalogue](10derCraft.examples.json) contains paths; [checksums](examples/10dercraft-manifest.json) identify the included project files. Calculations are saved inside projects; optional standalone worksheet documents are under [examples/calculations](examples/calculations/). A simple parameterized plate with FEM is deferred.

## Updated robot examples

The examples include a statically preloaded two-ROM robot and a separate shoulder-yoke assembly with three parameterized machined plates and forty native fasteners. Both contain a calculated six-axis movement. The optional static initialization is available in the MKS ribbon. The remaining gearbox, bearing and manufacturing qualification are documented in the robot reports. [Static initial state](docs/10dercraft/ROBOTER_STATISCHE_VORSPANNUNG.md), [shoulder assembly](docs/10dercraft/ROBOTER_SCHULTERGABEL.md)

## MKS workflow and video

CAD bodies and joints define the physical assembly. In MKS, drag a motion source onto a joint, edit it, confirm its connection, attach a sensor and connect its measurement output to Scope. Start simulation to calculate and play the 3D movement. Changed physical inputs require a new calculation. Display interpolation does not manufacture new force results.

**Video speichern** in the MKS ribbon exports a current simulation as AVI/MJPEG in the background. Default: 640 × 360 pixels, 30 fps, fixed fitted camera. The export interpolates joint-relative poses and keeps the source result unchanged. It refuses existing destinations; choose a new filename. Limits: 601 frames, 150 million total pixels, 128 MiB. Detailed assembly previews and exports share a bounded budget of 250,000 triangles and 500,000 total vertices, including flexible surfaces. PNG and animated GIF remain separate. MP4 and audio are not included.

Supported dynamics include bounded open rigid and mixed rigid/FFRF trees with revolute/prismatic joints. Component FEM supports linear-elastic TET4/TET10 volume meshes, RBE2 transfer surfaces and Craig-Bampton reduction. Current assembly modal histories can be displayed in the individual component view. Existing ROM settings can be reopened from a block, preserving imported mesh data and surface order. Forces and moments can be edited in the FEM form.

Native joint sensing also provides local torque components and their magnitude. Scope includes separate collective selections for total joint torque and revolute-axis Mz. The servo library contains an independently modelled Panasonic MSMF202L1H8 mounting envelope with catalogue mass. Its motor-side mounting module and a separate robot installation layout are included under Examples. [Method and remaining mechanical work](docs/10dercraft/MKS_GELENKMOMENTE_UND_A2_ANTRIEB.md)

Automatic volume meshing covers verified native boxes, cylinders and coaxially bored tubes. A flexible part can retain all other CAD bodies as explicitly assigned rigid attachment groups with their mass and inertia. The detailed six-axis example now includes two coupled hollow-profile ROMs and all its attached CAD bodies. Its remaining mechanical drive connections and manufacturing verification are still open. The [flexible robot workflow](docs/10dercraft/FLEXIBLER_ROBOTER_WORKFLOW.md) documents the modelling assumptions and the calculated example. Contact, a functioning gripper transmission and arbitrary controller execution remain open. See the [robot workflow status](docs/10dercraft/ROBOTER_ROM_WORKFLOW_STATUS.md). Upstream analytic drawing projection currently reports native involute surfaces as unsupported.

## Research and verification

[Integration report](docs/10dercraft/RELEASE_2026-10-30.md), [research log](docs/10dercraft/DISSERTATION_FORSCHUNGSLOG.md), [MKS library status](docs/10dercraft/MKS_BIBLIOTHEK_STATUS.md), [CAD–MKS workflow](docs/10dercraft/MKS_CAD_WORKFLOW.md) and [software roadmap](docs/10dercraft/SUITE_FAHRPLAN.md) describe evidence and limits.

Engine operations are registered commands and accessible through the existing control/MCP path. Video uses `mks.result.video`; calculation references and CAD updates use the same engine as the UI.

Full project gate before commits: `cargo xtask ci`. The integration report records actual checks; a successful build alone is not a physical validation of every example.

The earlier release has its own [verification scope](docs/10dercraft/evidence/focused-release.json). The current development snapshot has broad tests paused at the owner's request. Its [checkpoint record](docs/10dercraft/ZWISCHENSTAND_2026-11-06.md) distinguishes earlier checks from subsequent untested edits. Building or packaging the application supplies no additional physical qualification.

## Licence

See [MIT](LICENSE-MIT), [Apache-2.0](LICENSE-APACHE), [ATTRIBUTION.md](ATTRIBUTION.md) and [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). Scientific Python packages and CalcpadCE are optional separate processes, with their own licences. Personal input documents, local credentials, caches and installed toolchains are excluded.

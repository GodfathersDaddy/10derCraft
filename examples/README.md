# 10derCraft projects

Open the normal Start page and choose **Beispiele**. The root catalogue resolves
paths relative to the checkout, including folders with spaces. Project files keep
their calculation definitions, CAD features, assemblies and saved presentation.
`.solidcraft` remains a supported native extension in SolidCraft 0.27.1.

The principal gearbox is `projects/KUE-Planetenstufe-bearbeitbar.solidcraft`.
The latest short robot demonstration is `projects/Roboter-DragDrop-Workflow.solidcraft`;
the separate large-motion and all-domains models remain available in the catalogue.
The dense large-motion copy is `projects/Roboter-Grosse-Bewegung-Fein.solidcraft`:
five seconds, 5000 integration steps, 1001 stored output points.
The pendulum is `projects/Pendel-Feine-Ausgabe.solidcraft`.

`media/` contains actual exported AVI/MJPEG examples: robot at 30 fps (including
a five-second large-motion video) and pendulum at 15 fps. Stored solver outputs and video frames are different sampling
grids. Their numbers and AVI structure were independently checked; see the
integration report and `docs/10dercraft/evidence/`.

Optional standalone calculations are under `calculations/`. A project's embedded
calculation is authoritative for its CAD bindings; importing a worksheet alone
does not copy another project's reference bindings.

No simple plate/FEM example was found; the user explicitly deferred that addition.

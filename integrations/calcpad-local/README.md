# Local CalcpadCE integration

CalcpadCE is invoked as a short-lived console process with JSON on standard input
and output. It has no server, HTTP endpoint or background service. The Rust engine
starts it on Calculate, committing an inline formula/unit edit, or selection of
a saved Calcpad variant.
Both standard streams use UTF-8 without BOM, including mathematical comparison
symbols, regardless of the Windows console code page.

Upstream: https://github.com/imartincei/CalcpadCE
Source revision: `0cf438f7d4a78a7ee50724eefb712c23f2ec09c5` (MIT).
The upstream licence remains at `external/CalcpadCE/LICENSE`.

Run `powershell -NoProfile -ExecutionPolicy Bypass -File Build-Calcpad-Local.ps1`
from the workspace. It uses the workspace .NET 10 SDK, local NuGet cache and publishes
a self-contained Windows helper to `output/calcpad-local`. No global runtime install
is required. The desktop finds that folder relative to its executable's ancestors;
`SOLIDCRAFT_CALCPAD_HELPER` can override the executable path for packaging.

`SolidCraftExports.cs` is our small partial-class extension, compiled into Core.
The build script adds a capture call before Core clears its parser. This exports
unrounded finite real scalars with their units, not numbers scraped from HTML.
Vectors, matrices, complex values and nonfinite numbers are excluded from CAD exports.
Compound units remain valid calculation results; CAD accepts length, angle and
dimensionless references only. The formatted HTML is retained as data. The native
worksheet now provides directly editable two-dimensional fractions, powers and
subscripts, alongside the optional source/table view.

The optional JSON `units` map requests display units per variable. Original
`results` remain unchanged; `display_results` contains unrounded converted values
and `display_errors` reports invalid unit syntax. Dimension-compatible component
requests include Nm with mm -> Nmm. Incompatible dimensions preserve the requested
factor explicitly, e.g. 2 s with m -> 2 (s)*(1/(m))*(m), marked `forced_factor`.
These display values are presentation data and never replace CAD export scalars.

Embedded mode supports `#if`, `#else` and `#end if` for calculation variants.
Other `#` directives and `$` macros are rejected. File imports, plots,
iteration/solver macros and general Calcpad document UI controls need separate support.
The desktop enforces a 10-second process limit and 4 MiB output cap. Calculation
source is limited to 128 KiB. This is not a general untrusted-code sandbox.

Published CAD references are literal snapshots. Recalculation and variant selection
change the result table; `worksheet.apply` updates those snapshots atomically after
review. Source edits invalidate the cached exports until successful recalculation.
The entire calculation, variants, current result state and references persist in
the CAD project. `.scmath` exports definitions, Calcpad source/results and variants;
it does not transfer the project's CAD reference bindings to another project.

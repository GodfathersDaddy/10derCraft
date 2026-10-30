# Native Windows project/video check

After building 10derCraft and setting up the optional simulation environment:

```powershell
.\simulation\.venv\Scripts\python.exe -m pip install -r tools/10dercraft/requirements.txt
.\simulation\.venv\Scripts\python.exe -u tools/10dercraft/verify_release_video.py
```

Uses its own application instance and control token, opens the catalogue projects,
exports pendulum and robot videos, independently decodes every JPEG frame and
checks the AVI headers, timing, index, movement and overwrite protection. It also
exercises the ordinary video export dialog while querying UI responsiveness.
Only its own app instance is closed. Generated outputs go to ignored `output/`.

`SOLIDCRAFT_TEST_EXE` can select a separately built desktop executable. The helper
is for Windows desktop acceptance checks, separate from Rust's full `cargo xtask ci`.
Pillow is a verification dependency, not an application dependency.

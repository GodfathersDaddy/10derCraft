//! Component FEM models may explicitly assign each non-elastic CAD occurrence
//! to a reduced surface and native joints to those interfaces. Flexible results
//! retain separate rigid-attachment trajectories, mass and inertia accounting.
//! Older designs omit these fields and retain complete-block ROM semantics.
//! Earlier releases require an update instead of discarding attachment data.

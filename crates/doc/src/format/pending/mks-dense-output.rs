//! Dense MKS result arrays are stored in a bounded, copy-on-write document field,
//! separate from descriptive metadata. Old documents default to no such field;
//! legacy inline and per-body metadata trajectories remain readable. Earlier
//! releases ask for an update rather than silently dropping the dense results.

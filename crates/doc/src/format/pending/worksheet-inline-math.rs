//! Worksheet presentation adds significant_digits (2..16), unit_overrides (name to
//! requested output unit), unit_values (unrounded converted display scalars),
//! unit_errors and unit_values_source (SHA-256 of raw Calcpad scalar results), and
//! pinned_references (up to 128 scalar names). Native converted values additionally
//! carry raw_value/raw_unit for validity. These are presentation only: CAD reads
//! the original full precision calculation results and explicit CAD aliases.
//! Existing positions/notes/charts and legacy CAD references remain compatible.

//! Calculation documents reuse saved user parameters as their single dependency graph.
//! The existing namespaced metadata object `org.solidcraft.worksheet` stores title and notes.
//! No additional parameter representation or migration is needed; older designs have no sheet
//! description, and continue to expose their existing user parameters as calculations.

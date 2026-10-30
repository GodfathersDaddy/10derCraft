//! Design metadata org.solidcraft.calculation-families stores up to 32 named tables,
//! each with 1..32 unique named columns and 1..128 named rows of scalar expression
//! strings. An optional active row records the last successful selection. Selecting
//! creates ordinary Calcpad family_column definitions; CAD snapshots stay explicit.
//! Worksheet presentation additionally retains suspended_references (source to
//! prior CAD binding) for reversible unpinning without deleting CAD dimensions.

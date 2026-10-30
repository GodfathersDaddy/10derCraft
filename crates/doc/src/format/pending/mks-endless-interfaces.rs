//! Mechanical project ports gain optional label, role (load_in/load_out/bidirectional)
//! and area_face (persistent planar CAD face name, requires surface + joint_origin).
//! Missing fields preserve the existing point/declared-area behaviour. Area is
//! re-evaluated from CAD and missing or ambiguous faces invalidate load evaluation.
//! Roles describe engineering intent; a mechanical connection remains reciprocal.

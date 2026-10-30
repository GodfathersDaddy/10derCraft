//! Mechanical ports may reference an associative native named joint_origin.
//! Missing joint_origin preserves explicit legacy coordinate/quaternion ports.
//! MKS presentation may store unscoped_sensors: existing sensor blocks whose Scope
//! wire is disconnected. Missing list preserves previous connected sensor behaviour.
//! Previous releases ask for an update instead of dropping these fields.

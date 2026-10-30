//! Independent MKS motion sources and typed source-to-native-joint inputs are saved
//! in design metadata org.solidcraft.mks-diagram. Null inputs explicitly disconnect
//! former drivers. Absence preserves legacy job/presentation actuation unchanged.
//! Older releases request an update rather than silently discarding the graph.

//! LRP (Language Runtime Protocol) v2.0.0-rfc-kernel
//! Body is joined at build time from part_a + part_b (CI-friendly push size).

include!(concat!(env!("OUT_DIR"), "/lrp_joined.rs"));

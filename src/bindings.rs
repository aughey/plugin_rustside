#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct plugin_CBoeingPackageModel {
    _unused: [u8; 0],
}
#![feature(new_uninit)]
#![feature(maybe_uninit_slice, maybe_uninit_write_slice)]
pub(crate) mod config;
pub(crate) mod connection;
pub(crate) mod constants;
pub(crate) mod enums;
pub(crate) mod packet;
pub(crate) mod cursor;
pub(crate) mod encoding;
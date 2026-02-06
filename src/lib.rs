#![no_std]

pub mod mmio;

#[cfg(all(feature = "eiointc", target_arch = "loongarch64"))]
pub mod eiointc;

#[cfg(feature = "pch-pic")]
pub mod pch_pic;

#[cfg(all(feature = "liointc", target_arch = "loongarch64"))]
pub mod liointc;

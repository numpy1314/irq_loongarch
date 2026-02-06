#![allow(dead_code)]

#[inline(always)]
pub unsafe fn read32(addr: usize) -> u32 {
    (addr as *const u32).read_volatile()
}

#[inline(always)]
pub unsafe fn write32(addr: usize, val: u32) {
    (addr as *mut u32).write_volatile(val)
}

#[inline(always)]
pub unsafe fn write8(addr: usize, val: u8) {
    (addr as *mut u8).write_volatile(val)
}

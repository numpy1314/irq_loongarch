// Ref: Linux irq-loongson-pch-pic.c
use crate::mmio;

const PIC_COUNT_PER_REG: usize = 32;
const PIC_REG_COUNT: usize = 2;

const PCH_PIC_MASK: usize = 0x20;
const PCH_PIC_EDGE: usize = 0x60;
const PCH_PIC_POL: usize = 0x3e0;
const PCH_INT_HTVEC: usize = 0x200;

#[derive(Copy, Clone)]
pub struct PchPic {
    mmio_base: usize, // 这里传入“已经映射到虚拟地址空间的 base”
}

impl PchPic {
    /// # Safety
    /// 调用者必须保证 mmio_base 指向正确的 PCH-PIC MMIO 映射区域，并具备访问权限。
    pub const unsafe fn new(mmio_base: usize) -> Self {
        Self { mmio_base }
    }

    #[inline(always)]
    fn read_w(&self, off: usize) -> u32 {
        unsafe { mmio::read32(self.mmio_base + off) }
    }

    #[inline(always)]
    fn write_w(&self, off: usize, val: u32) {
        unsafe { mmio::write32(self.mmio_base + off, val) }
    }

    pub fn init(&self) {
        // High level triggered
        for _ in 0..PIC_REG_COUNT {
            self.write_w(PCH_PIC_EDGE, 0);
            self.write_w(PCH_PIC_POL, 0);
        }
    }

    fn split_bit(irq: usize) -> (usize, u32) {
        (irq / PIC_COUNT_PER_REG * 4, 1 << (irq % PIC_COUNT_PER_REG))
    }

    pub fn enable_irq(&self, irq: usize) {
        let (offset, bit) = Self::split_bit(irq);

        let addr = PCH_PIC_MASK + offset;
        self.write_w(addr, self.read_w(addr) & !bit);

        // route vector
        let addr = PCH_INT_HTVEC + irq;
        unsafe { mmio::write8(self.mmio_base + addr, irq as u8) };
    }

    pub fn disable_irq(&self, irq: usize) {
        let (offset, bit) = Self::split_bit(irq);
        let addr = PCH_PIC_MASK + offset;
        self.write_w(addr, self.read_w(addr) | bit);
    }
}

use crate::mmio;
use loongArch64::register::*;

const LIOINTC_INTC_CHIP_START: usize = 0x20;
const LIOINTC_REG_INTC_DISABLE: usize = LIOINTC_INTC_CHIP_START + 0x0c;
const LIOINTC_REG_INTC_POL: usize = LIOINTC_INTC_CHIP_START + 0x10;
const LIOINTC_REG_INTC_EDGE: usize = LIOINTC_INTC_CHIP_START + 0x14;
const LIOINTC_REG_BOUNCE: usize = LIOINTC_INTC_CHIP_START + 0x18;
const LIOINTC_REG_INTC_AUTO: usize = LIOINTC_INTC_CHIP_START + 0x1c;

const LIOINTC_SHIFT_INTX: usize = 4;
const VEC_COUNT: usize = 64;

#[derive(Copy, Clone)]
pub struct Liointc {
    mmio_base: usize,
    coreisr_base: usize,
}

impl Liointc {
    pub const unsafe fn new(mmio_base: usize, coreisr_base: usize) -> Self {
        Self {
            mmio_base,
            coreisr_base,
        }
    }

    #[inline(always)]
    fn bank_base(&self, idx: usize) -> usize {
        self.mmio_base + if idx > 31 { 0x40 } else { 0 }
    }

    fn irq_route_register(&self, idx: usize) -> usize {
        self.mmio_base + 0x00 + if idx > 31 { 0x40 } else { 0 } + (idx % 32)
    }

    fn irq_inten_set_register(&self, idx: usize) -> usize {
        self.mmio_base + 0x28 + if idx > 31 { 0x40 } else { 0 }
    }

    fn irq_inten_clr_register(&self, idx: usize) -> usize {
        self.mmio_base + 0x2c + if idx > 31 { 0x40 } else { 0 }
    }

    fn irq_coreisr_register0(&self, core: usize) -> usize {
        self.coreisr_base + core * 0x100
    }

    fn irq_coreisr_register1(&self, core: usize) -> usize {
        self.coreisr_base + 8 + core * 0x100
    }

    pub fn lioint_corex_inty(x: usize, y: usize) -> usize {
        (1 << x) | (1 << (y + LIOINTC_SHIFT_INTX))
    }

    pub fn init(&self) {
        let old_value = ecfg::read().lie();
        let new_value = old_value | ecfg::LineBasedInterrupt::HWI0;
        ecfg::set_lie(new_value);

        for i in 0..VEC_COUNT {
            unsafe {
                mmio::write8(
                    self.irq_route_register(i),
                    Self::lioint_corex_inty(0, 0) as u8,
                )
            };
        }

        // disable all IRQs
        unsafe {
            mmio::write32(self.bank_base(0) + LIOINTC_REG_INTC_DISABLE, 0xffff_ffff);
            mmio::write32(self.bank_base(32) + LIOINTC_REG_INTC_DISABLE, 0xffff_ffff);

            // set all IRQs to low level triggered
            mmio::write32(self.bank_base(0) + LIOINTC_REG_INTC_POL, 0);
            mmio::write32(self.bank_base(32) + LIOINTC_REG_INTC_POL, 0);
            mmio::write32(self.bank_base(0) + LIOINTC_REG_INTC_EDGE, 0);
            mmio::write32(self.bank_base(32) + LIOINTC_REG_INTC_EDGE, 0);

            // set all auto and bounce to 0
            mmio::write32(self.bank_base(0) + LIOINTC_REG_BOUNCE, 0);
            mmio::write32(self.bank_base(32) + LIOINTC_REG_BOUNCE, 0);
            mmio::write32(self.bank_base(0) + LIOINTC_REG_INTC_AUTO, 0);
            mmio::write32(self.bank_base(32) + LIOINTC_REG_INTC_AUTO, 0);
        }
    }

    pub fn enable_irq(&self, irq: usize) {
        let bit = 1u32 << (irq % 32);
        unsafe { mmio::write32(self.irq_inten_set_register(irq), bit) };
    }

    pub fn disable_irq(&self, irq: usize) {
        let bit = 1u32 << (irq % 32);
        unsafe { mmio::write32(self.irq_inten_clr_register(irq), bit) };
    }

    pub fn claim_irq(&self) -> Option<usize> {
        let mut pending: u64 = unsafe { mmio::read32(self.irq_coreisr_register1(0)) as u64 };
        pending = (pending << 32) | unsafe { mmio::read32(self.irq_coreisr_register0(0)) as u64 };

        if pending == 0 {
            return None;
        }

        let irq = pending.trailing_zeros() as usize;

        // clear current irq bit
        self.disable_irq(irq);

        pending &= !(1u64 << irq);
        if pending == 0 && estat::read().is() != 0 {
            let stat_mask = 0x3fc;
            unsafe {
                core::arch::asm!("csrxchg {}, {}, 0x5", in(reg) 0, in(reg) stat_mask);
            }
        }

        Some(irq)
    }

    pub fn complete_irq(&self, irq: usize) {
        if irq < 64 {
            self.enable_irq(irq);
        }
    }
}

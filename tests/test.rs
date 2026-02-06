// tests/test.rs

use irq_loongarch::mmio;

#[test]
fn mmio_read_write_u32_roundtrip() {
    let mut x: u32 = 0;
    let addr = core::ptr::addr_of_mut!(x) as usize;

    unsafe { mmio::write32(addr, 0x1234_5678) };
    let got = unsafe { mmio::read32(addr) };

    assert_eq!(got, 0x1234_5678);
}

#[cfg(feature = "pch-pic")]
mod pch_pic_tests {
    use super::*;
    use irq_loongarch::pch_pic::PchPic;

    const PCH_PIC_MASK: usize = 0x20;
    const PCH_PIC_EDGE: usize = 0x60;
    const PCH_PIC_POL: usize = 0x3e0;
    const PCH_INT_HTVEC: usize = 0x200;

    fn new_mmio_region() -> (Vec<u32>, usize) {
        let words = 0x1000 / 4;
        let mut mem = vec![0u32; words];
        let base = mem.as_mut_ptr() as usize;
        (mem, base)
    }

    #[test]
    fn pch_pic_init_sets_edge_pol_to_zero() {
        let (mut mem, base) = new_mmio_region();

        unsafe {
            mmio::write32(base + PCH_PIC_EDGE, 0xffff_ffff);
            mmio::write32(base + PCH_PIC_POL, 0xffff_ffff);
        }

        let pic = unsafe { PchPic::new(base) };
        pic.init();

        let edge = unsafe { mmio::read32(base + PCH_PIC_EDGE) };
        let pol = unsafe { mmio::read32(base + PCH_PIC_POL) };

        assert_eq!(edge, 0);
        assert_eq!(pol, 0);

        core::hint::black_box(&mut mem);
    }

    #[test]
    fn pch_pic_enable_irq_unmasks_and_sets_htvec() {
        let (mut mem, base) = new_mmio_region();
        let pic = unsafe { PchPic::new(base) };

        let irq: usize = 5;
        let bit: u32 = 1 << (irq % 32);

        unsafe { mmio::write32(base + PCH_PIC_MASK, 0xffff_ffff) };

        pic.enable_irq(irq);

        let mask_after = unsafe { mmio::read32(base + PCH_PIC_MASK) };
        assert_eq!(mask_after, 0xffff_ffff & !bit);

        let hvec_addr = base + PCH_INT_HTVEC + irq;
        let hvec = unsafe { (hvec_addr as *const u8).read_volatile() };
        assert_eq!(hvec, irq as u8);

        core::hint::black_box(&mut mem);
    }

    #[test]
    fn pch_pic_disable_irq_masks_bit() {
        let (mut mem, base) = new_mmio_region();
        let pic = unsafe { PchPic::new(base) };

        let irq: usize = 5;
        let bit: u32 = 1 << (irq % 32);

        unsafe { mmio::write32(base + PCH_PIC_MASK, 0) };
        pic.disable_irq(irq);

        let mask_after = unsafe { mmio::read32(base + PCH_PIC_MASK) };
        assert_eq!(mask_after, bit);

        core::hint::black_box(&mut mem);
    }
}

#[cfg(all(target_arch = "loongarch64", feature = "eiointc"))]
#[test]
fn eiointc_api_smoke_compiles() {
    let _init: fn() = irq_loongarch::eiointc::init;
    let _en: fn(usize) = irq_loongarch::eiointc::enable_irq;
    let _dis: fn(usize) = irq_loongarch::eiointc::disable_irq;
    let _claim: fn() -> Option<usize> = irq_loongarch::eiointc::claim_irq;
    let _complete: fn(usize) = irq_loongarch::eiointc::complete_irq;
}

#[cfg(all(target_arch = "loongarch64", feature = "liointc"))]
#[test]
fn liointc_api_smoke_compiles() {
    use irq_loongarch::liointc::Liointc;

    let _ctor: unsafe fn(usize, usize) -> Liointc = Liointc::new;
    let _init: fn(&Liointc) = Liointc::init;
    let _en: fn(&Liointc, usize) = Liointc::enable_irq;
    let _dis: fn(&Liointc, usize) = Liointc::disable_irq;
    let _claim: fn(&Liointc) -> Option<usize> = Liointc::claim_irq;
    let _complete: fn(&Liointc, usize) = Liointc::complete_irq;
}

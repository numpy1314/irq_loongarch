# irq-loongarch

LoongArch interrupt controller drivers as a standalone `no_std` crate.

This crate is intended for kernels / bare-metal environments, and provides low-level register access for:

- **EIOINTC** (External I/O interrupt controller, via IOCSR)
- **PCH-PIC** (Loongson PCH PIC, MMIO)
- **LIOINTC** (Loongson Local I/O interrupt controller, MMIO)

> Note: This crate is register-level. Address mapping, memory attributes, and platform-specific IRQ topology are the responsibility of the caller.

## Status

- `no_std`
- Works on host (x86_64) for `pch-pic` and basic compilation checks.
- `eiointc` / `liointc` are only compiled on `target_arch = "loongarch64"`.

## Repository

GitHub: `https://github.com/numpy1314/irq_loongarch`

## Features

| Feature     | Description | Notes |
|------------|-------------|------|
| `pch-pic`  | PCH PIC driver (MMIO) | Works on any architecture (MMIO base provided by caller) |
| `eiointc`  | EIOINTC driver (IOCSR) | Requires `target_arch=loongarch64` |
| `liointc`  | LIOINTC driver (MMIO + LoongArch CSRs) | Requires `target_arch=loongarch64` |

### Nightly note

If you enable `eiointc` / `liointc` and depend on the `loongArch64` crate, you may need **nightly Rust** depending on the upstream dependency configuration.

## Usage

Add to your `Cargo.toml`:

### Use from Git (recommended while developing)
```toml
[dependencies]
irq-loongarch = { git = "https://github.com/numpy1314/irq_loongarch", features = ["pch-pic"] }
```

## Safety
This crate uses volatile memory access and (for LoongArch-specific parts) inline assembly / CSR access.
The caller must ensure:

- MMIO base addresses are correct and mapped (if applicable)

- correct memory attributes (Device / Uncached) are used

- concurrent access is synchronized if multiple cores touch the same controller

- IRQ numbers passed in are within the controller's supported range

## API Overview
### EIOINTC (IOCSR)
```rust
#[cfg(all(feature = "eiointc", target_arch = "loongarch64"))]
fn example() {
    irq_loongarch::eiointc::init();
    irq_loongarch::eiointc::enable_irq(32);

    if let Some(irq) = irq_loongarch::eiointc::claim_irq() {
        // handle irq...
        irq_loongarch::eiointc::complete_irq(irq);
    }
}
```

### PCH-PIC (MMIO)
```rust
/// PCH-PIC requires a mapped MMIO base provided by your platform code.
use irq_loongarch::pch_pic::PchPic;

// example: platform should compute a valid mapped base address.
const PCH_PIC_MMIO_BASE: usize = 0xffff_0000_0000_0000;

fn setup_pic() {
    let pic = unsafe { PchPic::new(PCH_PIC_MMIO_BASE) };
    pic.init();
    pic.enable_irq(5);
}
```

### LIOINTC (MMIO + CSR)
```rust
// LIOINTC requires two mapped bases: the LIOINTC register base and the COREISR base.
#[cfg(all(feature = "liointc", target_arch = "loongarch64"))]
fn setup_lio() {
    use irq_loongarch::liointc::Liointc;

    const LIOINTC_MMIO_BASE: usize = 0xffff_ffc0_0000_0000 + 0x1fe0_1400;
    const LIOINTC_COREISR_BASE: usize = 0xffff_ffc0_0000_0000 + 0x1fe0_1040;

    let lio = unsafe { Liointc::new(LIOINTC_MMIO_BASE, LIOINTC_COREISR_BASE) };
    lio.init();

    if let Some(irq) = lio.claim_irq() {
        // handle...
        lio.complete_irq(irq);
    }
}
```
## License

Licensed under either of:

- Apache License, Version 2.0

- MIT license

at your option.
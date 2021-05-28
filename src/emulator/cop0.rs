//! Coprocessor 0: exception and interrupt handling, memory
//! controller settings.
//!
//! This is the first feature I'm implementing after graduating
//! `indy` from `minips-rs`, so I have more free time to try
//! different strategies. 
//!
//! Instead of just hardcoding the coprocessor registers in the
//! CPU struct, like I did with the floating point coprocessor (cop1), 
//! we have everything tidy up here in a separate file. This is
//! also because cop0's registers aren't a simple array of integers,
//! unlike cop1.

use log::debug;

/// Coprocessor 0.
///
/// The register names and descriptions are from the MIPS Vol. 3 manual.
#[derive(Default)]
#[allow(dead_code)]
pub struct Cop0 {
    /// Index into the TLB array.
    /// (n, sel) = (0, 0)
    pub index: u32,
    /// Randomly generated index into the TLB array.
    /// (n, sel) = (1, 0)
    pub random: u32,
    /// Low-order portion of the TLB entry for even-numbered
    /// virtual pages.
    /// (n, sel) = (2, 0)
    pub entry_lo0: u32,
    /// Low-order portion of the TLB entry for odd-numbered
    /// virtual pages.
    /// (n, sel) = (3, 0)
    pub entry_lo1: u32,
    /// Pointer to page table entry in memory.
    /// (n, sel) = (4, 0)
    pub context: u32,
    /// Control for variable page sizes in TLB entries.
    /// (n, sel) = (5, 0)
    pub page_mask: u32,
    /// Controls the number of fixed TLB entries.
    /// (n, sel) = (6, 0)
    pub wired: u32,
    /// Reports the address for the most recent address-related
    /// exception.
    /// (n, sel) = (8, 0)
    pub bad_v_addr: u32,
    /// Processor cycle count.
    /// (n, sel) = (9, 0)
    pub count: u32,
    /// High-order portion of the TLB entry.
    /// (n, sel) = (10, 0)
    pub entry_hi: u32,
    /// Timer interrupt control.
    /// (n, sel) = (11, 0)
    pub compare: u32,
    /// Processor status and control.
    /// (n, sel) = (12, 0)
    pub status: u32,
    /// Couse of last general exception.
    /// (n, sel) = (13, 0)
    pub cause: u32,
    /// Program counter at last exception.
    /// (n, sel) = (14, 0)
    pub epc: u32,
    /// Processor identification and revision.
    /// (n, sel) = (15, 0)
    pub prid: u32,
    /// Configuration register 0.
    /// (n, sel) = (16, 0)
    pub config0: u32,
    /// Configuration register 1.
    /// (n, sel) = (16, 1)
    pub config1: u32,
    /// Configuration register 2.
    /// (n, sel) = (16, 2)
    pub config2: u32,
    /// Configuration register 3.
    /// (n, sel) = (16, 3)
    pub config3: u32,
    /// Load linked address.
    /// (n, sel) = (17, 0)
    pub ll_addr: u32,
    /// Watchpoint address.
    /// (n, sel) = (18, 0)
    pub watch_lo: u32,
    /// Watchpoint control.
    /// (n, sel) = (19, 0)
    pub watch_hi: u32,
    /// EJTAG Debug register.
    /// (n, sel) = (23, 0)
    pub debug: u32,
    /// Program counter at last EJTAG debug exception.
    /// (n, sel) = (24, 0)
    pub depc: u32,
    /// Performance counter interface.
    /// (n, sel) = (25, 0)
    pub perf_cnt: u32,
    /// Parity/ECC error control and status.
    /// (n, sel) = (26, 0)
    pub err_ctl: u32,
    /// Cache parity error control and status.
    /// (n, sel) = (27, 0)
    pub cache_err: u32,
    /// Low-order portion of cache tag interface.
    /// (n, sel) = (28, 0)
    pub tag_lo: u32,
    /// Low-order portion of cache data interface.
    /// (n, sel) = (28, 1)
    pub data_lo: u32,
    /// High-order portion of cache tag interface.
    /// (n, sel) = (29, 0)
    pub tag_hi: u32,
    /// High-order portion of cache data interface.
    /// (n, sel) = (29, 1)
    pub data_hi: u32,
    /// Program counter at last error.
    /// (n, sel) = (30, 0)
    pub error_epc: u32,
    /// EJTAG debug exception save register.
    /// (n, sel) = (31, 0)
    pub desave: u32,
}

impl Cop0 {
    pub fn read_reg(&self, n: u32, sel: u32) -> u32 {
        debug!("read reg {} sel {}", n, sel);

        match (n, sel) {
            (0, 0) => self.index,
            (1, 0) => self.random,
            (2, 0) => self.entry_lo0,
            (3, 0) => self.entry_lo1,
            (4, 0) => self.context,
            (5, 0) => self.page_mask,
            (6, 0) => self.wired,
            (7, _) => unimplemented!(),
            (8, 0) => self.bad_v_addr,
            (9, 0) => self.count,
            (9, 6) | (9, 7) => unimplemented!(),
            (10, 0) => self.entry_hi,
            (11, 0) => self.compare,
            (11, 6) | (11, 7) => unimplemented!(),
            (12, 0) => self.status,
            (13, 0) => self.cause,
            (14, 0) => self.epc,
            (15, 0) => self.prid,
            (16, 0) => self.config0,
            (16, 1) => self.config1,
            (16, 2) => self.config2,
            (16, 3) => self.config3,
            (17, 0) => self.ll_addr,
            (18, _) => self.watch_lo,
            (19, _) => self.watch_hi,
            (20, 0) => unimplemented!(),
            (21, _) => unimplemented!(),
            (22, _) => unimplemented!(),
            (23, 0) => self.debug,
            (24, 0) => self.depc,
            (25, _) => self.perf_cnt,
            (26, 0) => self.err_ctl,
            (27, _) => self.cache_err,
            (28, 0) => self.tag_lo,
            (28, 1) => self.data_lo,
            (29, 0) => self.tag_hi,
            (29, 1) => self.data_hi,
            (30, 0) => self.error_epc,
            (31, 0) => self.desave,
            _ => unreachable!()
        }
    }

    pub fn write_reg(&mut self, n: u32, sel: u32, val: u32) {
        debug!("write {} to reg {} sel {}", val, n, sel);

        match (n, sel) {
            (0, 0) => self.index = val,
            (1, 0) => self.random = val,
            (2, 0) => self.entry_lo0 = val,
            (3, 0) => self.entry_lo1 = val,
            (4, 0) => self.context = val,
            (5, 0) => self.page_mask = val,
            (6, 0) => self.wired = val,
            (7, _) => unimplemented!(),
            (8, 0) => self.bad_v_addr = val,
            (9, 0) => self.count = val,
            (9, 6) | (9, 7) => unimplemented!(),
            (10, 0) => self.entry_hi = val,
            (11, 0) => self.compare = val,
            (11, 6) | (11, 7) => unimplemented!(),
            (12, 0) => self.status = val,
            (13, 0) => self.cause = val,
            (14, 0) => self.epc = val,
            (15, 0) => self.prid = val,
            (16, 0) => self.config0 = val,
            (16, 1) => self.config1 = val,
            (16, 2) => self.config2 = val,
            (16, 3) => self.config3 = val,
            (17, 0) => self.ll_addr = val,
            (18, _) => self.watch_lo = val,
            (19, _) => self.watch_hi = val,
            (20, 0) => unimplemented!(),
            (21, _) => unimplemented!(),
            (22, _) => unimplemented!(),
            (23, 0) => self.debug = val,
            (24, 0) => self.depc = val,
            (25, _) => self.perf_cnt = val,
            (26, 0) => self.err_ctl = val,
            (27, _) => self.cache_err = val,
            (28, 0) => self.tag_lo = val,
            (28, 1) => self.data_lo = val,
            (29, 0) => self.tag_hi = val,
            (29, 1) => self.data_hi = val,
            (30, 0) => self.error_epc = val,
            (31, 0) => self.desave = val,
            _ => unreachable!()
        }
    }
}

/// pub fn get_args(regs: &[u8]) -> &[u32];
/// pub fn get_return_address(regs: &[u8]);
/// pub fn get_return_value(regs: &[u8]) -> &[u32];


#[cfg(feature = "mips")]
mod mipsel32;

#[cfg(feature = "mips")]
pub use crate::arch::mipsel32::{
    get_args,
    get_return_address,
    get_return_value,
    get_stack_pointer,
};

#[cfg(feature = "armv5tel")]
mod armv5tel;

#[cfg(feature = "armv5tel")]
pub use crate::arch::armv5tel::{
    get_args,
    get_return_address,
    get_return_value,
    get_stack_pointer,
};

#[cfg(feature = "aarch64")]
compile_error!("Arch aarch64 not currently supported");
#[cfg(feature = "aarch64")]
compile_error!("Arch aarch64 not currently supported");
#[cfg(feature = "aarch64be")]
compile_error!("Arch aarch64be not currently supported");
#[cfg(feature = "alpha")]
compile_error!("Arch alpha not currently supported");
#[cfg(feature = "armv5teb")]
compile_error!("Arch armv5teb not currently supported");
#[cfg(feature = "cris")]
compile_error!("Arch cris not currently supported");
#[cfg(feature = "hexagon")]
compile_error!("Arch hexagon not currently supported");
#[cfg(feature = "i386")]
compile_error!("Arch i386 not currently supported");
#[cfg(feature = "i686")]
compile_error!("Arch i686 not currently supported");
#[cfg(feature = "m68k")]
compile_error!("Arch m68k not currently supported");
#[cfg(feature = "microblaze")]
compile_error!("Arch microblaze not currently supported");
#[cfg(feature = "mips64")]
compile_error!("Arch mips64 not currently supported");
#[cfg(feature = "nios2")]
compile_error!("Arch nios2 not currently supported");
#[cfg(feature = "openrisc")]
compile_error!("Arch openrisc not currently supported");
#[cfg(feature = "parisc")]
compile_error!("Arch parisc not currently supported");
#[cfg(feature = "ppc")]
compile_error!("Arch ppc not currently supported");
#[cfg(feature = "ppc64")]
compile_error!("Arch ppc64 not currently supported");
#[cfg(feature = "ppc64le")]
compile_error!("Arch ppc64le not currently supported");
#[cfg(feature = "riscv32")]
compile_error!("Arch riscv32 not currently supported");
#[cfg(feature = "riscv64")]
compile_error!("Arch riscv64 not currently supported");
#[cfg(feature = "s390x")]
compile_error!("Arch s390x not currently supported");
#[cfg(feature = "sh4")]
compile_error!("Arch sh4 not currently supported");
#[cfg(feature = "sparc")]
compile_error!("Arch sparc not currently supported");
#[cfg(feature = "sparc64")]
compile_error!("Arch sparc64 not currently supported");
#[cfg(feature = "x86_64")]
compile_error!("Arch x86_64 not currently supported");
#[cfg(feature = "xtensa")]
compile_error!("Arch xtensa not currently supported");

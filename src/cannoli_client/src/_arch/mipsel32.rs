use std::fmt;

const BITWIDTH: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum MipsReg {
    ZERO = 0, AT, V0, V1,
    A0, A1, A2, A3,
    T0, T1, T2, T3,
    T4, T5, T6, T7,
    S0, S1, S2, S3,
    S4, S5, S6, S7,
    T8, T9, K0, K1, 
    GP, SP, FP, RA,
}

impl Into<usize> for MipsReg {
    fn into(self) -> usize {
        self as usize
    }
}

impl From<usize> for MipsReg {
    fn from(val: usize) -> Self {
        match val {
            0  => Self::ZERO,                                  
            1  => Self::AT,                                    
            2  => Self::V0,                                    
            3  => Self::V1,                                    
            4  => Self::A0,                                    
            5  => Self::A1,                                    
            6  => Self::A2,                                    
            7  => Self::A3,                                    
            8  => Self::T0,                                    
            9  => Self::T1,                                    
            10 => Self::T2,                                    
            11 => Self::T3,                                    
            12 => Self::T4,                                    
            13 => Self::T5,                                    
            14 => Self::T6,                                    
            15 => Self::T7,                                    
            16 => Self::S0,                                    
            17 => Self::S1,                                    
            18 => Self::S2,                                    
            19 => Self::S3,                                    
            20 => Self::S4,                                    
            21 => Self::S5,                                    
            22 => Self::S6,                                    
            23 => Self::S7,                                    
            24 => Self::T8,                                    
            25 => Self::T9,                                    
            26 => Self::K0,                                    
            27 => Self::K1,                                    
            28 => Self::GP,                                    
            29 => Self::SP,                                    
            30 => Self::FP,                                    
            31 => Self::RA,                                    
            _ => panic!("Invalid MIPS register index {}", val),
        }                                                      
    }                                                          
}

#[inline]                                                  
fn get_mips_reg_le(regs: &[u8], register: MipsReg) -> u32 {
    let i = register as usize * BITWIDTH;                  
    // Get a slice at the register's index -> u32          
    u32::from_le_bytes(regs[i..i+4].try_into().unwrap())   
}

pub fn get_return_addr_le(regs: &[u8]) -> u64 {
    get_mips_reg_le(regs, MipsReg::RA) as u64
} 

pub fn testfunc() {
    println!("Hello world!");
}

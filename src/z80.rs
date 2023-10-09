#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::*;

type read_byte = fn(userdata: &mut userdata, addr: u16) -> u8;
type write_byte = fn(userdata: &mut userdata, addr: u16, val: u8);
type port_in = fn(z: &mut z80, port: u8) -> u8;
type port_out = fn(z: &mut z80, port: u8, val: u8);

pub fn read_byte_null(_userdata: &mut userdata, _addr: u16) -> u8 {
    return 0;
}

pub fn write_byte_null(_userdata: &mut userdata, _addr: u16, _val: u8) {}

pub fn port_in_null(_z: &mut z80, _port: u8) -> u8 {
    return 0;
}

pub fn port_out_null(_z: &mut z80, _port: u8, _val: u8) {}

pub struct z80<'a> {
    pub read_byte: read_byte,
    pub write_byte: write_byte,
    pub port_in: port_in,
    pub port_out: port_out,
    pub userdata: userdata<'a>,

    // cycle count (t-states)
    pub cyc: u64,

    // special purpose registers
    pub pc: u16,
    pub sp: u16,
    pub ix: u16,
    pub iy: u16,
    // "wz" register
    pub mem_ptr: u16,
    // main registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    // alternate registers
    pub a_: u8,
    pub b_: u8,
    pub c_: u8,
    pub d_: u8,
    pub e_: u8,
    pub h_: u8,
    pub l_: u8,
    pub f_: u8,
    // interrupt vector, memory refresh
    pub i: u8,
    pub r: u8,

    // flags: sign, zero, yf, half-carry, xf, parity/overflow, negative, carry
    pub sf: u8,
    pub zf: u8,
    pub yf: u8,
    pub hf: u8,
    pub xf: u8,
    pub pf: u8,
    pub nf: u8,
    pub cf: u8,

    pub iff_delay: u8,
    pub interrupt_mode: u8,
    pub int_data: u8,
    pub iff1: bool,
    pub iff2: bool,
    pub halted: bool,
    pub int_pending: bool,
    pub nmi_pending: bool,
}

impl<'a> z80<'a> {
    pub fn new() -> Self {
        Self {
            read_byte: read_byte_null,
            write_byte: write_byte_null,
            port_in: port_in_null,
            port_out: port_out_null,

            userdata: userdata::new(),
            // cycle count (t-states)
            cyc: 0,

            // special purpose registers
            pc: 0,
            sp: 0,
            ix: 0,
            iy: 0,
            // "wz" register
            mem_ptr: 0,
            // main registers
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            // alternate registers
            a_: 0,
            b_: 0,
            c_: 0,
            d_: 0,
            e_: 0,
            h_: 0,
            l_: 0,
            f_: 0,
            // interrupt vector, memory refresh
            i: 0,
            r: 0,

            // flags: sign, zero, yf, half-carry, xf, parity/overflow, negative, carry
            sf: 0,
            zf: 0,
            yf: 0,
            hf: 0,
            xf: 0,
            pf: 0,
            nf: 0,
            cf: 0,

            iff_delay: 0,
            interrupt_mode: 0,
            int_data: 0,
            iff1: false,
            iff2: false,
            halted: false,
            int_pending: false,
            nmi_pending: false,
        }
    }
}

// MARK: timings
pub const cyc_00: [u8; 256] = [
    4, 10, 7, 6, 4, 4, 7, 4, 4, 11, 7, 6, 4, 4, 7, 4, 8, 10, 7, 6, 4, 4, 7, 4, 12, 11, 7, 6, 4, 4,
    7, 4, 7, 10, 16, 6, 4, 4, 7, 4, 7, 11, 16, 6, 4, 4, 7, 4, 7, 10, 13, 6, 11, 11, 10, 4, 7, 11,
    13, 6, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4,
    4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 7, 7, 7, 7, 7, 7, 4, 7, 4,
    4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4,
    4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4,
    4, 4, 4, 4, 4, 7, 4, 5, 10, 10, 10, 10, 11, 7, 11, 5, 10, 10, 0, 10, 17, 7, 11, 5, 10, 10, 11,
    10, 11, 7, 11, 5, 4, 10, 11, 10, 0, 7, 11, 5, 10, 10, 19, 10, 11, 7, 11, 5, 4, 10, 4, 10, 0, 7,
    11, 5, 10, 10, 4, 10, 11, 7, 11, 5, 6, 10, 4, 10, 0, 7, 11,
];

pub const cyc_ed: [u8; 256] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    12, 12, 15, 20, 8, 14, 8, 9, 12, 12, 15, 20, 8, 14, 8, 9, 12, 12, 15, 20, 8, 14, 8, 9, 12, 12,
    15, 20, 8, 14, 8, 9, 12, 12, 15, 20, 8, 14, 8, 18, 12, 12, 15, 20, 8, 14, 8, 18, 12, 12, 15,
    20, 8, 14, 8, 8, 12, 12, 15, 20, 8, 14, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 8,
    8, 8, 8, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];

pub const cyc_ddfd: [u8; 256] = [
    4, 4, 4, 4, 4, 4, 4, 4, 4, 15, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 15, 4, 4, 4, 4, 4,
    4, 4, 14, 20, 10, 8, 8, 11, 4, 4, 15, 20, 10, 8, 8, 11, 4, 4, 4, 4, 4, 23, 23, 19, 4, 4, 15, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4,
    4, 4, 8, 8, 19, 4, 8, 8, 8, 8, 8, 8, 19, 8, 8, 8, 8, 8, 8, 8, 19, 8, 19, 19, 19, 19, 19, 19, 4,
    19, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8,
    8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4,
    4, 8, 8, 19, 4, 4, 4, 4, 4, 8, 8, 19, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 14, 4, 23, 4, 15, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 10, 4, 4, 4, 4, 4, 4,
];

// MARK: helpers

// get bit "n" of number "val"
pub fn GET_BIT(n: u8, val: u8) -> u8 {
    //println!("GET_BIT");

    return (val >> n) & 1;
}

pub fn rb(z: &mut z80, addr: u16) -> u8 {
    //println!("rb-z80");

    let result = (z.read_byte)(&mut z.userdata, addr);

    return result;
}

pub fn wb(z: &mut z80, addr: u16, val: u8) {
    //println!("wb-z80");

    (z.write_byte)(&mut z.userdata, addr, val);
}

pub fn rw(z: &mut z80, addr: u16) -> u16 {
    //println!("rw");

    let value1 = ((z.read_byte)(&mut z.userdata, addr + 1) as u16) << 8;
    let value2 = (z.read_byte)(&mut z.userdata, addr) as u16;
    let result = value1 | value2;

    return result as u16;
}

pub fn ww(z: &mut z80, addr: u16, val: u16) {
    //println!("ww");

    (z.write_byte)(&mut z.userdata, addr, (val & 0xFF) as u8);
    (z.write_byte)(&mut z.userdata, addr + 1, (val >> 8) as u8);
}

pub fn pushw(z: &mut z80, val: u16) {
    //println!("pushw");

    z.sp -= 2;
    ww(z, z.sp, val);
}

pub fn popw(z: &mut z80) -> u16 {
    //println!("popw");

    z.sp += 2;
    return rw(z, z.sp - 2);
}

pub fn nextb(z: &mut z80) -> u8 {
    //println!("nextb");

    let temp = rb(z, z.pc);
    z.pc += 1;
    return temp;
}

pub fn nextw(z: &mut z80) -> u16 {
    //println!("nextw");

    z.pc += 2;
    return rw(z, z.pc - 2);
}

pub fn get_bc(z: &mut z80) -> u16 {
    //println!("get_bc");

    return ((z.b as u16) << 8) | (z.c as u16);
}

pub fn get_de(z: &mut z80) -> u16 {
    //println!("get_de");

    return ((z.d as u16) << 8) | (z.e as u16);
}

pub fn get_hl(z: &mut z80) -> u16 {
    //println!("get_hl");

    return ((z.h as u16) << 8) | (z.l as u16);
}

pub fn set_bc(z: &mut z80, val: u16) {
    //println!("set_bc");

    z.b = (val >> 8) as u8;
    z.c = (val & 0xFF) as u8;
}

pub fn set_de(z: &mut z80, val: u16) {
    //println!("set_de");

    z.d = (val >> 8) as u8;
    z.e = (val & 0xFF) as u8;
}

pub fn set_hl(z: &mut z80, val: u16) {
    //println!("set_hl");

    z.h = (val >> 8) as u8;
    z.l = (val & 0xFF) as u8;
}

pub fn get_f(z: &mut z80) -> u8 {
    //println!("get_f");

    let mut val: u8 = 0;
    val |= z.cf << 0;
    val |= z.nf << 1;
    val |= z.pf << 2;
    val |= z.xf << 3;
    val |= z.hf << 4;
    val |= z.yf << 5;
    val |= z.zf << 6;
    val |= z.sf << 7;

    return val;
}

pub fn set_f(z: &mut z80, val: u8) {
    //println!("set_f");

    z.cf = (val >> 0) & 1;
    z.nf = (val >> 1) & 1;
    z.pf = (val >> 2) & 1;
    z.xf = (val >> 3) & 1;
    z.hf = (val >> 4) & 1;
    z.yf = (val >> 5) & 1;
    z.zf = (val >> 6) & 1;
    z.sf = (val >> 7) & 1;
}

// increments R, keeping the highest byte intact
pub fn inc_r(z: &mut z80) {
    //println!("inc_r");

    z.r = (z.r & 0x80) | ((z.r + 1) & 0x7f);
}

// returns if there was a carry between bit "bit_no" and "bit_no - 1" when
// executing "a + b + cy"
pub fn carry(bit_no: i32, a: u16, b: u16, cy: u16) -> u8 {
    //println!("carry");

    let cy_ = cy & 0x01;

    let result: i32 = (a + b + cy_) as i32;
    let carry: i32 = result ^ a as i32 ^ b as i32;
    let result = carry & (1 << bit_no);
    if result > 0 {
        return 1;
    } else {
        return 0;
    }
}

// returns the parity of byte: 0 if number of 1 bits in `val` is odd, else 1
pub fn parity(val: u8) -> bool {
    //println!("parity");

    let mut nb_one_bits: u8 = 0;
    for i in 0..8 {
        nb_one_bits += (val >> i) & 1;
    }

    return (nb_one_bits & 1) == 0;
}

// MARK: opcodes
// jumps to an address
pub fn jump(z: &mut z80, addr: u16) {
    //println!("jump");

    z.pc = addr;
    z.mem_ptr = addr;
}

// jumps to next word in memory if condition is true
pub fn cond_jump(z: &mut z80, condition: bool) {
    //println!("cond_jump");

    let addr: u16 = nextw(z);
    if condition {
        jump(z, addr);
    }
    z.mem_ptr = addr;
}

// calls to next word in memory
pub fn call(z: &mut z80, addr: u16) {
    //println!("call");

    pushw(z, z.pc);
    z.pc = addr;
    z.mem_ptr = addr;
}

// calls to next word in memory if condition is true
pub fn cond_call(z: &mut z80, condition: bool) {
    //println!("cond_call");

    let addr: u16 = nextw(z);
    if condition {
        call(z, addr);
        z.cyc += 7;
    }
    z.mem_ptr = addr;
}

// returns from subroutine
pub fn ret(z: &mut z80) {
    //println!("ret");

    z.pc = popw(z);
    z.mem_ptr = z.pc;
}

// returns from subroutine if condition is true
pub fn cond_ret(z: &mut z80, condition: bool) {
    //println!("cond_ret");

    if condition {
        ret(z);
        z.cyc += 6;
    }
}

pub fn jr(z: &mut z80, displacement: i8) {
    //println!("jr");

    z.pc = z.pc.wrapping_add(displacement as u16);

    z.mem_ptr = z.pc;
}

pub fn cond_jr(z: &mut z80, condition: bool) {
    //println!("cond_jr");

    let b: i8 = nextb(z) as i8;
    if condition {
        jr(z, b);
        z.cyc += 5;
    }
}

// ADD Byte: adds two bytes together
pub fn addb(z: &mut z80, a: u8, b: u8, cy: u8) -> u8 {
    //println!("addb");

    let cy_ = cy & 0x01;

    let result: u8 = (a as u16 + b as u16 + cy_ as u16) as u8;

    z.sf = result >> 7;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.hf = carry(4, a as u16, b as u16, cy_ as u16);
    if z.hf > 0 {
        z.hf = 1;
    } else {
        z.hf = 0;
    }
    if carry(7, a as u16, b as u16, cy_ as u16) != carry(8, a as u16, b as u16, cy_ as u16) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.cf = carry(8, a as u16, b as u16, cy_ as u16);
    if z.cf > 0 {
        z.cf = 1;
    } else {
        z.cf = 0;
    }
    z.nf = 0;
    z.xf = GET_BIT(3, result);
    if z.xf > 0 {
        z.xf = 1;
    } else {
        z.xf = 0;
    }
    z.yf = GET_BIT(5, result);
    if z.yf > 0 {
        z.yf = 1;
    } else {
        z.yf = 0;
    }

    return result;
}

// SUBstract Byte: substracts two bytes (with optional carry)
pub fn subb(z: &mut z80, a: u8, b: u8, cy: u8) -> u8 {
    //println!("subb");

    let cy__ = !cy & 0x01;

    let val: u8 = addb(z, a, !b, cy__);
    z.cf = !z.cf & 0x1;
    z.hf = !z.hf & 0x1;
    z.nf = 1;
    return val;
}

// ADD Word: adds two words together
pub fn addw(z: &mut z80, a: u16, b: u16, cy: u8) -> u16 {
    //println!("addw");

    let cy_ = cy & 0x01;

    let lsb: u8 = addb(z, a as u8, b as u8, cy_);
    let msb: u8 = addb(z, (a >> 8) as u8, (b >> 8) as u8, z.cf);

    let result: u16 = ((msb as u16) << 8) | lsb as u16;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.mem_ptr = a + 1;
    return result;
}

// SUBstract Word: substracts two words (with optional carry)
pub fn subw(z: &mut z80, a: u16, b: u16, cy: u8) -> u16 {
    //println!("subw");

    let cy_ = cy & 0x01;

    let lsb: u8 = subb(z, a as u8, b as u8, cy_);
    let msb: u8 = subb(z, (a >> 8) as u8, (b >> 8) as u8, z.cf);

    let result: u16 = ((msb as u16) << 8) | lsb as u16;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.mem_ptr = a + 1;
    return result;
}

// adds a word to HL
pub fn addhl(z: &mut z80, val: u16) {
    //println!("addhl");

    let sf: u8 = z.sf;
    let zf: u8 = z.zf;
    let pf: u8 = z.pf;
    let result: u16 = get_hl(z);
    let result: u16 = addw(z, result, val, 0);
    set_hl(z, result);
    z.sf = sf;
    z.zf = zf;
    z.pf = pf;
}

// adds a word to IX or IY
pub fn addiz(z: &mut z80, reg: &mut u16, val: u16) {
    //println!("addiz");

    let sf: u8 = z.sf;
    let zf: u8 = z.zf;
    let pf: u8 = z.pf;
    let result: u16 = addw(z, *reg, val, 0);
    *reg = result;
    z.sf = sf;
    z.zf = zf;
    z.pf = pf;
}

// adds a word (+ carry) to HL
pub fn adchl(z: &mut z80, val: u16) {
    //println!("adchl");

    let result: u16 = get_hl(z);
    let result: u16 = addw(z, result, val, z.cf);
    z.sf = (result >> 15) as u8;
    z.zf = (result == 0) as u8;
    set_hl(z, result);
}

// substracts a word (+ carry) to HL
pub fn sbchl(z: &mut z80, val: u16) {
    //println!("sbchl");

    let result: u16 = get_hl(z);
    let result: u16 = subw(z, result, val, z.cf);
    z.sf = (result >> 15) as u8;
    z.zf = (result == 0) as u8;
    set_hl(z, result);
}

// increments a byte value
pub fn inc(z: &mut z80, a: u8) -> u8 {
    //println!("inc");

    let cf: u8 = z.cf;
    let result: u8 = addb(z, a, 1, 0);
    z.cf = cf;
    return result;
}

// decrements a byte value
pub fn dec(z: &mut z80, a: u8) -> u8 {
    //println!("dec");

    let cf: u8 = z.cf;
    let result: u8 = subb(z, a, 1, 0);
    z.cf = cf;
    return result;
}

// MARK: bitwise

// executes a logic "and" between register A and a byte, then stores the
// result in register A
pub fn land(z: &mut z80, val: u8) {
    //println!("land");

    let result: u16 = (z.a & val) as u16;
    z.sf = (result >> 7) as u8;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.hf = 1;
    if parity(result as u8) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }

    z.nf = 0;
    z.cf = 0;
    z.xf = GET_BIT(3, result as u8);
    z.yf = GET_BIT(5, result as u8);
    z.a = result as u8;
}

// executes a logic "xor" between register A and a byte, then stores the
// result in register A
pub fn lxor(z: &mut z80, val: u8) {
    //println!("lxor");

    let result: u8 = z.a ^ val;
    z.sf = result >> 7;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.hf = 0;
    if parity(result as u8) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.nf = 0;
    z.cf = 0;
    z.xf = GET_BIT(3, result);
    z.yf = GET_BIT(5, result);
    z.a = result;
}

// executes a logic "or" between register A and a byte, then stores the
// result in register A
pub fn lor(z: &mut z80, val: u8) {
    //println!("lor");

    let result: u8 = z.a | val;
    z.sf = result >> 7;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.hf = 0;
    if parity(result as u8) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.nf = 0;
    z.cf = 0;
    z.xf = GET_BIT(3, result);
    z.yf = GET_BIT(5, result);
    z.a = result;
}

// compares a value with register A
pub fn cp(z: &mut z80, val: u8) {
    //println!("cp");

    subb(z, z.a, val, 0);

    // the only difference between cp and sub is that
    // the xf/yf are taken from the value to be substracted,
    // not the result
    z.yf = GET_BIT(5, val);
    z.xf = GET_BIT(3, val);
}

// 0xCB opcodes
// rotate left with carry
pub fn cb_rlc(z: &mut z80, val: u8) -> u8 {
    //println!("cb_rlc");

    let old: u8 = val >> 7;
    let val = (val << 1) | old;
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    z.cf = old;
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// rotate right with carry
pub fn cb_rrc(z: &mut z80, val: u8) -> u8 {
    //println!("cb_rrc");

    let old: u8 = val & 1;
    let val = (val >> 1) | (old << 7);
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    z.cf = old;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// rotate left (simple)
pub fn cb_rl(z: &mut z80, val: u8) -> u8 {
    //println!("cb_rl");

    let cf: u8 = z.cf;
    z.cf = val >> 7;
    let val = (val << 1) | cf;
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// rotate right (simple)
pub fn cb_rr(z: &mut z80, val: u8) -> u8 {
    //println!("cb_rr");

    let c: u8 = z.cf;
    z.cf = val & 1;
    let val = (val >> 1) | (c << 7);
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// shift left preserving sign
pub fn cb_sla(z: &mut z80, val: u8) -> u8 {
    //println!("cb_sla");

    z.cf = val >> 7;
    let val = val << 1;
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// SLL (exactly like SLA, but sets the first bit to 1)
pub fn cb_sll(z: &mut z80, val: u8) -> u8 {
    //println!("cb_sll");

    z.cf = val >> 7;
    let mut val = val << 1;
    val |= 1;
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// shift right preserving sign
pub fn cb_sra(z: &mut z80, val: u8) -> u8 {
    //println!("cb_sra");

    z.cf = val & 1;
    let val = (val >> 1) | (val & 0x80); // 0b10000000
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }
    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// shift register right
pub fn cb_srl(z: &mut z80, val: u8) -> u8 {
    //println!("cb_srl");

    z.cf = val & 1;
    let val = val >> 1;
    z.sf = val >> 7;
    if val == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.nf = 0;
    z.hf = 0;
    if parity(val) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
    z.xf = GET_BIT(3, val);
    z.yf = GET_BIT(5, val);
    return val;
}

// tests bit "n" from a byte
pub fn cb_bit(z: &mut z80, val: u8, n: u8) -> u8 {
    //println!("cb_bit");

    let result: u8 = val & (1 << n);
    z.sf = result >> 7;
    if result == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.yf = GET_BIT(5, val);
    z.hf = 1;
    z.xf = GET_BIT(3, val);
    z.pf = z.zf;
    z.nf = 0;
    return result;
}

pub fn ldi(z: &mut z80) {
    //println!("ldi");

    let de: u16 = get_de(z);
    let hl: u16 = get_hl(z);
    let val: u8 = rb(z, hl);

    wb(z, de, val);

    let result = get_hl(z);
    set_hl(z, result + 1);
    let result = get_de(z);
    set_de(z, result + 1);
    let result = get_bc(z);
    set_bc(z, result - 1);

    // see https://wikiti.brandonw.net/index.php?title=Z80_Instruction_Set
    // for the calculation of xf/yf on LDI
    let result: u16 = val.wrapping_add(z.a) as u16;
    z.xf = GET_BIT(3, result as u8);
    z.yf = GET_BIT(1, result as u8);

    z.nf = 0;
    z.hf = 0;
    if get_bc(z) > 0 {
        z.pf = 1;
    } else {
        z.pf = 0;
    }
}

pub fn ldd(z: &mut z80) {
    //println!("ldd");

    ldi(z);
    // same as ldi but HL and DE are decremented instead of incremented
    let result = get_hl(z);
    set_hl(z, result - 2);
    let result = get_de(z);
    set_de(z, result - 2);
}

pub fn cpi(z: &mut z80) {
    //println!("cpi");

    let cf: u8 = z.cf;
    let result = get_hl(z);
    let result = rb(z, result);
    let result: u16 = subb(z, z.a, result, 0) as u16;
    let temp = get_hl(z);
    set_hl(z, temp + 1);
    let temp = get_bc(z);
    set_bc(z, temp - 1);
    z.xf = GET_BIT(3, result as u8 - z.hf);
    z.yf = GET_BIT(1, result as u8 - z.hf);
    if get_bc(z) != 0 {
        z.pf = 1;
    } else {
        z.pf = 0;
    }

    z.cf = cf;
    z.mem_ptr += 1;
}

pub fn cpd(z: &mut z80) {
    //println!("cpd");

    cpi(z);
    // same as cpi but HL is decremented instead of incremented
    let result = get_hl(z);
    set_hl(z, result - 2);
    z.mem_ptr -= 2;
}

pub fn in_r_c(z: &mut z80, r: &mut u8) {
    //println!("in_r_c");

    *r = (z.port_in)(z, z.c);
    if *r == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.sf = *r >> 7;
    if parity(*r) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }

    z.nf = 0;
    z.hf = 0;
}

pub fn ini(z: &mut z80) {
    //println!("ini");

    let val: u8 = (z.port_in)(z, z.c);
    let result = get_hl(z);
    wb(z, result, val);
    let result = get_hl(z);
    set_hl(z, result + 1);
    z.b -= 1;
    if z.b == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.nf = 1;
    z.mem_ptr = get_bc(z) + 1;
}

pub fn ind(z: &mut z80) {
    //println!("ind");

    ini(z);
    let temp = get_hl(z);
    set_hl(z, temp - 2);
    z.mem_ptr = get_bc(z) - 2;
}

pub fn outi(z: &mut z80) {
    //println!("outi");

    let temp = get_hl(z);
    let temp = rb(z, temp);
    (z.port_out)(z, z.c, temp);
    let temp = get_hl(z) + 1;
    set_hl(z, temp);
    z.b -= 1;
    if z.b == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    z.nf = 1;
    z.mem_ptr = get_bc(z) + 1;
}

pub fn outd(z: &mut z80) {
    //println!("outd");

    outi(z);
    let temp = get_hl(z);
    set_hl(z, temp - 2);
    z.mem_ptr = get_bc(z) - 2;
}

pub fn daa(z: &mut z80) {
    //println!("daa");

    // "When this instruction is executed, the A register is BCD corrected
    // using the  contents of the flags. The exact process is the following:
    // if the least significant four bits of A contain a non-BCD digit
    // (i. e. it is greater than 9) or the H flag is set, then $06 is
    // added to the register. Then the four most significant bits are
    // checked. If this more significant digit also happens to be greater
    // than 9 or the C flag is set, then $60 is added."
    // > http://z80-heaven.wikidot.com/instructions-set:daa
    let mut correction: u8 = 0;

    if (z.a & 0x0F) > 0x09 || z.hf != 0 {
        correction += 0x06;
    }

    if z.a > 0x99 || z.cf != 0 {
        correction += 0x60;
        z.cf = 1;
    }

    let substraction: u8 = z.nf;
    if substraction != 0 {
        if z.hf != 0 && (z.a & 0x0F) < 0x06 {
            z.hf = 1;
        } else {
            z.hf = 0;
        }

        z.a = z.a.wrapping_sub(correction);
    } else {
        if (z.a & 0x0F) > 0x09 {
            z.hf = 1;
        } else {
            z.hf = 0;
        }

        z.a = z.a.wrapping_add(correction);
    }

    z.sf = z.a >> 7;
    if z.a == 0 {
        z.zf = 1;
    } else {
        z.zf = 0;
    }

    if parity(z.a) {
        z.pf = 1;
    } else {
        z.pf = 0;
    }

    z.xf = GET_BIT(3, z.a);
    z.yf = GET_BIT(5, z.a);
}

pub fn displace(z: &mut z80, base_addr: u16, displacement: i8) -> u16 {
    //println!("displace");

    let addr: u16 = base_addr.wrapping_add(displacement as u16);
    z.mem_ptr = addr;
    return addr;
}

pub fn process_interrupts(z: &mut z80) {
    //println!("process_interrupts");

    // "When an EI instruction is executed, any pending interrupt request
    // is not accepted until after the instruction following EI is executed."
    if z.iff_delay > 0 {
        z.iff_delay -= 1;
        if z.iff_delay == 0 {
            z.iff1 = true;
            z.iff2 = true;
        }
        return;
    }

    if z.nmi_pending {
        z.nmi_pending = false;
        z.halted = false;
        z.iff1 = false;
        inc_r(z);

        z.cyc += 11;
        call(z, 0x66);
        return;
    }

    if z.int_pending && z.iff1 {
        z.int_pending = false;
        z.halted = false;
        z.iff1 = false;
        z.iff2 = false;
        inc_r(z);

        match z.interrupt_mode {
            0 => {
                z.cyc += 11;
                exec_opcode(z, z.int_data);
            }

            1 => {
                z.cyc += 13;
                call(z, 0x38);
            }

            2 => {
                z.cyc += 19;
                let temp = rw(z, ((z.i as u16) << 8) | z.int_data as u16);
                call(z, temp);
            }

            _ => {
                println!("unsupported interrupt mode {}", z.interrupt_mode);
            }
        }

        return;
    }
}

// MARK: interface
// initialises a z80 struct. Note that read_byte, write_byte, port_in, port_out
// and userdata must be manually set by the user afterwards.
pub fn z80_init(z: &mut z80) {
    //println!("z80_init");

    //z.read_byte = NULL;
    //z.write_byte = NULL;
    //z.port_in = NULL;
    //z.port_out = NULL;
    //z.userdata = NULL;

    z.cyc = 0;

    z.pc = 0;
    z.sp = 0xFFFF;
    z.ix = 0;
    z.iy = 0;
    z.mem_ptr = 0;

    // af and sp are set to 0xFFFF after reset,
    // and the other values are undefined (z80-documented)
    z.a = 0xFF;
    z.b = 0;
    z.c = 0;
    z.d = 0;
    z.e = 0;
    z.h = 0;
    z.l = 0;

    z.a_ = 0;
    z.b_ = 0;
    z.c_ = 0;
    z.d_ = 0;
    z.e_ = 0;
    z.h_ = 0;
    z.l_ = 0;
    z.f_ = 0;

    z.i = 0;
    z.r = 0;

    z.sf = 1;
    z.zf = 1;
    z.yf = 1;
    z.hf = 1;
    z.xf = 1;
    z.pf = 1;
    z.nf = 1;
    z.cf = 1;

    z.iff_delay = 0;
    z.interrupt_mode = 0;
    z.iff1 = false;
    z.iff2 = false;
    z.halted = false;
    z.int_pending = false;
    z.nmi_pending = false;
    z.int_data = 0;
}

// executes the next instruction in memory + handles interrupts
pub fn z80_step(z: &mut z80) {
    //println!("z80_step");

    if z.halted {
        exec_opcode(z, 0x00);
    } else {
        let opcode: u8 = nextb(z);
        //println!("BUG z80_step opcode:{}", opcode);
        exec_opcode(z, opcode);
    }

    process_interrupts(z);
}

// outputs to stdout a debug trace of the emulator
pub fn z80_debug_output(z: &mut z80) {
    //println!("z80_debug_output");

    let temp1 = get_f(z);
    let temp2 = get_bc(z);
    let temp3 = get_de(z);
    let temp4 = get_hl(z);
    print!("PC: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}, IX: {:04X}, IY: {:04X}, I: {:02X}, R: {:02X}",
        z.pc, ((z.a as u16) << 8) | temp1 as u16, temp2, temp3, temp4, z.sp,
        z.ix, z.iy, z.i, z.r);

    print!(
        "\t({:02X} {:02X} {:02X} {:02X}), cyc: {}\n",
        rb(z, z.pc),
        rb(z, z.pc + 1),
        rb(z, z.pc + 2),
        rb(z, z.pc + 3),
        z.cyc
    );
}

// function to call when an NMI is to be serviced
pub fn z80_gen_nmi(z: &mut z80) {
    //println!("z80_gen_nmi");

    z.nmi_pending = true;
}

// function to call when an INT is to be serviced
pub fn z80_gen_int(z: &mut z80, data: u8) {
    //println!("z80_gen_int");

    z.int_pending = true;
    z.int_data = data;
}

// executes a non-prefixed opcode
pub fn exec_opcode(z: &mut z80, opcode: u8) {
    //println!("exec_opcode");

    z.cyc += cyc_00[opcode as usize] as u64;
    inc_r(z);

    match opcode {
        0x7F => {
            z.a = z.a;
        } // ld a,a
        0x78 => {
            z.a = z.b;
        } // ld a,b
        0x79 => {
            z.a = z.c;
        } // ld a,c
        0x7A => {
            z.a = z.d;
        } // ld a,d
        0x7B => {
            z.a = z.e;
        } // ld a,e
        0x7C => {
            z.a = z.h;
        } // ld a,h
        0x7D => {
            z.a = z.l;
        } // ld a,l
        0x47 => {
            z.b = z.a;
        } // ld b,a
        0x40 => {
            z.b = z.b;
        } // ld b,b
        0x41 => {
            z.b = z.c;
        } // ld b,c
        0x42 => {
            z.b = z.d;
        } // ld b,d
        0x43 => {
            z.b = z.e;
        } // ld b,e
        0x44 => {
            z.b = z.h;
        } // ld b,h
        0x45 => {
            z.b = z.l;
        } // ld b,l
        0x4F => {
            z.c = z.a;
        } // ld c,a
        0x48 => {
            z.c = z.b;
        } // ld c,b
        0x49 => {
            z.c = z.c;
        } // ld c,c
        0x4A => {
            z.c = z.d;
        } // ld c,d
        0x4B => {
            z.c = z.e;
        } // ld c,e
        0x4C => {
            z.c = z.h;
        } // ld c,h
        0x4D => {
            z.c = z.l;
        } // ld c,l
        0x57 => {
            z.d = z.a;
        } // ld d,a
        0x50 => {
            z.d = z.b;
        } // ld d,b
        0x51 => {
            z.d = z.c;
        } // ld d,c
        0x52 => {
            z.d = z.d;
        } // ld d,d
        0x53 => {
            z.d = z.e;
        } // ld d,e
        0x54 => {
            z.d = z.h;
        } // ld d,h
        0x55 => {
            z.d = z.l;
        } // ld d,l
        0x5F => {
            z.e = z.a;
        } // ld e,a
        0x58 => {
            z.e = z.b;
        } // ld e,b
        0x59 => {
            z.e = z.c;
        } // ld e,c
        0x5A => {
            z.e = z.d;
        } // ld e,d
        0x5B => {
            z.e = z.e;
        } // ld e,e
        0x5C => {
            z.e = z.h;
        } // ld e,h
        0x5D => {
            z.e = z.l;
        } // ld e,l
        0x67 => {
            z.h = z.a;
        } // ld h,a
        0x60 => {
            z.h = z.b;
        } // ld h,b
        0x61 => {
            z.h = z.c;
        } // ld h,c
        0x62 => {
            z.h = z.d;
        } // ld h,d
        0x63 => {
            z.h = z.e;
        } // ld h,e
        0x64 => {
            z.h = z.h;
        } // ld h,h
        0x65 => {
            z.h = z.l;
        } // ld h,l
        0x6F => {
            z.l = z.a;
        } // ld l,a
        0x68 => {
            z.l = z.b;
        } // ld l,b
        0x69 => {
            z.l = z.c;
        } // ld l,c
        0x6A => {
            z.l = z.d;
        } // ld l,d
        0x6B => {
            z.l = z.e;
        } // ld l,e
        0x6C => {
            z.l = z.h;
        } // ld l,h
        0x6D => {
            z.l = z.l;
        } // ld l,l
        0x7E => {
            let result = get_hl(z);
            z.a = rb(z, result);
        } // ld a,(hl)
        0x46 => {
            let result = get_hl(z);
            z.b = rb(z, result);
        } // ld b,(hl)
        0x4E => {
            let result = get_hl(z);
            z.c = rb(z, result);
        } // ld c,(hl)
        0x56 => {
            let result = get_hl(z);
            z.d = rb(z, result);
        } // ld d,(hl)
        0x5E => {
            let result = get_hl(z);
            z.e = rb(z, result);
        } // ld e,(hl)
        0x66 => {
            let result = get_hl(z);
            z.h = rb(z, result);
        } // ld h,(hl)
        0x6E => {
            let result = get_hl(z);
            z.l = rb(z, result);
        } // ld l,(hl)
        0x77 => {
            let result = get_hl(z);
            wb(z, result, z.a);
        } // ld (hl),a
        0x70 => {
            let result = get_hl(z);
            wb(z, result, z.b);
        } // ld (hl),b
        0x71 => {
            let result = get_hl(z);
            wb(z, result, z.c);
        } // ld (hl),c
        0x72 => {
            let result = get_hl(z);
            wb(z, result, z.d);
        } // ld (hl),d
        0x73 => {
            let result = get_hl(z);
            wb(z, result, z.e);
        } // ld (hl),e
        0x74 => {
            let result = get_hl(z);
            wb(z, result, z.h);
        } // ld (hl),h
        0x75 => {
            let result = get_hl(z);
            wb(z, result, z.l);
        } // ld (hl),l
        0x3E => {
            z.a = nextb(z);
        } // ld a,*
        0x06 => {
            z.b = nextb(z);
        } // ld b,*
        0x0E => {
            z.c = nextb(z);
        } // ld c,*
        0x16 => {
            z.d = nextb(z);
        } // ld d,*
        0x1E => {
            z.e = nextb(z);
        } // ld e,*
        0x26 => {
            z.h = nextb(z);
        } // ld h,*
        0x2E => {
            z.l = nextb(z);
        } // ld l,*
        0x36 => {
            let temp1 = get_hl(z);
            let temp2 = nextb(z);
            wb(z, temp1, temp2);
        } // ld (hl),*
        0x0A => {
            let temp = get_bc(z);
            z.a = rb(z, temp);
            z.mem_ptr = get_bc(z) + 1;
            // ld a,(bc)
        }
        0x1A => {
            let temp = get_de(z);
            z.a = rb(z, temp);
            z.mem_ptr = get_de(z) + 1;
            // ld a,(de)
        }
        0x3A => {
            let addr: u16 = nextw(z);
            z.a = rb(z, addr);
            z.mem_ptr = addr + 1;
        } // ld a,(**)
        0x02 => {
            let temp = get_bc(z);
            wb(z, temp, z.a);
            z.mem_ptr = ((z.a as u16) << 8) | ((get_bc(z) + 1) & 0xFF);
            // ld (bc),a
        }
        0x12 => {
            let temp = get_de(z);
            wb(z, temp, z.a);
            z.mem_ptr = ((z.a as u16) << 8) | ((get_de(z) + 1) & 0xFF);
            // ld (de),a
        }
        0x32 => {
            let addr: u16 = nextw(z);
            wb(z, addr, z.a);
            z.mem_ptr = ((z.a as u16) << 8) | ((addr + 1) & 0xFF);
        } // ld (**),a
        0x01 => {
            let temp = nextw(z);
            set_bc(z, temp);
        } // ld bc,**
        0x11 => {
            let temp = nextw(z);
            set_de(z, temp);
        } // ld de,**
        0x21 => {
            let temp = nextw(z);
            set_hl(z, temp);
        } // ld hl,**
        0x31 => {
            z.sp = nextw(z);
        } // ld sp,**
        0x2A => {
            let addr: u16 = nextw(z);
            let temp = rw(z, addr);
            set_hl(z, temp);
            z.mem_ptr = addr + 1;
        } // ld hl,(**)
        0x22 => {
            let addr: u16 = nextw(z);
            let temp = get_hl(z);
            ww(z, addr, temp);
            z.mem_ptr = addr + 1;
        } // ld (**),hl
        0xF9 => {
            z.sp = get_hl(z);
        } // ld sp,hl
        0xEB => {
            let de: u16 = get_de(z);
            let temp = get_hl(z);
            set_de(z, temp);
            set_hl(z, de);
        } // ex de,hl
        0xE3 => {
            let val: u16 = rw(z, z.sp);
            let temp = get_hl(z);
            ww(z, z.sp, temp);
            set_hl(z, val);
            z.mem_ptr = val;
        } // ex (sp),hl
        0x87 => {
            z.a = addb(z, z.a, z.a, 0);
        } // add a,a
        0x80 => {
            z.a = addb(z, z.a, z.b, 0);
        } // add a,b
        0x81 => {
            z.a = addb(z, z.a, z.c, 0);
        } // add a,c
        0x82 => {
            z.a = addb(z, z.a, z.d, 0);
        } // add a,d
        0x83 => {
            z.a = addb(z, z.a, z.e, 0);
        } // add a,e
        0x84 => {
            z.a = addb(z, z.a, z.h, 0);
        } // add a,h
        0x85 => {
            z.a = addb(z, z.a, z.l, 0);
        } // add a,l
        0x86 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            z.a = addb(z, z.a, temp2, 0);
        } // add a,(hl)
        0xC6 => {
            let temp = nextb(z);
            z.a = addb(z, z.a, temp, 0);
        } // add a,*
        0x8F => {
            z.a = addb(z, z.a, z.a, z.cf);
        } // adc a,a
        0x88 => {
            z.a = addb(z, z.a, z.b, z.cf);
        } // adc a,b
        0x89 => {
            z.a = addb(z, z.a, z.c, z.cf);
        } // adc a,c
        0x8A => {
            z.a = addb(z, z.a, z.d, z.cf);
        } // adc a,d
        0x8B => {
            z.a = addb(z, z.a, z.e, z.cf);
        } // adc a,e
        0x8C => {
            z.a = addb(z, z.a, z.h, z.cf);
        } // adc a,h
        0x8D => {
            z.a = addb(z, z.a, z.l, z.cf);
        } // adc a,l
        0x8E => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            z.a = addb(z, z.a, temp2, z.cf);
        } // adc a,(hl)
        0xCE => {
            let temp = nextb(z);
            z.a = addb(z, z.a, temp, z.cf);
        } // adc a,*
        0x97 => {
            z.a = subb(z, z.a, z.a, 0);
        } // sub a,a
        0x90 => {
            z.a = subb(z, z.a, z.b, 0);
        } // sub a,b
        0x91 => {
            z.a = subb(z, z.a, z.c, 0);
        } // sub a,c
        0x92 => {
            z.a = subb(z, z.a, z.d, 0);
        } // sub a,d
        0x93 => {
            z.a = subb(z, z.a, z.e, 0);
        } // sub a,e
        0x94 => {
            z.a = subb(z, z.a, z.h, 0);
        } // sub a,h
        0x95 => {
            z.a = subb(z, z.a, z.l, 0);
        } // sub a,l
        0x96 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            z.a = subb(z, z.a, temp2, 0);
        } // sub a,(hl)
        0xD6 => {
            let temp = nextb(z);
            z.a = subb(z, z.a, temp, 0);
        } // sub a,*
        0x9F => {
            z.a = subb(z, z.a, z.a, z.cf);
        } // sbc a,a
        0x98 => {
            z.a = subb(z, z.a, z.b, z.cf);
        } // sbc a,b
        0x99 => {
            z.a = subb(z, z.a, z.c, z.cf);
        } // sbc a,c
        0x9A => {
            z.a = subb(z, z.a, z.d, z.cf);
        } // sbc a,d
        0x9B => {
            z.a = subb(z, z.a, z.e, z.cf);
        } // sbc a,e
        0x9C => {
            z.a = subb(z, z.a, z.h, z.cf);
        } // sbc a,h
        0x9D => {
            z.a = subb(z, z.a, z.l, z.cf);
        } // sbc a,l
        0x9E => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            z.a = subb(z, z.a, temp2, z.cf);
        } // sbc a,(hl)
        0xDE => {
            let temp = nextb(z);
            z.a = subb(z, z.a, temp, z.cf);
        } // sbc a,*
        0x09 => {
            let temp = get_bc(z);
            addhl(z, temp);
        } // add hl,bc
        0x19 => {
            let temp = get_de(z);
            addhl(z, temp);
        } // add hl,de
        0x29 => {
            let temp = get_hl(z);
            addhl(z, temp);
        } // add hl,hl
        0x39 => {
            addhl(z, z.sp);
        } // add hl,sp
        0xF3 => {
            z.iff1 = false;
            z.iff2 = false;
            // di
        }
        0xFB => {
            z.iff_delay = 1;
        } // ei
        0x00 => {} // nop
        0x76 => {
            z.halted = true;
        } // halt
        0x3C => {
            z.a = inc(z, z.a);
        } // inc a
        0x04 => {
            z.b = inc(z, z.b);
        } // inc b
        0x0C => {
            z.c = inc(z, z.c);
        } // inc c
        0x14 => {
            z.d = inc(z, z.d);
        } // inc d
        0x1C => {
            z.e = inc(z, z.e);
        } // inc e
        0x24 => {
            z.h = inc(z, z.h);
        } // inc h
        0x2C => {
            z.l = inc(z, z.l);
        } // inc l
        0x34 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            let result: u16 = inc(z, temp2) as u16;
            let temp3 = get_hl(z);
            wb(z, temp3, result as u8);
        } // inc (hl)
        0x3D => {
            z.a = dec(z, z.a);
        } // dec a
        0x05 => {
            z.b = dec(z, z.b);
        } // dec b
        0x0D => {
            z.c = dec(z, z.c);
        } // dec c
        0x15 => {
            z.d = dec(z, z.d);
        } // dec d
        0x1D => {
            z.e = dec(z, z.e);
        } // dec e
        0x25 => {
            z.h = dec(z, z.h);
        } // dec h
        0x2D => {
            z.l = dec(z, z.l);
        } // dec l
        0x35 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            let result: u16 = dec(z, temp2) as u16;
            let temp3 = get_hl(z);
            wb(z, temp3, result as u8);
        } // dec (hl)
        0x03 => {
            let temp = get_bc(z);
            set_bc(z, temp + 1);
        } // inc bc
        0x13 => {
            let temp = get_de(z);
            set_de(z, temp + 1);
        } // inc de
        0x23 => {
            let temp = get_hl(z);
            set_hl(z, temp + 1);
        } // inc hl
        0x33 => {
            z.sp = z.sp + 1;
        } // inc sp
        0x0B => {
            let temp = get_bc(z);
            set_bc(z, temp - 1);
        } // dec bc
        0x1B => {
            let temp = get_de(z);
            set_de(z, temp - 1);
        } // dec de
        0x2B => {
            let temp = get_hl(z);
            set_hl(z, temp - 1);
        } // dec hl
        0x3B => {
            z.sp = z.sp - 1;
        } // dec sp
        0x27 => {
            daa(z);
        } // daa
        0x2F => {
            z.a = !z.a;
            z.nf = 1;
            z.hf = 1;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
            // cpl
        }
        0x37 => {
            z.cf = 1;
            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
            // scf
        }
        0x3F => {
            z.hf = z.cf;
            z.cf = !z.cf & 0x1;
            z.nf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
            // ccf
        }
        0x07 => {
            z.cf = z.a >> 7;
            z.a = (z.a << 1) | z.cf;
            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
        } // rlca (rotate left)
        0x0F => {
            z.cf = z.a & 1;
            z.a = (z.a >> 1) | (z.cf << 7);
            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
        } // rrca (rotate right)
        0x17 => {
            let cy: u8 = z.cf;
            let cy_ = cy & 0x01;
            z.cf = z.a >> 7;
            z.a = (z.a << 1) | cy_;
            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
        } // rla
        0x1F => {
            let cy: u8 = z.cf;
            let cy_ = cy & 0x01;
            z.cf = z.a & 1;
            z.a = (z.a >> 1) | (cy_ << 7);
            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
        } // rra
        0xA7 => {
            land(z, z.a);
        } // and a
        0xA0 => {
            land(z, z.b);
        } // and b
        0xA1 => {
            land(z, z.c);
        } // and c
        0xA2 => {
            land(z, z.d);
        } // and d
        0xA3 => {
            land(z, z.e);
        } // and e
        0xA4 => {
            land(z, z.h);
        } // and h
        0xA5 => {
            land(z, z.l);
        } // and l
        0xA6 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            land(z, temp2);
        } // and (hl)
        0xE6 => {
            let temp = nextb(z);
            land(z, temp);
        } // and *
        0xAF => {
            lxor(z, z.a);
        } // xor a
        0xA8 => {
            lxor(z, z.b);
        } // xor b
        0xA9 => {
            lxor(z, z.c);
        } // xor c
        0xAA => {
            lxor(z, z.d);
        } // xor d
        0xAB => {
            lxor(z, z.e);
        } // xor e
        0xAC => {
            lxor(z, z.h);
        } // xor h
        0xAD => {
            lxor(z, z.l);
        } // xor l
        0xAE => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            lxor(z, temp2);
        } // xor (hl)
        0xEE => {
            let temp = nextb(z);
            lxor(z, temp);
        } // xor *
        0xB7 => {
            lor(z, z.a);
        } // or a
        0xB0 => {
            lor(z, z.b);
        } // or b
        0xB1 => {
            lor(z, z.c);
        } // or c
        0xB2 => {
            lor(z, z.d);
        } // or d
        0xB3 => {
            lor(z, z.e);
        } // or e
        0xB4 => {
            lor(z, z.h);
        } // or h
        0xB5 => {
            lor(z, z.l);
        } // or l
        0xB6 => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            lor(z, temp2);
        } // or (hl)
        0xF6 => {
            let temp = nextb(z);
            lor(z, temp);
        } // or *
        0xBF => {
            cp(z, z.a);
        } // cp a
        0xB8 => {
            cp(z, z.b);
        } // cp b
        0xB9 => {
            cp(z, z.c);
        } // cp c
        0xBA => {
            cp(z, z.d);
        } // cp d
        0xBB => {
            cp(z, z.e);
        } // cp e
        0xBC => {
            cp(z, z.h);
        } // cp h
        0xBD => {
            cp(z, z.l);
        } // cp l
        0xBE => {
            let temp1 = get_hl(z);
            let temp2 = rb(z, temp1);
            cp(z, temp2);
        } // cp (hl)
        0xFE => {
            let temp = nextb(z);
            cp(z, temp);
        } // cp *
        0xC3 => {
            let temp = nextw(z);
            jump(z, temp);
        } // jm **
        0xC2 => {
            cond_jump(z, z.zf == 0);
        } // jp nz, **
        0xCA => {
            cond_jump(z, z.zf == 1);
        } // jp z, **
        0xD2 => {
            cond_jump(z, z.cf == 0);
        } // jp nc, **
        0xDA => {
            cond_jump(z, z.cf == 1);
        } // jp c, **
        0xE2 => {
            cond_jump(z, z.pf == 0);
        } // jp po, **
        0xEA => {
            cond_jump(z, z.pf == 1);
        } // jp pe, **
        0xF2 => {
            cond_jump(z, z.sf == 0);
        } // jp p, **
        0xFA => {
            cond_jump(z, z.sf == 1);
        } // jp m, **
        0x10 => {
            let condition: bool;
            z.b = z.b.wrapping_sub(1);
            if z.b != 0 {
                condition = true;
            } else {
                condition = false;
            }
            cond_jr(z, condition);
        } // djnz *
        0x18 => {
            let temp = nextb(z) as i8;
            z.pc = z.pc.wrapping_add(temp as u16);
        } // jr *
        0x20 => {
            cond_jr(z, z.zf == 0);
        } // jr nz, *
        0x28 => {
            cond_jr(z, z.zf == 1);
        } // jr z, *
        0x30 => {
            cond_jr(z, z.cf == 0);
        } // jr nc, *
        0x38 => {
            cond_jr(z, z.cf == 1);
        } // jr c, *
        0xE9 => {
            z.pc = get_hl(z);
        } // jp (hl)
        0xCD => {
            let temp = nextw(z);
            call(z, temp);
        } // call
        0xC4 => {
            cond_call(z, z.zf == 0);
        } // cnz
        0xCC => {
            cond_call(z, z.zf == 1);
        } // cz
        0xD4 => {
            cond_call(z, z.cf == 0);
        } // cnc
        0xDC => {
            cond_call(z, z.cf == 1);
        } // cc
        0xE4 => {
            cond_call(z, z.pf == 0);
        } // cpo
        0xEC => {
            cond_call(z, z.pf == 1);
        } // cpe
        0xF4 => {
            cond_call(z, z.sf == 0);
        } // cp
        0xFC => {
            cond_call(z, z.sf == 1);
        } // cm
        0xC9 => {
            ret(z);
        } // ret
        0xC0 => {
            cond_ret(z, z.zf == 0);
        } // ret nz
        0xC8 => {
            cond_ret(z, z.zf == 1);
        } // ret z
        0xD0 => {
            cond_ret(z, z.cf == 0);
        } // ret nc
        0xD8 => {
            cond_ret(z, z.cf == 1);
        } // ret c
        0xE0 => {
            cond_ret(z, z.pf == 0);
        } // ret po
        0xE8 => {
            cond_ret(z, z.pf == 1);
        } // ret pe
        0xF0 => {
            cond_ret(z, z.sf == 0);
        } // ret p
        0xF8 => {
            cond_ret(z, z.sf == 1);
        } // ret m
        0xC7 => {
            call(z, 0x00);
        } // rst 0
        0xCF => {
            call(z, 0x08);
        } // rst 1
        0xD7 => {
            call(z, 0x10);
        } // rst 2
        0xDF => {
            call(z, 0x18);
        } // rst 3
        0xE7 => {
            call(z, 0x20);
        } // rst 4
        0xEF => {
            call(z, 0x28);
        } // rst 5
        0xF7 => {
            call(z, 0x30);
        } // rst 6
        0xFF => {
            call(z, 0x38);
        } // rst 7
        0xC5 => {
            let temp = get_bc(z);
            pushw(z, temp);
        } // push bc
        0xD5 => {
            let temp = get_de(z);
            pushw(z, temp);
        } // push de
        0xE5 => {
            let temp = get_hl(z);
            pushw(z, temp);
        } // push hl
        0xF5 => {
            let temp = get_f(z);
            pushw(z, ((z.a as u16) << 8) | temp as u16);
        } // push af
        0xC1 => {
            let temp = popw(z);
            set_bc(z, temp);
        } // pop bc
        0xD1 => {
            let temp = popw(z);
            set_de(z, temp);
        } // pop de
        0xE1 => {
            let temp = popw(z);
            set_hl(z, temp);
        } // pop hl
        0xF1 => {
            let val: u16 = popw(z);
            z.a = (val >> 8) as u8;
            set_f(z, (val & 0xFF) as u8);
        } // pop af
        0xDB => {
            let port: u8 = nextb(z);
            let a: u8 = z.a;
            z.a = (z.port_in)(z, port);
            z.mem_ptr = ((a as u16) << 8) | (z.a as u16 + 1);
        } // in a,(n)
        0xD3 => {
            let port: u8 = nextb(z);
            (z.port_out)(z, port, z.a);
            z.mem_ptr = (port as u16 + 1) | ((z.a as u16) << 8);
        } // out (n), a
        0x08 => {
            let a: u8 = z.a;
            let f: u8 = get_f(z);

            z.a = z.a_;
            set_f(z, z.f_);

            z.a_ = a;
            z.f_ = f;
        } // ex af,af'
        0xD9 => {
            let b: u8 = z.b;
            let c: u8 = z.c;
            let d: u8 = z.d;
            let e: u8 = z.e;
            let h: u8 = z.h;
            let l: u8 = z.l;

            z.b = z.b_;
            z.c = z.c_;
            z.d = z.d_;
            z.e = z.e_;
            z.h = z.h_;
            z.l = z.l_;

            z.b_ = b;
            z.c_ = c;
            z.d_ = d;
            z.e_ = e;
            z.h_ = h;
            z.l_ = l;
        } // exx
        0xCB => {
            let temp = nextb(z);
            exec_opcode_cb(z, temp);
        }
        0xED => {
            let temp = nextb(z);
            exec_opcode_ed(z, temp);
        }
        0xDD => {
            let temp1 = nextb(z);
            let mut temp2 = z.ix;
            exec_opcode_ddfd(z, temp1, &mut temp2);
            z.ix = temp2;
        }
        0xFD => {
            let temp1 = nextb(z);
            let mut temp2 = z.iy;
            exec_opcode_ddfd(z, temp1, &mut temp2);
            z.iy = temp2;
        }
    }
}

// executes a DD/FD opcode (IZ = IX or IY)
pub fn exec_opcode_ddfd(z: &mut z80, opcode: u8, iz: &mut u16) {
    //println!("exec_opcode_ddfd");

    z.cyc += cyc_ddfd[opcode as usize] as u64;
    inc_r(z);

    match opcode {
        0xE1 => {
            *iz = popw(z);
        } // pop iz
        0xE5 => {
            pushw(z, *iz);
        } // push iz
        0xE9 => {
            jump(z, *iz);
        } // jp iz
        0x09 => {
            let temp = get_bc(z);
            addiz(z, iz, temp);
        } // add iz,bc
        0x19 => {
            let temp = get_de(z);
            addiz(z, iz, temp);
        } // add iz,de
        0x29 => {
            addiz(z, iz, *iz);
        } // add iz,iz
        0x39 => {
            addiz(z, iz, z.sp);
        } // add iz,sp
        0x84 => {
            let IZH = *iz >> 8;
            z.a = addb(z, z.a, IZH as u8, 0);
        } // add a,izh
        0x85 => {
            z.a = addb(z, z.a, (*iz & 0xFF) as u8, 0);
        } // add a,izl
        0x8C => {
            let IZH = *iz >> 8;
            z.a = addb(z, z.a, IZH as u8, z.cf);
        } // adc a,izh
        0x8D => {
            z.a = addb(z, z.a, (*iz & 0xFF) as u8, z.cf);
        } // adc a,izl
        0x86 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            z.a = addb(z, z.a, temp, 0);
        } // add a,(iz+*)
        0x8E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            z.a = addb(z, z.a, temp, z.cf);
        } // adc a,(iz+*)
        0x96 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            z.a = subb(z, z.a, temp, 0);
        } // sub (iz+*)
        0x9E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            z.a = subb(z, z.a, temp, z.cf);
        } // sbc (iz+*)
        0x94 => {
            let IZH = *iz >> 8;
            z.a = subb(z, z.a, IZH as u8, 0);
        } // sub izh
        0x95 => {
            z.a = subb(z, z.a, (*iz & 0xFF) as u8, 0);
        } // sub izl
        0x9C => {
            let IZH = *iz >> 8;
            z.a = subb(z, z.a, IZH as u8, z.cf);
        } // sbc izh
        0x9D => {
            z.a = subb(z, z.a, (*iz & 0xFF) as u8, z.cf);
        } // sbc izl
        0xA6 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            land(z, temp);
        } // and (iz+*)
        0xA4 => {
            let IZH = *iz >> 8;
            land(z, IZH as u8);
        } // and izh
        0xA5 => {
            land(z, (*iz & 0xFF) as u8);
        } // and izl
        0xAE => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            lxor(z, temp);
        } // xor (iz+*)
        0xAC => {
            let IZH = *iz >> 8;
            lxor(z, IZH as u8);
        } // xor izh
        0xAD => {
            lxor(z, (*iz & 0xFF) as u8);
        } // xor izl
        0xB6 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            lor(z, temp);
        } // or (iz+*)
        0xB4 => {
            let IZH = *iz >> 8;
            lor(z, IZH as u8);
        } // or izh
        0xB5 => {
            lor(z, (*iz & 0xFF) as u8);
        } // or izl
        0xBE => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let temp = rb(z, IZD);
            cp(z, temp);
        } // cp (iz+*)
        0xBC => {
            let IZH = *iz >> 8;
            cp(z, IZH as u8);
        } // cp izh
        0xBD => {
            cp(z, (*iz & 0xFF) as u8);
        } // cp izl
        0x23 => {
            *iz += 1;
        } // inc iz
        0x2B => {
            *iz -= 1;
        } // dec iz
        0x34 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let addr: u16 = IZD;
            let temp1 = rb(z, addr);
            let temp2 = inc(z, temp1);
            wb(z, addr, temp2);
        } // inc (iz+*)
        0x35 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let addr: u16 = IZD;
            let temp1 = rb(z, addr);
            let temp2 = dec(z, temp1);
            wb(z, addr, temp2);
        } // dec (iz+*)
        0x24 => {
            let IZL = *iz & 0xFF;
            let IZH = *iz >> 8;
            *iz = IZL | ((inc(z, IZH as u8) as u16) << 8);
        } // inc izh
        0x25 => {
            let IZL = *iz & 0xFF;
            let IZH = *iz >> 8;
            *iz = IZL | ((dec(z, IZH as u8) as u16) << 8);
        } // dec izh
        0x2C => {
            let IZH = *iz >> 8;
            let IZL = *iz & 0xFF;
            *iz = (IZH << 8) | inc(z, IZL as u8) as u16;
        } // inc izl
        0x2D => {
            let IZH = *iz >> 8;
            let IZL = *iz & 0xFF;
            *iz = (IZH << 8) | dec(z, IZL as u8) as u16;
        } // dec izl
        0x2A => {
            let temp = nextw(z);
            *iz = rw(z, temp);
        } // ld iz,(**)
        0x22 => {
            let temp = nextw(z);
            ww(z, temp, *iz);
        } // ld (**),iz
        0x21 => {
            *iz = nextw(z);
        } // ld iz,**
        0x36 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let addr: u16 = IZD;
            let temp = nextb(z);
            wb(z, addr, temp);
        } // ld (iz+*),*
        0x70 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.b);
        } // ld (iz+*),b
        0x71 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.c);
        } // ld (iz+*),c
        0x72 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.d);
        } // ld (iz+*),d
        0x73 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.e);
        } // ld (iz+*),e
        0x74 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.h);
        } // ld (iz+*),h
        0x75 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.l);
        } // ld (iz+*),l
        0x77 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            wb(z, IZD, z.a);
        } // ld (iz+*),a
        0x46 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.b = rb(z, IZD);
        } // ld b,(iz+*)
        0x4E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.c = rb(z, IZD);
        } // ld c,(iz+*)
        0x56 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.d = rb(z, IZD);
        } // ld d,(iz+*)
        0x5E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.e = rb(z, IZD);
        } // ld e,(iz+*)
        0x66 => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.h = rb(z, IZD);
        } // ld h,(iz+*)
        0x6E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.l = rb(z, IZD);
        } // ld l,(iz+*)
        0x7E => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            z.a = rb(z, IZD);
        } // ld a,(iz+*)
        0x44 => {
            let IZH = *iz >> 8;
            z.b = IZH as u8;
        } // ld b,izh
        0x4C => {
            let IZH = *iz >> 8;
            z.c = IZH as u8;
        } // ld c,izh
        0x54 => {
            let IZH = *iz >> 8;
            z.d = IZH as u8;
        } // ld d,izh
        0x5C => {
            let IZH = *iz >> 8;
            z.e = IZH as u8;
        } // ld e,izh
        0x7C => {
            let IZH = *iz >> 8;
            z.a = IZH as u8;
        } // ld a,izh
        0x45 => {
            let IZL = *iz & 0xFF;
            z.b = IZL as u8;
        } // ld b,izl
        0x4D => {
            let IZL = *iz & 0xFF;
            z.c = IZL as u8;
        } // ld c,izl
        0x55 => {
            let IZL = *iz & 0xFF;
            z.d = IZL as u8;
        } // ld d,izl
        0x5D => {
            let IZL = *iz & 0xFF;
            z.e = IZL as u8;
        } // ld e,izl
        0x7D => {
            let IZL = *iz & 0xFF;
            z.a = IZL as u8;
        } // ld a,izl
        0x60 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | ((z.b as u16) << 8);
        } // ld izh,b
        0x61 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | ((z.c as u16) << 8);
        } // ld izh,c
        0x62 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | ((z.d as u16) << 8);
        } // ld izh,d
        0x63 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | ((z.e as u16) << 8);
        } // ld izh,e
        0x64 => {} // ld izh,izh
        0x65 => {
            let IZL = *iz & 0xFF;
            *iz = (IZL << 8) | IZL;
        } // ld izh,izl
        0x67 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | (((z.a) as u16) << 8);
        } // ld izh,a
        0x26 => {
            let IZL = *iz & 0xFF;
            *iz = IZL | ((nextb(z) as u16) << 8);
        } // ld izh,*
        0x68 => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | z.b as u16;
        } // ld izl,b
        0x69 => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | z.c as u16;
        } // ld izl,c
        0x6A => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | z.d as u16;
        } // ld izl,d
        0x6B => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | z.e as u16;
        } // ld izl,e
        0x6C => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | IZH as u16;
        } // ld izl,izh
        0x6D => {} // ld izl,izl
        0x6F => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | z.a as u16;
        } // ld izl,a
        0x2E => {
            let IZH = *iz >> 8;
            *iz = (IZH << 8) | nextb(z) as u16;
        } // ld izl,*
        0xF9 => {
            z.sp = *iz;
        } // ld sp,iz
        0xE3 => {
            let val: u16 = rw(z, z.sp);
            ww(z, z.sp, *iz);
            *iz = val;
            z.mem_ptr = val;
        } // ex (sp),iz
        0xCB => {
            let temp = nextb(z);
            let IZD = displace(z, *iz, temp as i8);
            let addr: u16 = IZD;
            let op: u8 = nextb(z);
            exec_opcode_dcb(z, op, addr);
        }
        _ => {
            // any other FD/DD opcode behaves as a non-prefixed opcode:
            exec_opcode(z, opcode);
            // R should not be incremented twice:
            z.r = (z.r & 0x80) | ((z.r.wrapping_sub(1)) & 0x7f);
        }
    }
}

// executes a CB opcode
pub fn exec_opcode_cb(z: &mut z80, opcode: u8) {
    //println!("exec_opcode_cb");

    z.cyc += 8;
    inc_r(z);

    // decoding instructions from http://z80.info/decoding.htm#cb
    let x_: u8 = (opcode >> 6) & 3; // 0b11
    let y_: u8 = (opcode >> 3) & 7; // 0b111
    let z_: u8 = opcode & 7; // 0b111

    let mut hl: u8 = 0;
    let mut reg: *mut u8 = ptr::null_mut();
    match z_ {
        0 => {
            reg = &mut z.b;
        }
        1 => {
            reg = &mut z.c;
        }
        2 => {
            reg = &mut z.d;
        }
        3 => {
            reg = &mut z.e;
        }
        4 => {
            reg = &mut z.h;
        }
        5 => {
            reg = &mut z.l;
        }
        6 => {
            let temp = get_hl(z);
            hl = rb(z, temp);
            reg = &mut hl;
        }

        7 => {
            reg = &mut z.a;
        }
        _ => {}
    }
    unsafe {
        match x_ {
            0 => match y_ {
                0 => {
                    *reg = cb_rlc(z, *reg);
                }
                1 => {
                    *reg = cb_rrc(z, *reg);
                }
                2 => {
                    *reg = cb_rl(z, *reg);
                }
                3 => {
                    *reg = cb_rr(z, *reg);
                }
                4 => {
                    *reg = cb_sla(z, *reg);
                }
                5 => {
                    *reg = cb_sra(z, *reg);
                }
                6 => {
                    *reg = cb_sll(z, *reg);
                }
                7 => {
                    *reg = cb_srl(z, *reg);
                }
                _ => {}
            }, // rot[y] r[z]
            1 => {
                // BIT y, r[z]
                {
                    cb_bit(z, *reg, y_);
                }

                // in bit (hl), x/y flags are handled differently:
                if z_ == 6 {
                    z.yf = GET_BIT(5, (z.mem_ptr >> 8) as u8);
                    z.xf = GET_BIT(3, (z.mem_ptr >> 8) as u8);
                    z.cyc += 4;
                }
            }
            2 => {
                *reg &= !(1 << y_);
            } // RES y, r[z]
            3 => {
                *reg |= 1 << y_;
            } // SET y, r[z]
            _ => {}
        }
    }

    if (x_ == 0 || x_ == 2 || x_ == 3) && z_ == 6 {
        z.cyc += 7;
    }

    if reg == &mut hl {
        let temp = get_hl(z);
        wb(z, temp, hl);
    }
}

// executes a displaced CB opcode (DDCB or FDCB)
pub fn exec_opcode_dcb(z: &mut z80, opcode: u8, addr: u16) {
    //println!("exec_opcode_dcb");

    let val: u8 = rb(z, addr);
    let mut result: u16 = 0;

    // decoding instructions from http://z80.info/decoding.htm#ddcb
    let x_: u8 = (opcode >> 6) & 3; // 0b11
    let y_: u8 = (opcode >> 3) & 7; // 0b111
    let z_: u8 = opcode & 7; // 0b111

    match x_ {
        0 => {
            // rot[y] (iz+d)
            match y_ {
                0 => {
                    result = cb_rlc(z, val) as u16;
                }
                1 => {
                    result = cb_rrc(z, val) as u16;
                }
                2 => {
                    result = cb_rl(z, val) as u16;
                }
                3 => {
                    result = cb_rr(z, val) as u16;
                }
                4 => {
                    result = cb_sla(z, val) as u16;
                }
                5 => {
                    result = cb_sra(z, val) as u16;
                }
                6 => {
                    result = cb_sll(z, val) as u16;
                }
                7 => {
                    result = cb_srl(z, val) as u16;
                }
                _ => {}
            }
        }
        1 => {
            result = cb_bit(z, val, y_) as u16;
            z.yf = GET_BIT(5, (addr >> 8) as u8);
            z.xf = GET_BIT(3, (addr >> 8) as u8);
        } // bit y,(iz+d)
        2 => {
            result = (val & !(1 << y_)) as u16;
        } // res y, (iz+d)
        3 => {
            result = (val | (1 << y_)) as u16;
        } // set y, (iz+d)

        _ => {
            println!("unknown XYCB opcode: {:2x}", opcode);
        }
    }

    // ld r[z], rot[y] (iz+d)
    // ld r[z], res y,(iz+d)
    // ld r[z], set y,(iz+d)

    if x_ != 1 && z_ != 6 {
        match z_ {
            0 => {
                z.b = result as u8;
            }
            1 => {
                z.c = result as u8;
            }
            2 => {
                z.d = result as u8;
            }
            3 => {
                z.e = result as u8;
            }
            4 => {
                z.h = result as u8;
            }
            5 => {
                z.l = result as u8;
            }
            6 => {
                let temp = get_hl(z);
                wb(z, temp, result as u8);
            }
            7 => {
                z.a = result as u8;
            }
            _ => {}
        }
    }

    if x_ == 1 {
        // bit instructions take 20 cycles, others take 23
        z.cyc += 20;
    } else {
        wb(z, addr, result as u8);
        z.cyc += 23;
    }
}

// executes a ED opcode
pub fn exec_opcode_ed(z: &mut z80, opcode: u8) {
    //println!("exec_opcode_ed");

    z.cyc += cyc_ed[opcode as usize] as u64;
    inc_r(z);
    match opcode {
        0x47 => {
            z.i = z.a;
        } // ld i,a
        0x4F => {
            z.r = z.a;
        } // ld r,a

        0x57 => {
            z.a = z.i;
            z.sf = z.a >> 7;
            if z.a == 0 {
                z.zf = 1;
            } else {
                z.zf = 0;
            }
            z.hf = 0;
            z.nf = 0;
            if z.iff2 {
                z.pf = 1;
            } else {
                z.pf = 0;
            }
            // ld a,i
        }
        0x5F => {
            z.a = z.r;
            z.sf = z.a >> 7;
            if z.a == 0 {
                z.zf = 1;
            } else {
                z.zf = 0;
            }
            z.hf = 0;
            z.nf = 0;
            if z.iff2 {
                z.pf = 1;
            } else {
                z.pf = 0;
            }
            // ld a,r
        }
        0x45 | 0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D => {
            z.iff1 = z.iff2;
            ret(z);
            // retn
        }
        0x4D => {
            ret(z);
        } // reti

        0xA0 => {
            ldi(z);
        } // ldi
        0xB0 => {
            ldi(z);

            if get_bc(z) != 0 {
                z.pc -= 2;
                z.cyc += 5;
                z.mem_ptr = z.pc + 1;
            }
        } // ldir
        0xA8 => {
            ldd(z);
        } // ldd
        0xB8 => {
            ldd(z);

            if get_bc(z) != 0 {
                z.pc -= 2;
                z.cyc += 5;
                z.mem_ptr = z.pc + 1;
            }
        } // lddr
        0xA1 => {
            cpi(z);
        } // cpi
        0xA9 => {
            cpd(z);
        } // cpd
        0xB1 => {
            cpi(z);
            if get_bc(z) != 0 && z.zf == 0 {
                z.pc -= 2;
                z.cyc += 5;
                z.mem_ptr = z.pc + 1;
            } else {
                z.mem_ptr += 1;
            }
        } // cpir
        0xB9 => {
            cpd(z);
            if get_bc(z) != 0 && z.zf == 0 {
                z.pc -= 2;
                z.cyc += 5;
            } else {
                z.mem_ptr += 1;
            }
        } // cpdr
        0x40 => {
            let mut value = z.b;
            in_r_c(z, &mut value);
            z.b = value;
        } // in b, (c)
        0x48 => {
            let mut value = z.c;
            in_r_c(z, &mut value);
            z.c = value;
        } // in c, (c)
        0x50 => {
            let mut value = z.d;
            in_r_c(z, &mut value);
            z.d = value;
        } // in d, (c)
        0x58 => {
            let mut value = z.e;
            in_r_c(z, &mut value);
            z.e = value;
        } // in e, (c)
        0x60 => {
            let mut value = z.h;
            in_r_c(z, &mut value);
            z.h = value;
        } // in h, (c)
        0x68 => {
            let mut value = z.l;
            in_r_c(z, &mut value);
            z.l = value;
        } // in l, (c)
        0x70 => {
            let mut val: u8 = 0;
            in_r_c(z, &mut val);
        } // in (c)
        0x78 => {
            let mut value = z.a;
            in_r_c(z, &mut value);
            z.a = value;
            z.mem_ptr = get_bc(z) + 1;
            // in a, (c)
        }
        0xA2 => {
            ini(z);
        } // ini}
        0xB2 => {
            ini(z);
            if z.b > 0 {
                z.pc -= 2;
                z.cyc += 5;
            }
            // inir
        }
        0xAA => {
            ind(z);
        } // ind}
        0xBA => {
            ind(z);
            if z.b > 0 {
                z.pc -= 2;
                z.cyc += 5;
            }
            // indr
        }
        0x41 => {
            (z.port_out)(z, z.c, z.b);
        } // out (c), b
        0x49 => {
            (z.port_out)(z, z.c, z.c);
        } // out (c), c
        0x51 => {
            (z.port_out)(z, z.c, z.d);
        } // out (c), d
        0x59 => {
            (z.port_out)(z, z.c, z.e);
        } // out (c), e
        0x61 => {
            (z.port_out)(z, z.c, z.h);
        } // out (c), h
        0x69 => {
            (z.port_out)(z, z.c, z.l);
        } // out (c), l
        0x71 => {
            (z.port_out)(z, z.c, 0);
        } // out (c), 0
        0x79 => {
            (z.port_out)(z, z.c, z.a);
            z.mem_ptr = get_bc(z) + 1;
            // out (c), a
        }
        0xA3 => {
            outi(z);
        } // outi
        0xB3 => {
            outi(z);
            if z.b > 0 {
                z.pc -= 2;
                z.cyc += 5;
            }
        } // otir
        0xAB => {
            outd(z);
        } // outd
        0xBB => {
            outd(z);
            if z.b > 0 {
                z.pc -= 2;
            }
        } // otdr
        0x42 => {
            let result = get_bc(z);
            sbchl(z, result);
        } // sbc hl,bc
        0x52 => {
            let result = get_de(z);
            sbchl(z, result);
        } // sbc hl,de
        0x62 => {
            let result = get_hl(z);
            sbchl(z, result);
        } // sbc hl,hl
        0x72 => {
            sbchl(z, z.sp);
        } // sbc hl,sp
        0x4A => {
            let result = get_bc(z);
            adchl(z, result);
        } // adc hl,bc
        0x5A => {
            let result = get_de(z);
            adchl(z, result);
        } // adc hl,de
        0x6A => {
            let result = get_hl(z);
            adchl(z, result);
        } // adc hl,hl
        0x7A => {
            adchl(z, z.sp);
        } // adc hl,sp
        0x43 => {
            let addr: u16 = nextw(z);
            let result = get_bc(z);
            ww(z, addr, result);
            z.mem_ptr = addr + 1;
        } // ld (**), bc
        0x53 => {
            let addr: u16 = nextw(z);
            let result = get_de(z);
            ww(z, addr, result);
            z.mem_ptr = addr + 1;
        } // ld (**), de
        0x63 => {
            let addr: u16 = nextw(z);
            let result = get_hl(z);
            ww(z, addr, result);
            z.mem_ptr = addr + 1;
        } // ld (**), hl
        0x73 => {
            let addr: u16 = nextw(z);
            ww(z, addr, z.sp);
            z.mem_ptr = addr + 1;
        } // ld (**),sp
        0x4B => {
            let addr: u16 = nextw(z);
            let result = rw(z, addr);
            set_bc(z, result);
            z.mem_ptr = addr + 1;
        } // ld bc, (**)
        0x5B => {
            let addr: u16 = nextw(z);
            let result = rw(z, addr);
            set_de(z, result);
            z.mem_ptr = addr + 1;
        } // ld de, (**)
        0x6B => {
            let addr: u16 = nextw(z);
            let result = rw(z, addr);
            set_hl(z, result);
            z.mem_ptr = addr + 1;
        } // ld hl, (**)
        0x7B => {
            let addr: u16 = nextw(z);
            z.sp = rw(z, addr);
            z.mem_ptr = addr + 1;
        } // ld sp,(**)
        0x44 | 0x54 | 0x64 | 0x74 | 0x4C | 0x5C | 0x6C | 0x7C => {
            z.a = subb(z, 0, z.a, 0);
        } // neg
        0x46 | 0x66 => {
            z.interrupt_mode = 0;
        } // im 0
        0x56 | 0x76 => {
            z.interrupt_mode = 1;
        } // im 1
        0x5E | 0x7E => {
            z.interrupt_mode = 2;
        } // im 2
        0x67 => {
            let a: u8 = z.a;
            let result = get_hl(z);
            let val: u8 = rb(z, result);
            z.a = (a & 0xF0) | (val & 0xF);
            let result = get_hl(z);
            wb(z, result, (val >> 4) | (a << 4));

            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
            if z.a == 0 {
                z.zf = 1;
            } else {
                z.zf = 0;
            }

            z.sf = z.a >> 7;
            if parity(z.a) {
                z.pf = 1;
            } else {
                z.pf = 0;
            }
            z.mem_ptr = get_hl(z) + 1;
        } // rrd
        0x6F => {
            let a: u8 = z.a;
            let result = get_hl(z);
            let val: u8 = rb(z, result);
            z.a = (a & 0xF0) | (val >> 4);
            let result = get_hl(z);
            wb(z, result, (val << 4) | (a & 0xF));

            z.nf = 0;
            z.hf = 0;
            z.xf = GET_BIT(3, z.a);
            z.yf = GET_BIT(5, z.a);
            if z.a == 0 {
                z.zf = 1;
            } else {
                z.zf = 0;
            }
            z.sf = z.a >> 7;
            if parity(z.a) {
                z.pf = 1;
            } else {
                z.pf = 0;
            }

            z.mem_ptr = get_hl(z) + 1;
        } // rld

        _ => {
            println!("unknown ED opcode: {:02X}", opcode);
        }
    }
}

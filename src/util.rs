use crate::rgd::*;
use crate::op_def::*;
use num_traits::FromPrimitive;

pub fn to_rgd_op(op: u32) -> u32 {
  match op {
    DFSAN_BVEQ => RGD::Equal as u32,
    DFSAN_BVNEQ => RGD::Distinct as u32,
    DFSAN_BVSGT => RGD::Sgt as u32,
    DFSAN_BVSGE => RGD::Sge as u32,
    DFSAN_BVSLT => RGD::Slt as u32,
    DFSAN_BVSLE => RGD::Sle as u32,
    DFSAN_BVUGT => RGD::Ugt as u32,
    DFSAN_BVUGE => RGD::Uge as u32,
    DFSAN_BVULT => RGD::Ult as u32,
    DFSAN_BVULE => RGD::Ule as u32,
    _ => 0,
  }
}

pub fn do_print(node: &AstNode) {
  print!("{}(",node.get_name());
  print!("width={},",node.get_bits());
  print!("label={},",node.get_label());
  match FromPrimitive::from_u32(node.get_kind()) {
    Some(RGD::Bool) => print!("{}",node.get_value()),
    Some(RGD::Constant) => print!("{},",node.get_value()),
    Some(RGD::Memcmp) => print!("{},",node.get_value()),
    Some(RGD::Read) => print!("{},",node.get_index()),
    Some(RGD::Extract) => print!("{},",node.get_index()),
    _ => (),
  }
  for c in node.get_children() {
    do_print(c);
    print!(", ");
  }
  print!(")");
}

pub fn print_node(node: &AstNode) {
  do_print(node);
  println!("");
}

pub fn print_task(task: &SearchTask) {
  for cons in task.get_constraints() {
    println!("Op is {}", cons.get_comparison());
    print_node(cons.get_left());
    print_node(cons.get_right());
  }
}

#[inline(always)]
pub fn xxhash(h1: u32, h2: u32, h3: u32) -> u32 {
  //const PRIME32_1: u32 = 2654435761;
  const PRIME32_2: u32 = 2246822519;
  const PRIME32_3: u32 = 3266489917;
  const PRIME32_4: u32 =  668265263;
  const PRIME32_5: u32 =  374761393;

  let mut h32: u32 = PRIME32_5;
  h32 += h1 * PRIME32_3;
  h32  = (h32 << 17 | h32 >> 15)  * PRIME32_4;
  h32 += h2 * PRIME32_3;
  h32  = (h32 << 17 | h32 >> 15)  * PRIME32_4;
  h32 += h3 * PRIME32_3;
  h32  = (h32 << 17 | h32 >> 15)  * PRIME32_4;

  h32 ^= h32 >> 15;
  h32 *= PRIME32_2;
  h32 ^= h32 >> 13;
  h32 *= PRIME32_3;
  h32 ^= h32 >> 16;

  h32
}

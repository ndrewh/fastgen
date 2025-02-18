#[link(name = "gd")]
//#[link(name = "protobuf")]
//#[link(name = "LLVM")]
#[link(name = "stdc++")]
//#[link(name = "z3")]
extern "C" {
    pub fn init_core();
    pub fn qsym_filter(session: u64, addr: u64, direction: bool) -> bool;
    pub fn start_session() -> u64;
    pub fn end_session(s: u64);
    //pub fn get_next_input(input: *mut u8, addr: *mut u64, ctx: *mut u64,
    //      order: *mut u32, fid: *mut u32, direction: *mut u64,
    //      bid: *mut u32, sctx: *mut u32, is_cmp: *mut bool, predicate: *mut u32, target_cond: *mut u64, cons_hash: *mut u32, size: usize);
    //pub fn get_next_input_id() -> u32;
    //pub fn run_solver(shmid: i32, pipefd: i32);
    //pub fn insert_flip(addr: u64, ctx: u64, direction: u64, order: u32);
}

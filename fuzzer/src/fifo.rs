use nix::sys::stat;
use nix::unistd;
//use std::io;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::BufReader;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::{
    fs::File,
    io::{self, Read},
};

pub struct PipeMsg {
    pub msgtype: u32, //gep, cond, add_constraints, strcmp
    pub tid: u32,
    pub label: u32,
    pub result: u64, //direction for conditional branch, index for GEP
    pub addr: u64,
    pub ctx: u64,
    pub localcnt: u32,
    pub bid: u32,
    pub sctx: u32,
    pub predicate: u32,
    pub target_cond: u64,
}

impl PipeMsg {
    pub fn from_reader(mut rdr: impl Read) -> io::Result<Self> {
        let msgtype = rdr.read_u32::<LittleEndian>()?;
        let tid = rdr.read_u32::<LittleEndian>()?;
        let label = rdr.read_u32::<LittleEndian>()?;
        let result = rdr.read_u64::<LittleEndian>()?;
        let addr = rdr.read_u64::<LittleEndian>()?;
        let ctx = rdr.read_u64::<LittleEndian>()?;
        let localcnt = rdr.read_u32::<LittleEndian>()?;
        let bid = rdr.read_u32::<LittleEndian>()?;
        let sctx = rdr.read_u32::<LittleEndian>()?;
        let predicate = rdr.read_u32::<LittleEndian>()?;
        let target_cond = rdr.read_u64::<LittleEndian>()?;

        Ok(PipeMsg {
            msgtype,
            tid,
            label,
            result,
            addr,
            ctx,
            localcnt,
            bid,
            sctx,
            predicate,
            target_cond,
        })
    }
}

pub fn make_pipe() {
    match unistd::mkfifo("/tmp/wp", stat::Mode::S_IRWXU) {
        Ok(_) => println!("created"),
        Err(err) => println!("Error creating fifo: {}", err),
    }
}

/*
pub fn read_pipe(piped: RawFd) -> (Vec<(u32,u32,u64,u64,u64,u32,u32)>, VecDeque<[u8;1024]>) {
  let f = unsafe { File::from_raw_fd(piped) };
  let mut reader = BufReader::new(f);
  let mut ret = Vec::new();
  let mut retdata = VecDeque::new();
  loop {
    let mut buffer = String::new();
    let num_bytes = reader.read_line(&mut buffer).expect("read pipe failed");
    //if not EOF
    if num_bytes !=0  {
      let tokens: Vec<&str> = buffer.trim().split(',').collect();
      let tid = tokens[0].trim().parse::<u32>().expect("we expect u32 number in each line");
      let label = tokens[1].trim().parse::<u32>().expect("we expect u32 number in each line");
      let direction = tokens[2].trim().parse::<u64>().expect("we expect u32 number in each line");
      let addr = tokens[3].trim().parse::<u64>().expect("we expect u64 number in each line");
      let ctx = tokens[4].trim().parse::<u64>().expect("we expect u64 number in each line");
      let order = tokens[5].trim().parse::<u32>().expect("we expect u32 number in each line");
      let isgep = tokens[6].trim().parse::<u32>().expect("we expect u32 number in each line");
      ret.push((tid,label,direction,addr,ctx,order,isgep));
      if isgep == 2 {
        let mut buffer = String::new();
        let num_bytes = reader.read_line(&mut buffer).expect("read pipe failed");
        let size = label;
        let mut data = [0;1024];
        if num_bytes !=0 {
          let tokens: Vec<&str> = buffer.trim().split(',').collect();
          for i in 0..size as usize {
            data[i] = tokens[i].trim().parse::<u8>().expect("we expect u8");
          }
          retdata.push_back(data);
        } else {
          break;
        }
      }
    } else  {
      break;
    }
  }
  (ret,retdata)
}

*/
pub fn read_pipe(
    piped: RawFd,
) -> (
    Vec<(u32, u32, u64, u64, u64, u32, u32, u32, u32)>,
    VecDeque<Vec<u8>>,
) {
    let f = unsafe { File::from_raw_fd(piped) };
    let mut reader = BufReader::new(f);
    let mut ret = Vec::new();
    let mut retdata = VecDeque::new();
    loop {
        let msg = PipeMsg::from_reader(&mut reader);
        if let Ok(rawmsg) = msg {
            let tid = rawmsg.tid;
            let label = rawmsg.label;
            let direction = rawmsg.result;
            let addr = rawmsg.addr;
            let ctx = rawmsg.ctx;
            let isgep = rawmsg.msgtype;
            let order = rawmsg.localcnt;
            let bid = rawmsg.bid;
            let sctx = rawmsg.sctx;
            ret.push((tid, label, direction, addr, ctx, order, isgep, bid, sctx));
            if isgep == 2 {
                let mut data = Vec::new();
                for _i in 0..direction as usize {
                    if let Ok(cur) = reader.read_u8() {
                        data.push(cur);
                    } else {
                        break;
                    }
                }
                if data.len() < direction as usize {
                    break;
                }
                retdata.push_back(data);
            }
        } else {
            break;
        }
    }
    (ret, retdata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_pipe() {
        make_pipe()
    }

    #[test]
    fn test_read_pipe() {
        let (v, w) = read_pipe(2);
        println!("{:?}", v);
    }
}

use crate::depot_dir::*;
use crate::file::*;
use crate::status_type::StatusType;
use rand;
use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
// https://crates.io/crates/priority-queue

pub struct Depot {
    pub num_inputs: AtomicUsize,
    pub num_hangs: AtomicUsize,
    pub num_crashes: AtomicUsize,
    pub next_id: AtomicUsize,
    pub dirs: DepotDir,
}

impl Depot {
    pub fn new(in_dir: PathBuf, out_dir: &Path) -> Self {
        Self {
            num_inputs: AtomicUsize::new(0),
            num_hangs: AtomicUsize::new(0),
            num_crashes: AtomicUsize::new(0),
            next_id: AtomicUsize::new(0),
            dirs: DepotDir::new(in_dir, out_dir),
        }
    }

    fn save_input(status: &StatusType, buf: &Vec<u8>, num: &AtomicUsize, dir: &Path) -> usize {
        let mut id = num.load(Ordering::Acquire);
        trace!("Find {} th new {:?} input", id, status,);
        let new_path = get_file_name(dir, id);
        let mut f = fs::File::create(new_path.as_path()).expect("Could not save new input file.");
        f.write_all(buf)
            .expect("Could not write seed buffer to file.");
        f.flush().expect("Could not flush file I/O.");
        id = id + 1;
        num.store(id, Ordering::Release);
        id
    }

    pub fn save(&self, status: StatusType, buf: &Vec<u8>) -> usize {
        match status {
            StatusType::Normal => {
                Self::save_input(&status, buf, &self.num_inputs, &self.dirs.inputs_dir)
            }
            StatusType::Timeout => {
                Self::save_input(&status, buf, &self.num_hangs, &self.dirs.hangs_dir)
            }
            StatusType::Crash => {
                Self::save_input(&status, buf, &self.num_crashes, &self.dirs.crashes_dir)
            }
            _ => 0,
        }
    }

    pub fn empty(&self) -> bool {
        self.num_inputs.load(Ordering::Relaxed) == 0
    }

    pub fn next_random(&self) -> usize {
        rand::random::<usize>() % self.num_inputs.load(Ordering::Relaxed)
    }

    pub fn get_input_buf(&self, id: usize) -> Option<Vec<u8>> {
        let path = get_file_name(&self.dirs.inputs_dir, id);
        read_from_file(&path)
    }

    pub fn get_input_path(&self, id: usize) -> PathBuf {
        get_file_name(&self.dirs.inputs_dir, id)
    }

    pub fn get_num_inputs(&self) -> usize {
        self.num_inputs.load(Ordering::Relaxed)
    }

    pub fn get_next_input(&self) -> Option<usize> {
        let cur_id = self.next_id.load(Ordering::Relaxed);
        if cur_id < self.get_num_inputs() {
            let retval = self.next_id.compare_exchange(cur_id, cur_id + 1, Ordering::Relaxed, Ordering::Relaxed).ok();

            if let Some(id) = retval {
                let mut progress = Vec::new();
                progress.write_u32::<LittleEndian>(id as u32).unwrap();
                std::fs::write("ce_progress", &progress)
                    .map_err(|err| println!("{:?}", err))
                    .ok();
            }
            return retval;
        }
        None
    }

    pub fn load_cached_next_id(&self) {
        let progress_data = std::fs::read("ce_progress").unwrap();
        let id: u32 = (&progress_data[..]).read_u32::<LittleEndian>().unwrap();
        println!("restarting scan from id {}", id);
        self.next_id.store(id as usize, Ordering::Relaxed);
    }
}

#[macro_use]
extern crate clap;
use clap::{App, Arg};

//extern crate angora;
//extern crate angora_common;
use fastgen::fuzz_main::*;

fn main() {
    let matches = App::new("angora-fuzzer")
        .version(crate_version!())
        .about("Fastgen is a mutation-based fuzzer.")
        .arg(Arg::with_name("input_dir")
             .short("i")
             .long("input")
             .value_name("DIR")
             .help("Sets the directory of input seeds, use \"-\" to restart with existing output directory")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("output_dir")
             .short("o")
             .long("output")
             .value_name("DIR")
             .help("Sets the directory of outputs")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("track_target")
             .short("t")
             .long("track")
             .value_name("PROM")
             .help("Sets the target (USE_TRACK or USE_PIN) for tracking, including taints, cmps.  Only set in LLVM mode.")
             .takes_value(true))
        .arg(Arg::with_name("pargs")
            .help("Targeted program (USE_FAST) and arguments. Any \"@@\" will be substituted with the input filename from Angora.")
            .required(true)
            .multiple(true)
            .allow_hyphen_values(true)
            .last(true)
            .index(1))
        .arg(Arg::with_name("memory_limit")
             .short("M")
             .long("memory_limit")
             .value_name("MEM")
             .help("Memory limit for programs, default is 200(MB), set 0 for unlimit memory")
             .takes_value(true))
        .arg(Arg::with_name("time_limit")
             .short("T")
             .long("time_limit")
             .value_name("TIME")
             .help("time limit for programs, default is 1(s), the tracking timeout is 12 * TIME")
             .takes_value(true))
        .arg(Arg::with_name("thread_jobs")
             .short("j")
             .long("jobs")
             .value_name("JOB")
             .help("Sets the number of thread jobs, default is 1")
             .takes_value(true))
        .arg(Arg::with_name("grader_jobs")
             .short("g")
             .long("graders")
             .value_name("Graders")
             .help("Sets the number of grader jobs, default is 1")
             .takes_value(true))
        .arg(Arg::with_name("executor_timeout")
             .long("executor_timeout")
             .value_name("EXECUTOR_TIMEOUT")
             .help("Timeout in seconds for the executor")
             .takes_value(true))
        .arg(Arg::with_name("solver_timeout")
             .long("solver_timeout")
             .value_name("SOLVER_TIMEOUT")
             .help("Timeout in seconds for the solver")
             .takes_value(true))
        .arg(Arg::with_name("sync_afl")
             .short("S")
             .long("sync_afl")
             .help("Sync the seeds with AFL. Output directory should be in AFL's directory structure."))
       .get_matches();

    fuzz_main(
        matches.value_of("input_dir").unwrap(),
        matches.value_of("output_dir").unwrap(),
        matches.value_of("track_target").unwrap_or("-"),
        matches.values_of_lossy("pargs").unwrap(),
        value_t!(matches, "thread_jobs", usize).unwrap_or(1),
        value_t!(matches, "grader_jobs", usize).unwrap_or(1),
        value_t!(matches, "memory_limit", u64).unwrap_or(fastgen_common::config::MEM_LIMIT),
        value_t!(matches, "time_limit", u64).unwrap_or(fastgen_common::config::TIME_LIMIT),
        matches.occurrences_of("sync_afl") > 0,
        value_t!(matches, "executor_timeout", usize).unwrap_or(120) as u64,
        value_t!(matches, "solver_timeout", usize).unwrap_or(10) as u64,
    );
}

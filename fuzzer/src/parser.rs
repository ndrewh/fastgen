use crate::rgd::*;
use crate::util::*;
use std::collections::HashMap;
use std::path::Path;
use crate::gd::*;
use crate::task::Cons;
use crate::task::Fut;
use crate::jit::JITEngine;
use crate::solution::Solution;
use blockingqueue::BlockingQueue;
use std::time;
use inkwell::execution_engine::JitFunction;
use crate::jit::JigsawFnType;
use std::hash::{Hash, Hasher};

static mut gengine: Option<JITEngine> = None;
static mut gfuncache: Option<HashMap<AstNode, JitFunction<JigsawFnType>>> = None;

impl Eq for AstNode {
}

impl Hash for AstNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_hash().hash(state);
    }
}


pub struct SearchTaskBuilder {
  pub per_session_cache: HashMap<u32, Constraint>,  
  pub last_fid: u32,
  pub func_cache: HashMap<AstNode, u64>, 
}

impl SearchTaskBuilder {
  pub fn new() -> Self {
    let cache = HashMap::new();   
    let fcache = HashMap::new();   
    Self {
          per_session_cache: cache, 
          last_fid: std::u32::MAX,
          func_cache: fcache,
        }
  }

  pub fn construct_task<'a>(&mut self, task: &SearchTask, engine: &'a JITEngine, 
                        fun_cache: &'a mut HashMap<AstNode, JitFunction<'a, JigsawFnType>>) -> Fut<'a> {
  //pub fn construct_task(&mut self, task: &SearchTask, engine: &JITEngine) -> Fut {
    let mut fut = Fut::new();
    if task.get_fid() != self.last_fid {
      //a new seed
      self.per_session_cache.clear();
      self.last_fid = task.get_fid(); 
    }
    let mut con_index = 0;
    for cons in task.get_constraints() {
      let mut constraint;
      if con_index == 0 {
        constraint = cons.clone();
        self.per_session_cache.insert(cons.get_label(), constraint.clone());
      } else {
        constraint = self.per_session_cache[&cons.get_label()].clone();
      }
      con_index += 1;
      let mut cons = Cons::new();
      //TODO we do not transfer information using protobuf anymore
      self.append_meta(&mut cons, &constraint); 
      let t_start = time::Instant::now();
      //let mut x = vec![1, 1, 1, 1, 12350, 15, 16, 17, 18, 19];
      //unsafe { println!("result is {}, left {} right {}", cons.call_func(&mut x), x[0], x[1]); }
      if !fun_cache.contains_key(&constraint.get_node()) {
        let fun = engine.add_function(&constraint.get_node(), &cons.local_map);
        println!("miss and jitime is {}", t_start.elapsed().as_micros());
        fun_cache.insert(constraint.get_node().clone(), fun.clone());
        cons.set_func(fun);
      } else {
        let fun = fun_cache[&constraint.get_node()].clone();
        cons.set_func(fun);
        println!("hit and jitime is {}", t_start.elapsed().as_micros());
      }
      fut.constraints.push(cons);
    }
    fut.finalize();
    fut
  }

  pub fn append_meta(&self, cons: &mut Cons, constraint: &Constraint) {
    for amap in constraint.get_meta().get_map() {
      cons.local_map.insert(amap.get_k(), amap.get_v());
    }
    for aarg in constraint.get_meta().get_args() {
      cons.input_args.push((aarg.get_isinput(), aarg.get_v()));
    }
    for ainput in constraint.get_meta().get_inputs() {
      cons.inputs.insert(ainput.get_offset(), ainput.get_iv() as u8);
    }
    for ashape in constraint.get_meta().get_shape() {
      cons.shape.insert(ashape.get_offset(), ashape.get_start());
    }
    cons.comparison = constraint.get_node().get_kind();
    cons.const_num = constraint.get_meta().get_const_num();
  }

  pub fn submit_task_rust(&mut self, task: &SearchTask, 
      solution_queue: BlockingQueue<Solution>,
      solve: bool) {
    
/*
       info!("print task number of children is {} fid {}",task.get_constraints().len(), task.get_fid());
       print_task(task);
       let r = save_request(task, &Path::new("saved_test"));
       if r.is_err() {
       debug!("save error");
       }
     */    
    if !solve {
      if task.get_fid() != self.last_fid {
        //a new seed
        self.per_session_cache.clear();
        self.last_fid = task.get_fid(); 
      }
      self.per_session_cache.insert(task.get_constraints()[0].get_label(), 
          task.get_constraints()[0].clone());

      return;
    }
    unsafe {
      if gengine.is_none() {
        gengine = Some(JITEngine::new());
      }
      if gfuncache.is_none() {
        gfuncache = Some(HashMap::new());
      }
      let sengine = gengine.as_ref().unwrap();
      let sfuncache = gfuncache.as_mut().unwrap();
      let mut fut = self.construct_task(task, sengine, sfuncache);
      gd_search(&mut fut);
      for sol in fut.rgd_solutions {
        let sol_size = sol.len();
        let rgd_sol = Solution::new(sol, task.get_fid(), task.get_addr(), task.get_ctx(), 
            task.get_order(), task.get_direction(), 0, sol_size);
        solution_queue.push(rgd_sol);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::rgd::*;
  use crate::util::*;
  use crate::parser::*;
  use std::path::Path;
  use crate::gd::*;
  use crate::task::SContext;
  use std::collections::HashMap;
#[test]
  fn test_load() {
    let tasks: Vec<SearchTask> = load_request(Path::new("saved_test")).expect("ok");
    let mut tb = SearchTaskBuilder::new();
    let engine = JITEngine::new();
    let mut funcache = HashMap::new();
    for task in tasks { let task_copy = task.clone();
      print_task(&task_copy);
      let mut fut = tb.construct_task(&task_copy, &engine, &mut funcache);
      println!("search!");
      gd_search(&mut fut);
      for sol in fut.rgd_solutions {
        for (k,v) in sol.iter() {
          println!("k {} v {}", k, v);
        }
      }
    }
  }
#[test]  
  fn test_input() {
    let mut ctx = SContext::new(2,2,4);
    ctx.min_input.value.push(1);
    let mut input = &mut ctx.min_input;
    let mut scratch_input = input.clone();
    scratch_input.set(0,2);
    *input = scratch_input;
    println!("{}",ctx.min_input.get(0));
  }
}

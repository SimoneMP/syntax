extern crate core;
extern crate time;
extern crate rand;
extern crate mech_syntax;
extern crate mech_core;

use std::time::SystemTime;
use std::thread::{self};
use std::time::*;
use rand::{Rng, thread_rng};
use mech_syntax::lexer::Lexer;
use mech_syntax::parser::{Parser, ParseStatus, Node};
use mech_syntax::compiler::Compiler;
use mech_core::Block;
use mech_core::{Change, Transaction};
use mech_core::{Value};
use mech_core::Hasher;
use mech_core::Core;

fn main() {
  let mut compiler = Compiler::new();
  let mut core = Core::new(100000, 50000);
  let input = String::from("# Header
block 
  a = 7 + 8
  #test = [x y z
           1 4 5
           0 a 0
           7 9 8]");
  compiler.compile_string(input);
  core.register_blocks(compiler.blocks.clone());
  //println!("{:?}", compiler.parse_tree);
  println!("{:?}", compiler.syntax_tree);
  core.step();
  println!("{:?}", core);
  println!("{:?}", core.runtime); 

}   
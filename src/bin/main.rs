use mech_syntax::parser::Parser;
use mech_syntax::ast::Ast;
use mech_syntax::compiler::Compiler;
use mech_core::{Core,MechError};

use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(),MechError> {

  let mut parser = Parser::new();
  let mut ast = Ast::new();
  let mut compiler = Compiler::new();
  let mut core = Core::new();

  parser.parse(r#"
block
  #test = #app{2}{2}{1}

block
  #app = [1 [2 [12 3]]]"#);

  //println!("{:#?}", parser.parse_tree);

  ast.build_syntax_tree(&parser.parse_tree);

  println!("{:?}", ast.syntax_tree);

  let blocks = compiler.compile_blocks(&vec![ast.syntax_tree.clone()]).unwrap();

  for block in blocks {
    match core.insert_block(Rc::new(RefCell::new(block.clone()))) {
      Ok(()) => (),
      Err(mech_error) => println!("ERROR: {:?}", mech_error),
    }
  }
  
  /*for t in blocks {
    println!("{:#?}", t);
  }*/

  println!("{:#?}", core);

  println!("{:#?}", core.get_table("test"));

  Ok(())
}
/*
├Root
│├Program(None)
││├Section(None)
│││├Paragraph
││││├ParagraphText(['b', 'l', 'o', 'c', 'k'])
│││├Block
││││├Transformation
│││││├Statement
││││││├VariableDefine
│││││││├Identifier(['y'](uly-min-oct-eas))
│││││││├Expression
││││││││├NumberLiteral([3])
││││├Transformation
│││││├Statement
││││││├VariableDefine
│││││││├Identifier(['x'](ska-pri-oct-osc))
│││││││├Expression
││││││││├AnonymousTableDefine
│││││││││├TableRow
││││││││││├TableColumn
│││││││││││├Expression
││││││││││││├NumberLiteral([10])
││││││││││├TableColumn
│││││││││││├Expression
││││││││││││├AnonymousTableDefine
│││││││││││││├TableRow
││││││││││││││├TableColumn
│││││││││││││││├Expression
││││││││││││││││├NumberLiteral([3])*/
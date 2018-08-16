// # Mech Syntax Compiler

// ## Preamble

use mech_core::{Block, Constraint};
use mech_core::{Function, Plan, Comparator};
use mech_core::Hasher;
use parser;
use lexer::Lexer;
use parser::{Parser, ParseStatus};
use lexer::Token;
use alloc::{String, Vec, fmt};

// ## Compiler Nodes

#[derive(Clone, PartialEq)]
pub enum Node {
  Root{ children: Vec<Node> },
  Fragment{ children: Vec<Node> },
  Program{ children: Vec<Node> },
  Head{ children: Vec<Node> },
  Body{ children: Vec<Node> },
  Section{ children: Vec<Node> },
  Block{ children: Vec<Node> },
  Statement{ children: Vec<Node> },
  Expression{ children: Vec<Node> },
  MathExpression{ children: Vec<Node> },
  FilterExpression{ children: Vec<Node> },
  SelectExpression{ children: Vec<Node> },
  Data{ children: Vec<Node> },
  DataWatch{ children: Vec<Node> },
  SelectData{ children: Vec<Node> },
  SetData{ children: Vec<Node> },
  RowDefine{ children: Vec<Node> },
  Column{ children: Vec<Node> },
  Binding{ children: Vec<Node> },
  Function{ name: String, children: Vec<Node> },
  Define { name: String, id: u64},
  DotIndex { rows: Vec<Node>, columns: Vec<Node>},
  BracketIndex { rows: Vec<Node>, columns: Vec<Node>},
  ColumnDefine {children: Vec<Node> },
  TableDefine {children: Vec<Node> },
  AddRow {children: Vec<Node> },
  Constraint{ children: Vec<Node> },
  Title{ text: String },
  Identifier{ name: String, id: u64 },
  Table{ name: String, id: u64 },
  Paragraph{ text: String },
  Constant {value: u64},
  String{ text: String },
  Token{ token: Token, byte: u8 },
  Null,
}

impl fmt::Debug for Node {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    print_recurse(self, 0);
    Ok(())
  }
}

pub fn print_recurse(node: &Node, level: usize) {
  spacer(level);
  let children: Option<&Vec<Node>> = match node {
    Node::Root{children} => {print!("Root\n"); Some(children)},
    Node::Fragment{children} => {print!("Fragment\n"); Some(children)},
    Node::Program{children} => {print!("Program\n"); Some(children)},
    Node::Head{children} => {print!("Head\n"); Some(children)},
    Node::Body{children} => {print!("Body\n"); Some(children)},
    Node::ColumnDefine{children} => {print!("ColumnDefine\n"); Some(children)},
    Node::RowDefine{children} => {print!("RowDefine\n"); Some(children)},
    Node::Column{children} => {print!("Column\n"); Some(children)},
    Node::Binding{children} => {print!("Binding\n"); Some(children)},
    Node::TableDefine{children} => {print!("TableDefine\n"); Some(children)},
    Node::AddRow{children} => {print!("AddRow\n"); Some(children)},
    Node::Section{children} => {print!("Section\n"); Some(children)},
    Node::Block{children} => {print!("Block\n"); Some(children)},
    Node::Statement{children} => {print!("Statement\n"); Some(children)},
    Node::SetData{children} => {print!("SetData\n"); Some(children)},
    Node::Data{children} => {print!("Data\n"); Some(children)},
    Node::DataWatch{children} => {print!("DataWatch\n"); Some(children)},
    Node::SelectData{children} => {print!("SelectData\n"); Some(children)},
    Node::DotIndex{rows, columns} => {print!("DotIndex[rows: {:?}, columns: {:?}]\n", rows, columns); None},
    Node::BracketIndex{rows, columns} => {print!("BracketIndex[rows: {:?}, columns: {:?}]\n", rows, columns); None},
    Node::Expression{children} => {print!("Expression\n"); Some(children)},
    Node::Function{name, children} => {print!("Function({:?})\n", name); Some(children)},
    Node::MathExpression{children} => {print!("MathExpression\n"); Some(children)},
    Node::SelectExpression{children} => {print!("SelectExpression\n"); Some(children)},
    Node::FilterExpression{children} => {print!("FilterExpression\n"); Some(children)},
    Node::Constraint{children} => {print!("Constraint\n"); Some(children)},
    Node::Identifier{name, id} => {print!("{}({:?})\n", name, id); None},
    Node::String{text} => {print!("String({:?})\n", text); None},
    Node::Title{text} => {print!("Title({:?})\n", text); None},
    Node::Constant{value} => {print!("{:?}\n", value); None},
    Node::Paragraph{text} => {print!("Paragraph({:?})\n", text); None},
    Node::Table{name,id} => {print!("#{}({:?})\n", name, id); None},
    Node::Define{name,id} => {print!("Define #{}({:?})\n", name, id); None},
    Node::Token{token, byte} => {print!("Token({:?})\n", token); None},
    Node::Null => {print!("Null\n"); None},
    _ => {print!("Unhandled Node"); None},
  };  
  match children {
    Some(childs) => {
      for child in childs {
        print_recurse(child, level + 1)
      }
    },
    _ => (),
  }    
}

pub fn spacer(width: usize) {
  let limit = if width > 0 {
    width - 1
  } else {
    width
  };
  for _ in 0..limit {
    print!("│");
  }
  print!("├");
}

// ## Compiler

#[derive(Debug)]
pub struct Compiler {
  pub blocks: Vec<Block>,
  pub constraints: Vec<Constraint>,
  pub depth: usize,
  pub input_registers: usize,
  pub memory_registers: usize,
  pub output_registers: usize,
  pub parse_tree: parser::Node,
  pub syntax_tree: Node,
  pub node_stack: Vec<Node>, 
  pub section: usize,
  pub block: usize,
}

impl Compiler {

  pub fn new() -> Compiler {
    Compiler {
      blocks: Vec::new(),
      constraints: Vec::new(),
      node_stack: Vec::new(),
      depth: 0,
      section: 1,
      block: 1,
      input_registers: 1,
      memory_registers: 1,
      output_registers: 1,
      parse_tree: parser::Node::Root{ children: Vec::new() },
      syntax_tree: Node::Root{ children: Vec::new() },
    }
  }

  pub fn compile_string(&mut self, input: String) -> &Vec<Block> {
    let mut lexer = Lexer::new();
    let mut parser = Parser::new();
    lexer.add_string(input.clone());
    let tokens = lexer.get_tokens();
    parser.text = input;
    parser.add_tokens(&mut tokens.clone());
    parser.build_parse_tree();
    self.parse_tree = parser.parse_tree.clone();
    self.build_syntax_tree(parser.parse_tree);
    let ast = self.syntax_tree.clone();
    self.compile_blocks(ast);
    &self.blocks
  }

  pub fn compile_blocks(&mut self, node: Node) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    match node {
      Node::Block{children} => {
        let mut block = Block::new();
        block.name = format!("{:?},{:?}", self.section, self.block);
        block.id = Hasher::hash_string(block.name.clone()) as usize;
        self.block += 1;
        self.input_registers = 1;
        self.memory_registers = 1;
        self.output_registers = 1;
        let constraints = self.compile_constraints(&children);
        block.add_constraints(constraints);
        block.plan();
        blocks.push(block);
      },
      Node::Root{children} => {
        let result = self.compile_children(children);
        self.blocks = result;
      },
      Node::Program{children} => {blocks.append(&mut self.compile_children(children));},
      Node::Fragment{children} => {
        let mut block = Block::new();
        block.name = format!("{:?},{:?}", self.section, self.block);
        block.id = Hasher::hash_string(block.name.clone()) as usize;
        self.block += 1;
        self.input_registers = 1;
        self.memory_registers = 1;
        self.output_registers = 1;
        let constraints = self.compile_constraints(&children);
        block.add_constraints(constraints);
        block.plan();
        blocks.push(block);
      },
      Node::Body{children} => {blocks.append(&mut self.compile_children(children));},
      Node::Section{children} => {
        blocks.append(&mut self.compile_children(children));
        self.section += 1;
        self.block = 1;
      },
      _ => (),
    }
    blocks
  }

  pub fn compile_children(&mut self, nodes: Vec<Node>) -> Vec<Block> {
    let mut compiled = Vec::new();
    for node in nodes {
      compiled.append(&mut self.compile_blocks(node));
    }
    compiled
  }

  pub fn compile_constraint(&mut self, node: &Node) -> Vec<Constraint> {
    let mut constraints: Vec<Constraint> = Vec::new();
    match node {
      Node::Constraint{children} |
      Node::Statement{children} |
      Node::Expression{children} => {
        constraints.append(&mut self.compile_constraints(children));
      },
      Node::MathExpression{children} => {
        let m = self.memory_registers as u64;
        let mut result = self.compile_constraints(children);
        constraints.push(Constraint::Data{table: 0, column: m});
        constraints.append(&mut result);
      },
      Node::SetData{children} => {
        let mut table_id = 0;
        let mut column_id = 1;
        let mut output_column = 0;
        let m = self.memory_registers as u64;
        let o = self.output_registers as u64;

        let mut lhs_constraints = self.compile_constraint(&children[0]);
        let mut rhs_constraints = self.compile_constraint(&children[1]);
        lhs_constraints.reverse();
        rhs_constraints.reverse();

        let table = lhs_constraints.pop();
        let output = rhs_constraints.pop();

        lhs_constraints.reverse();
        rhs_constraints.reverse();

        match table {
          Some(Constraint::Data{table: t, column: c}) => {
            table_id = t;
            column_id = c;
          },
          _ => (), 
        }
        
        match output {
          Some(Constraint::Data{table: 0, column: c}) => {
            output_column = c;
          },
          _ => (),
        }

        // If there is an index:
        for index_mask in lhs_constraints {
          match index_mask {
            Constraint::IndexMask{source, truth, memory} => {
              constraints.push(Constraint::IndexMask{ source: output_column, truth, memory});
              output_column = memory;
            }
            _ => (),
          }
        }
        constraints.push(Constraint::Insert{memory: output_column, table: table_id, column: column_id});
        self.output_registers += 1;
        constraints.append(&mut rhs_constraints);
      },
      Node::DataWatch{children} => {
       let result = self.compile_constraints(children);
        for constraint in result {
          match constraint {
            Constraint::Data{table, column} => {
              constraints.push(Constraint::ChangeScan {table, column, input: self.input_registers as u64});
              self.input_registers += 1;
            },
            _ => (),
          }
        }
      }
      Node::RowDefine{children} => {
        let m = self.memory_registers;
        let mut result = self.compile_constraints(children);
        // Assign the column
        let mut column_ix = 1;
        for constraint in result {
          match constraint {
            Constraint::Identifier{id, memory} => {
              column_ix = id;
              constraints.push(constraint);
            },
            Constraint::Insert{memory, table, column} => {
              constraints.push(Constraint::Insert{memory, table, column: column_ix});
            },
            _ => constraints.push(constraint),
          }
        }
      },
      Node::Column{children} => {
        let mut result = self.compile_constraints(children);
        for constraint in result {
          match constraint {
            Constraint::Data{table, column} => {
              constraints.push(Constraint::Identifier{id: column, memory: self.memory_registers as u64 - 1 });
            },
            _ => {
              constraints.push(constraint)
            }, 
          }
        }
      },
      Node::Binding{children} => {
        constraints.push(Constraint::Insert{memory: self.memory_registers as u64, table: 0, column: 0});
        self.output_registers += 1;
        constraints.append(&mut self.compile_constraints(children));
      },
      Node::Data{children} => {
        let mut row = 1;
        let mut column = 1;
        let mut table = 0;
        let mut data: Vec<Constraint> = Vec::new();
        for child in children {
          match child {
            Node::Table{name, id} => table = *id,
            Node::DotIndex{rows, columns} => {
              for column in columns {
                match column {
                  Node::Identifier{name, id} => data.push(Constraint::Data{table, column: *id}),
                  Node::BracketIndex{rows, columns} => {
                    for column in columns {
                      match column {
                        Node::Identifier{name, id} => {
                          data.push(Constraint::IndexMask{ source: 0, truth: *id, memory: self.memory_registers as u64});
                          self.memory_registers += 1;
                        },
                        _ => (),
                      }
                    }
                  },
                  _ => (),
                }
              }
            }
            _ => constraints.append(&mut self.compile_constraints(children)),
          }
        };
        // If there is no index, we just take the first column for now later we'll take the whole table
        if data.len() == 0 {
          data.push(Constraint::Data{table, column: 1})
        }
        data.reverse();
        constraints.append(&mut data);
      },
      Node::ColumnDefine{children} => {
        let m = self.memory_registers as u64;
        let mut result = self.compile_constraints(children);
        result.reverse();
        let identifier = result.pop();
        result.reverse();
        match identifier {
          Some(Constraint::Data{table, column}) => {
            constraints.push(Constraint::Identifier{id: column, memory: m});
          },
          _ => (),
        }
        constraints.append(&mut result);
      },
      Node::TableDefine{children} => {
        let mut table_id = 0;
        let m = self.memory_registers as u64;
        let o = self.output_registers as u64;
        let mut result = self.compile_constraints(children);
        result.reverse();
        let table = result.pop();
        result.reverse();
        // Create the new table
        match table {
          Some(Constraint::Data{table: t, ..}) => {
            table_id = t;
          },
          _ => (), 
        }
        // Assign the table
        let mut column_ix = 1;
        for constraint in result {
          match constraint {
            Constraint::Insert{memory, table, column} => {
              constraints.push(Constraint::Insert{memory, table: table_id, column});
            },
            Constraint::Data{table: 0, column} => {
              constraints.push(Constraint::Insert{table: table_id, column: 1, memory: column});
              self.output_registers += 1;
            },
            _ => constraints.push(constraint),
          }
        }
        let columns = self.memory_registers as u64 - m;
        constraints.push(Constraint::NewTable{id: table_id, rows: 1, columns});
      },
      Node::AddRow{children} => {
        let mut table_id = 0;
        let m = self.memory_registers as u64;
        let o = self.output_registers as u64;
        let mut result = self.compile_constraints(children);
        result.reverse();
        let table = result.pop();
        result.reverse();
        // Create the new table
        match table {
          Some(Constraint::Data{table: t, ..}) => {
            table_id = t;
          },
          _ => (), 
        }
        // Assign the table
        let mut column_ix = 1;
        for constraint in result {
          match constraint {
            Constraint::Insert{memory, table, column} => {
              constraints.push(Constraint::Append{memory, table: table_id, column});
            },
            _ => constraints.push(constraint),
          }
        }
      },
      Node::FilterExpression{children} => {   
        let m = self.memory_registers as u64;
        self.memory_registers += 1;
        // Get the comparator. One of: > < != =
        let comparator_node = &children[1];
        let comparator: Comparator = match comparator_node {
          Node::Token{token: Token::GreaterThan, ..} => Comparator::GreaterThan,
          Node::Token{token: Token::LessThan, ..} => Comparator::LessThan,
          _ => Comparator::Equal,
        };
        let mut lhs_constraints = self.compile_constraint(&children[0]);
        let lhs = match &lhs_constraints[0] {
          Constraint::Function{operation, parameters, memory} => *memory,
          Constraint::Constant{value, memory} => *memory,
          Constraint::CopyInput{input, memory} => *memory,
          Constraint::Scan{..} => {
            match &lhs_constraints[1] {
              Constraint::CopyInput{input, memory} => *memory,
              _ => 0,
            }
          },
          _ => 0,
        };
        let mut rhs_constraints = self.compile_constraint(&children[2]);
        let rhs = match &rhs_constraints[0] {
          Constraint::Function{operation, parameters, memory} => *memory,
          Constraint::Constant{value, memory} => *memory,
          Constraint::CopyInput{input, memory} => *memory,
          Constraint::Scan{..} => {
            match &rhs_constraints[1] {
              Constraint::CopyInput{input, memory} => *memory,
              _ => 0,
            }
          },
          _ => 0,
        };
        constraints.push(Constraint::Filter{comparator, lhs, rhs, memory: m});
        constraints.append(&mut lhs_constraints);
        constraints.append(&mut rhs_constraints);
      },
      Node::Function{name, children} => {   
        let operation = match name.as_ref() {
          "+" => Function::Add,
          "-" => Function::Subtract,
          "*" => Function::Multiply,
          "/" => Function::Divide,
          _ => Function::Add,
        };
        let o1 = self.memory_registers as u64;
        self.memory_registers += 1;
        let p1 = self.memory_registers as u64;
        let mut result = self.compile_constraints(children);
        if result.len() >= 2 {
          let p2 = match &result[result.len() - 1] {
            Constraint::Function{operation, parameters, memory} => *memory,
            Constraint::Constant{value, memory} => *memory,
            Constraint::CopyInput{input, memory} => *memory,
            _ => 0,
          };
          constraints.append(&mut result);
          constraints.push(Constraint::Function{operation, parameters: vec![p1, p2], memory: o1});
        }
      },
      Node::SelectExpression{children} => {
        let m = self.memory_registers as u64;
        let mut result = self.compile_constraints(children);
        for constraint in result {
          match constraint {
            Constraint::Data{table, column} => {
              let input = self.input_registers as u64;
              let memory = self.memory_registers as u64;
              self.input_registers += 1;
              self.memory_registers += 1;
              constraints.push(Constraint::Data{table: 0, column: m});
              constraints.push(Constraint::Scan{table, column, input});
              constraints.push(Constraint::CopyInput{input, memory});
            },
            _ => constraints.push(constraint),
          }
        }
      },
      Node::SelectData{children} => {
        let mut result = self.compile_constraints(children);
        for constraint in result {
          match constraint {
            Constraint::Data{table, column} => {
              let input = self.input_registers as u64;
              let memory = self.memory_registers as u64;
              self.input_registers += 1;
              self.memory_registers += 1;
              constraints.push(Constraint::Scan{table, column, input});
              constraints.push(Constraint::CopyInput{input, memory});
            },
            _ => constraints.push(constraint),
          }
        }
      },
      Node::Identifier{name, id} => {
        constraints.push(Constraint::Data{table: 0, column: *id});
      },
      Node::Table{name, id} => {
        constraints.push(Constraint::Data{table: *id, column: 1});
      },
      Node::Constant{value} => {
        constraints.push(Constraint::Constant{value: *value as i64, memory: self.memory_registers as u64});
        self.memory_registers += 1;
      },
      _ => (),
    }
    constraints
  }

  pub fn compile_constraints(&mut self, nodes: &Vec<Node>) -> Vec<Constraint> {
    let mut compiled = Vec::new();
    for node in nodes {
      compiled.append(&mut self.compile_constraint(node));
    }
    compiled
  }

  pub fn build_syntax_tree(&mut self, node: parser::Node) -> Vec<Node> {
    let mut compiled = Vec::new();
    self.depth += 1;
    match node {
      parser::Node::Root{children} => {
        let result = self.compile_nodes(children);
        self.syntax_tree = Node::Root{children: result};        
      },
      parser::Node::Fragment{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Fragment{children: result});
      },
      parser::Node::Program{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Program{children: result});
      },
      parser::Node::Head{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Head{children: result});
      },
      parser::Node::Body{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Body{children: result});
      },
      parser::Node::Section{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Section{children: result});
      },
      parser::Node::Block{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Block{children: result});
      },
      parser::Node::Data{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Data{children: result});
      },
      parser::Node::SelectData{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::SelectData{children: result});
      },
      parser::Node::TableDefineRHS{children} => {
        let mut result = self.compile_nodes(children);
        compiled.append(&mut result);
      },
      parser::Node::Statement{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Statement{children: result});
      },
      parser::Node::Expression{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::Expression{children: result});
      },
      parser::Node::DataWatch{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::DataWatch{children: result});
      },
      parser::Node::RowDefine{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::RowDefine{children});
      },
      parser::Node::SetData{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::SetData{children});
      },
      parser::Node::Column{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::Column{children});
      },
      parser::Node::Binding{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::Binding{children});
      },
      parser::Node::Constraint{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            // Ignore irrelevant nodes like spaces and operators
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::Constraint{children});
      },
      parser::Node::SelectExpression{children} => {
        let result = self.compile_nodes(children);
        compiled.push(Node::SelectExpression{children: result});
      },
      parser::Node::FilterExpression{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{token: Token::Space, ..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::FilterExpression{children});
      },
      parser::Node::MathExpression{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            // Ignore irrelevant nodes like spaces and operators
            Node::Token{..} => (), 
            _ => children.push(node),
          }
        }
        compiled.push(Node::MathExpression{children});
      },
      parser::Node::Infix{children} => {
        let result = self.compile_nodes(children);
        let operator = &result[0];
        let name: String = match operator {
          Node::Token{token, byte} => byte_to_char(*byte).unwrap().to_string(),
          _ => String::from(""),
        };
        compiled.push(Node::Function{name, children: vec![]});
      },
      parser::Node::ColumnDefine{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (),
            _ => children.push(node),
          }
        }
        compiled.push(Node::ColumnDefine{children});
      },
      parser::Node::TableDefine{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (),
            _ => children.push(node),
          }
        }
        compiled.push(Node::TableDefine{children});
      },
      parser::Node::AddRow{children} => {
        let result = self.compile_nodes(children);
        let mut children: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{..} => (),
            _ => children.push(node),
          }
        }
        compiled.push(Node::AddRow{children});
      },
      parser::Node::Index{children} => {
        compiled.append(&mut self.compile_nodes(children));
      },
      parser::Node::DotIndex{children} => {
        let result = self.compile_nodes(children);
        let mut columns: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{token, byte} => (),
            _ => columns.push(node),
          };
        }
        compiled.push(Node::DotIndex{rows: vec![], columns});
      },
      parser::Node::BracketIndex{children} => {
        let result = self.compile_nodes(children);
        let mut columns: Vec<Node> = Vec::new();
        for node in result {
          match node {
            Node::Token{token, byte} => (),
            _ => columns.push(node),
          };
        }
        compiled.push(Node::BracketIndex{rows: vec![], columns});
      },
      parser::Node::Table{children} => {
        let result = self.compile_nodes(children);
        match &result[1] {
          Node::Identifier{name, id} => compiled.push(Node::Table{name: name.to_string(), id: *id}),
          _ => (),
        };
      },  
      // Quantities
      parser::Node::Number{children} => {
        let mut value = 0;
        let mut result = self.compile_nodes(children);
        let mut place = result.len();
        for node in result {
          match node {
            Node::Token{token, byte} => {
              let digit = byte_to_digit(byte).unwrap();
              let q = digit * magnitude(place);
              place -= 1;
              value += q;
            },
            _ => (),
          }
        }
        compiled.push(Node::Constant{value});
      },
      // String-like nodes
      parser::Node::Paragraph{children} => {
        let mut result = self.compile_nodes(children);
        let node = match &result[0] {
          Node::String{text} => Node::Paragraph{text: text.clone()},
          _ => Node::Null,
        };
        compiled.push(node);
      },
      parser::Node::Title{children} => {
        let mut result = self.compile_nodes(children);
        // space space #
        let node = match &result[2] {
          Node::String{text} => Node::Title{text: text.clone()},
          _ => Node::Null,
        };
        compiled.push(node);
      },
      parser::Node::Subtitle{children} => {
        let mut result = self.compile_nodes(children);
        // space space # #
        let node = match &result[3] {
          Node::String{text} => Node::Title{text: text.clone()},
          _ => Node::Null,
        };
        compiled.push(node);
      },
      parser::Node::Text{children} => {
        let mut result = self.compile_nodes(children);
        let mut text_node = String::new();
        for node in result {
          match node {
            Node::String{text} => text_node.push_str(&text),
            Node::Token{token: Token::Space, ..} => text_node.push(' '),
            _ => (),
          }
        }
        compiled.push(Node::String{text: text_node});
      },
      parser::Node::Word{children} => {
        let mut word = String::new();
        let mut result = self.compile_nodes(children);
        for node in result {
          match node {
            Node::Token{token, byte} => {
              let character = byte_to_char(byte).unwrap();
              word.push(character);
            },
            _ => (),
          }
        }
        compiled.push(Node::String{text: word});
      },
      parser::Node::TableIdentifier{children} |
      parser::Node::Identifier{children} => {
        let mut word = String::new();
        let mut result = self.compile_nodes(children);
        for node in result {
          match node {
            Node::Token{token, byte} => {
              let character = byte_to_char(byte).unwrap();
              word.push(character);
            },
            _ => compiled.push(node),
          }
        }
        let id = Hasher::hash_string(word.clone());
        compiled.push(Node::Identifier{name: word, id});
      },
      // Math
      parser::Node::L1{children} |
      parser::Node::L2{children} |
      parser::Node::L3{children} |
      parser::Node::L4{children} => {
        let result = self.compile_nodes(children);
        let mut last = Node::Null;
        for node in result {
          match last {
            Node::Null => last = node,
            _ => {
              let (name, mut children) = match node {
                Node::Function{name, mut children} => (name.clone(), children.clone()),
                _ => (String::from(""), vec![]),
              };
              children.push(last);
              children.reverse();
              last = Node::Function{name, children};
            },
          };
        }
        compiled.push(last);
      },
      parser::Node::L1Infix{children} |
      parser::Node::L2Infix{children} |
      parser::Node::L3Infix{children} => {
        let result = self.compile_nodes(children);
        let operator = &result[1].clone();
        let input = &result[3].clone();
        let name: String = match operator {
          Node::Token{token, byte} => byte_to_char(*byte).unwrap().to_string(),
          _ => String::from(""),
        };        
        compiled.push(Node::Function{name, children: vec![input.clone()]});
      },
      // Pass through nodes. These will just be omitted
      parser::Node::Comparator{children} |
      parser::Node::IdentifierOrNumber{children} |
      parser::Node::ProseOrCode{children}|
      parser::Node::StatementOrExpression{children} |
      parser::Node::DataWatch{children} |
      parser::Node::Constant{children} |
      parser::Node::SetOperator{children} |
      parser::Node::Repeat{children} |
      parser::Node::Alphanumeric{children} |
      parser::Node::IdentifierCharacter{children} => {
        compiled.append(&mut self.compile_nodes(children));
      },
      parser::Node::Token{token, byte} => {
        compiled.push(Node::Token{token, byte});
      },
      _ => (),
    }
    
    //self.constraints = constraints.clone();
    compiled
  }

  pub fn compile_nodes(&mut self, nodes: Vec<parser::Node>) -> Vec<Node> {
    let mut compiled = Vec::new();
    for node in nodes {
      compiled.append(&mut self.build_syntax_tree(node));
    }
    compiled
  }

}

fn get_destination_register(constraint: &Constraint) -> Option<usize> {
  match constraint {
    Constraint::CopyInput{input, memory} => Some(*memory as usize),
    _ => None,
  }
}

// ## Appendix 

// ### Encodings

fn byte_to_digit(byte: u8) -> Option<u64> {
  match byte {
    48 => Some(0),
    49 => Some(1),
    50 => Some(2),
    51 => Some(3),
    52 => Some(4),
    53 => Some(5),
    54 => Some(6),
    55 => Some(7),
    56 => Some(8),
    57 => Some(9),
    _ => None,
  }
}

fn byte_to_char(byte: u8) -> Option<char> {
  match byte {
    33 => Some('!'),
    42 => Some('*'),
    43 => Some('+'),
    45 => Some('-'),
    47 => Some('/'),
    48 => Some('0'),
    49 => Some('1'),
    50 => Some('2'),
    51 => Some('3'),
    52 => Some('4'),
    53 => Some('5'),
    54 => Some('6'),
    55 => Some('7'),
    56 => Some('8'),
    57 => Some('9'),
    60 => Some('<'),
    62 => Some('>'),
    97 => Some('a'),
    98 => Some('b'),
    99 => Some('c'),
    100 => Some('d'),
    101 => Some('e'),
    102 => Some('f'),
    103 => Some('g'),
    104 => Some('h'),
    105 => Some('i'),
    106 => Some('j'),
    107 => Some('k'),
    108 => Some('l'),
    109 => Some('m'),
    110 => Some('n'),
    111 => Some('o'),
    112 => Some('p'),
    113 => Some('q'),
    114 => Some('r'),
    115 => Some('s'),
    116 => Some('t'),
    117 => Some('u'),
    118 => Some('v'),
    119 => Some('w'),
    120 => Some('x'),
    121 => Some('y'),    
    122 => Some('z'),
    126 => Some('~'),
    65 => Some('A'),
    66 => Some('B'),
    67 => Some('C'),
    68 => Some('D'),
    69 => Some('E'),
    70 => Some('F'),
    71 => Some('G'),
    72 => Some('H'),
    73 => Some('I'),
    74 => Some('J'),
    75 => Some('K'),
    76 => Some('L'),
    77 => Some('M'),
    78 => Some('N'),
    79 => Some('O'),
    80 => Some('P'),
    81 => Some('Q'),
    82 => Some('R'),
    83 => Some('S'),
    84 => Some('T'),
    85 => Some('U'),
    86 => Some('V'),
    87 => Some('W'),
    88 => Some('X'),
    89 => Some('Y'),
    90 => Some('Z'),
    94 => Some('^'),
    _ => {
      println!("Unhandled Byte {:?}", byte);
      None
    },
  }
}

// ### Utility Functions

fn magnitude(n: usize) -> u64 {
  let mut m = 1;
  for i in 1 .. n {
    m = m * 10;
  }
  m
}
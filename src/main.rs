#[cfg(feature = "with_cargo")]
mod position;
#[cfg(feature = "with_cargo")]
mod board;
#[cfg(feature = "with_cargo")]
mod location;
#[cfg(feature = "with_cargo")]
mod engine;

#[cfg(feature = "with_cargo")]
use position::Position;
#[cfg(feature = "with_cargo")]
use location::Location;

#[cfg(not(feature = "with_cargo"))]
use src::position::Position;

#[cfg(not(feature = "with_cargo"))]
use src::location::Location;

use std::io;

pub fn main() {

  let mut position = Position::default();
  println!("{}[2J", 27 as char);
  
  loop {

    println!("{}", position);
    println!("{:?}'s turn", position.turn);

    //Input your move in the format of "[int] [int]"
    let mut input = String::new();

    if let Err(e) = io::stdin().read_line(&mut input){
      println!("{}", e);
      continue;
    } else {
      println!("{}[2J", 27 as char);

    }
    
    let mut iter = input.split_whitespace();

    if let (Some(x), Some(y)) = (iter.next(), iter.next()) {
      if let (Ok(int_x), Ok(int_y)) = (x.parse(), y.parse()){
        if let Err(_) = position.make_move(Location::new(int_x, int_y)){
          println!("Couldn't put a piece there!");
        }
      } else {
        println!("Couldn't parse as integers");        
      }
    } else {
      println!("Invalid input format.");           
    }
  }
}


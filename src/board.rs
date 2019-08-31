use std::io::ErrorKind;
use std::ops::Index;

#[cfg(feature = "with_cargo")]
use position::Player;
#[cfg(feature = "with_cargo")]
use location::Location;

#[cfg(not(feature = "with_cargo"))]
use src::position::Player;
#[cfg(not(feature = "with_cargo"))]
use src::location::Location;


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BoardValue{
  Empty,
  Filled(Player)
}

impl BoardValue{
  fn is_color(&self, player: &Player) -> bool{
    match self {
      BoardValue::Empty => false,
      BoardValue::Filled(i) => i == player
    }
  }
}

impl Default for BoardValue{
  fn default() -> Self{
    BoardValue::Empty
  }
}

#[derive(Debug)]
pub struct Board {
  pub board: Vec<Vec<BoardValue>>,
  pub size_x: usize,
  pub size_y: usize
}

impl Board{
  #[allow(dead_code)]
  pub fn new(board: Vec<Vec<BoardValue>>) -> Self {
    Self {
      size_x: board.len(),
      size_y: match board.get(0){
        Some(i) => i.len(),
        None => 0,
      },
      board
    }
  }

  #[allow(dead_code)]
  pub fn append(&mut self, data: Vec<BoardValue>){
    self.board.push(data);
  }

  pub fn in_bounds(&self, location: &Location) -> bool {
    (0..self.size_x).contains(&(location.x as usize)) && 
    (0..self.size_y).contains(&(location.y as usize))
  }

  pub fn clear_at(&mut self, location: &Location) -> Result<(), ErrorKind> {
    if self.in_bounds(location) {
      self.board[location.x as usize][location.y as usize] = BoardValue::Empty;
      Ok(())
    } else {
      Err(ErrorKind::InvalidInput)
    }
  }

  pub fn insert_piece(&mut self, color: &Player, location: &Location) -> Result<(), ErrorKind> {
      if self.in_bounds(location){
        self.board[location.x as usize][location.y as usize] = BoardValue::Filled(color.clone()); 
        Ok(())
      } else {
        Err(ErrorKind::InvalidInput)
      }
  }

  pub fn get_at(&self, location: &Location) -> Result<&BoardValue, ErrorKind>{
    if !self.in_bounds(location){
      Err(ErrorKind::InvalidInput)
    }else{
      Ok(&self.board[location.x as usize][location.y as usize])
    }
  }

  pub fn is_color_at(&self, location: &Location, player: &Player) -> bool{
    match self.get_at(location){
      Ok(i) => {
        i.is_color(player)
      },
      Err(_) => false
    }
  }

  pub fn proxy<'b>(&'b self, location: &'b Location) -> BoardProxy<'b>{
    BoardProxy::new(self, &location)
  }
}

impl Default for Board{
  fn default() -> Self{

      let size_x = 13;
      let size_y = 13;

      Self {
        size_x: size_x,
        size_y: size_y,
        board: vec![vec![BoardValue::default(); size_x]; size_y],
      }
  }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "   ")?;
      for i in 0..self.size_x{
        write!(f, "{:2}", i)?;
      }
      write!(f, "\n")?;
      for x in 0..self.size_x{
        write!(f, "{:3} ", x)?;

        for y in 0..self.size_y{
          match &self.board[y][x]{        
            BoardValue::Empty => write!(f, "  ")?,
            BoardValue::Filled(i) => match i {
              Player::White => write!(f, " W")?,
              Player::Black => write!(f, " B")?
            }
          };
        }
        write!(f, "\n")?;
      }

      Ok(())
    }
}


impl Index<usize> for Board{
    type Output = Vec<BoardValue>;

    fn index(&self, index: usize) -> &Self::Output {
      &self.board[index]
    }
}

pub struct BoardProxy<'b>{
  center: &'b Location,
  board: &'b Board
}

impl<'b> BoardProxy<'b>{
  pub fn new(board: &'b Board, center: &'b Location) -> Self{
    Self {
      center,
      board
    }
  }

  pub fn get_absolute(&self, location: &Location) -> Location {
    location + self.center
  }

  #[allow(dead_code)]
  pub fn get_at(&self, location: &Location) -> Result<&BoardValue, ErrorKind>{
    let x = self.center.x + location.x;
    let y = self.center.y + location.y;
    
    self.board.get_at(&Location::new(x,y))
  }


  pub fn is_color_at(&self, location: &Location, player: &Player) -> bool{
    let x = self.center.x + location.x;
    let y = self.center.y + location.y;

    self.board.is_color_at(&Location::new(x,y), player)
  }
}

use std::collections::HashSet;
use std::io::ErrorKind;

#[cfg(feature = "with_cargo")]
use board::{Board, BoardValue};
#[cfg(feature = "with_cargo")]
use location::Location;


#[cfg(not(feature = "with_cargo"))]
use src::board::{Board, BoardValue};
#[cfg(not(feature = "with_cargo"))]
use src::location::Location;

#[derive(Debug, Clone)]
struct EmptySquareError;
impl std::fmt::Display for EmptySquareError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Empty square found at position")
    }
}
impl std::error::Error for EmptySquareError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Player {
  Black,
  White
}

impl Default for Player {
  fn default() -> Self {
    Player::Black
  }
}

impl Player {
  fn switch(&self) -> Self {
    match self {
      Player::White => Player::Black,
      _ => Player::White
    }
  } 
}

pub struct Position {
  pub board: Board,
  pub turn: Player,

  pub legal_white_moves: HashSet<Location>,
  pub legal_black_moves: HashSet<Location>,
}

impl Position{
  /// Switches the current active player
  fn next_turn(&mut self){
    self.turn = self.turn.switch();
  }

  /// Returns the legal moves set of the current active player
  fn legal_moves(&mut self) -> &mut HashSet<Location> {
    match self.turn {
      Player::Black => &mut self.legal_black_moves,
      _ => &mut self.legal_white_moves
    }
  }

  pub fn make_move(&mut self, location: Location) -> Result<(), ErrorKind>{

    // move calidation
    if !self.legal_moves().contains(&location) {
      return Err(ErrorKind::InvalidInput);
    }

    // piece insertion
    self.board.insert_piece(&self.turn, &location)?;

    // updating of legal moves
    for i in self.get_reachable(&location, &self.turn.to_owned()) {
      self.legal_moves().insert(i.to_owned());
    };

    self.legal_white_moves.remove(&location);
    self.legal_black_moves.remove(&location);

    match self.turn {
      Player::Black => self.get_cut_off_squares(&location)?
        .iter()
        .for_each(|i| {self.legal_white_moves.remove(i);}),
      _ => self.get_cut_off_squares(&location)?
        .iter()
        .for_each(|i| {self.legal_black_moves.remove(i);}),
    }   

    // capturing
    let cloud = self.get_connection_cloud(&location)?;
    for coordinate in cloud
      .iter()
      .filter(|i| self.board.is_color_at(i, &self.turn.switch()))
      .collect::<Vec<&Location>>(){
      dbg!(&coordinate);

      if let Some(pieces_to_remove) = self.connection_to_edge(coordinate)?{
        self.capture_piece(&coordinate, pieces_to_remove)?;
      }
    }

    // wrapping up
    self.next_turn();

    Ok(())
  }


  /// Handles all the logic behind capturing a piece and updating the legal moves sets
  fn capture_piece(&mut self, _location: &Location, pieces_to_remove: HashSet<Location>) -> Result<(), ErrorKind> {
    for piece in pieces_to_remove{
      self.board.clear_at(&piece)?;
    }

    // Add locations of fallen pieces to legal moves of both black and white
    // Remove positions previously reachable from fallen pieces from legal moves
    // Idea: possibly do these together by iterating over all the locations of fallen pieces once.
    // more?

    Ok(())
  }

  /// Returns all the squares around a placed piece,
  ///  that used to be accessible from other places around the placed piece,
  /// but are now blocked off by this placed piece.
  fn get_cut_off_squares(&mut self, location: &Location) -> Result<Vec<Location>, ErrorKind> {
    let other_player = match self.board.get_at(location)?{
      BoardValue::Filled(i) => i.switch(),
      _ => return Err(ErrorKind::InvalidInput),
    };

    let empty = BoardValue::Empty;

    Ok(self.get_connection_cloud(location)?
      .iter()
      .filter(|i| {
        self.board.get_at(i) == Ok(&empty) &&
        self.get_reachable(&i, &other_player)
          .iter()
          .filter(|i| self.board.is_color_at(&i, &other_player))
          .peekable() // is empty
          .peek()
          .is_none()
      })
      .cloned()
      .collect())
  }

  /// Returns whether or not a piece is connected to the edge in any way. 
  /// Uses a depth first approch.
  /// returns either None (the piece was connected to the edge)
  /// or Some(HashSet<Location>) indicating the piece is not connecting to the edge.
  /// In this case, the returned hashset is a history of all pieces it visited during /// it's DFS. All of these pieces are definitely not connected to the edge and can 
  /// therefore with certainty be removed from the game.
  fn connection_to_edge(&self, location: &Location) -> Result<Option<HashSet<Location>>, ErrorKind>{
    let player = match self.board.get_at(location)?{
      BoardValue::Filled(i) => i,
      _ => return Err(ErrorKind::InvalidInput),
    };


    let mut visited: HashSet<Location> = HashSet::new();
    let mut trace_stack: Vec<Location> = Vec::new();

    trace_stack.push(location.to_owned());

    while let Some(coordinate) = trace_stack.pop() {
      
      if visited.contains(&coordinate){
        continue;
      }

      if 
        coordinate.x == 0 || 
        coordinate.x == self.board.size_x as isize - 1 || 
        coordinate.y == 0 || 
        coordinate.y == self.board.size_y as isize - 1 {
        return Ok(None);
      }

      visited.insert(coordinate.to_owned());

      trace_stack.extend(
        self.get_reachable(&coordinate, player)
        .iter()
        .filter(|i| self.board.is_color_at(i, player))
        .filter(|i| !visited.contains(i))
        .cloned()
      )
    }

    return Ok(Some(visited));
  }

  /// Calculates and returns for all pieces of around a location which are the opposite
  /// color to the piece, the locations that would be blocked by the placing of the
  /// piece at the location. This can be used to check if pieces in these places should 
  /// fall, or be removed from the legal moves sets.
  fn get_connection_cloud(&self, location: &Location) -> Result<Vec<Location>, ErrorKind> {

    let mut result = HashSet::new();
    
    let result_vec: Vec<Location> = {
      let other_player = match self.board.get_at(location)?{
        BoardValue::Filled(i) => i.switch(),
        _ => return Err(ErrorKind::InvalidInput),
      };

      let proxy = self.board.proxy(location);

      if proxy.is_color_at(&Location::new(-2, 0), &other_player) {
        result.insert(( 0, 1));
        result.insert(( 0, -1));
      }
      if proxy.is_color_at(&Location::new(-1, 1), &other_player) {
        result.insert(( 0,-1));
        result.insert(( 1, 0));
      }
      if proxy.is_color_at(&Location::new(-1, 0), &other_player) {
        result.insert(( 1, 1));
        result.insert(( 1,-1));
        result.insert(( 0, 2));
        result.insert(( 0,-2));
      }
      
      if proxy.is_color_at(&Location::new(-1,-1), &other_player) {
        result.insert(( 1, 0));
        result.insert(( 0, 1));
      }

      if proxy.is_color_at(&Location::new( 0, 2), &other_player) {
        result.insert((-1, 0));
        result.insert(( 1, 0));
      }

      if proxy.is_color_at(&Location::new( 0, 1), &other_player) {
        result.insert((-1,-1));
        result.insert(( 1,-1));
        result.insert((-2, 0));
        result.insert(( 2, 0));
      }
      
      if proxy.is_color_at(&Location::new( 0,-1), &other_player) {
        result.insert(( 1, 1));
        result.insert((-1, 1));
        result.insert((-2, 0));
        result.insert(( 2, 0));
      }

      if proxy.is_color_at(&Location::new( 0,-2), &other_player) {
        result.insert((-1, 0));
        result.insert(( 1, 0));
      }

      if proxy.is_color_at(&Location::new( 1, 1), &other_player) {
        result.insert(( 0,-1));
        result.insert((-1, 0));
      }

      if proxy.is_color_at(&Location::new( 1, 0), &other_player) {
        result.insert((-1, 1));
        result.insert((-1,-1));
        result.insert(( 0, 2));
        result.insert(( 0,-2));
      }

      if proxy.is_color_at(&Location::new( 1,-1), &other_player) {
        result.insert(( 0, 1));
        result.insert((-1, 0));
      }

      if proxy.is_color_at(&Location::new( 2, 0), &other_player) {
        result.insert(( 0, 1));
        result.insert(( 0,-1));
      } 

      result
        .iter()
        .map(|i| Location::new(i.0, i.1))
        .map(|i| proxy.get_absolute(&i))
        .collect()
    };
      
    Ok(result_vec.iter().
      filter(|i| 
        i.x >= 0 && 
        i.x < self.board.size_x as isize && 
        i.y >= 0 &&
        i.y < self.board.size_y as isize
      )
      .cloned()
      .collect())
  }
  

  fn get_reachable(&self, location: &Location, player: &Player) -> Vec<Location>{

    self.connections_around(location)
      .iter()
      .filter(|square| self.is_connection_between(location, square, player).unwrap_or(false))
      .map(|i| i.to_owned())
      .collect()
  }

  fn connections_around(&self, location: &Location) -> Vec<Location>{
    let proxy = self.board.proxy(location);
    
    let result = vec![
      proxy.get_absolute(&Location::new(-2, 1)),
      proxy.get_absolute(&Location::new(2, 1)),
      proxy.get_absolute(&Location::new(2, -1)),
      proxy.get_absolute(&Location::new(-2, -1)),
      
      proxy.get_absolute(&Location::new(-1, 2)),
      proxy.get_absolute(&Location::new(1, 2)),
      proxy.get_absolute(&Location::new(1, -2)),
      proxy.get_absolute(&Location::new(-1, -2))
    ];

    result.iter().filter(|i| 
      i.x >= 0 && 
      i.x < self.board.size_x as isize && 
      i.y >= 0 &&
      i.y < self.board.size_y as isize
    ).cloned().collect()
  }

  /// Calculates if one connection between two squares is uninterrupted 
  /// NOTE: does not check if the connection actually exists 
  /// or is in the bounds of the board.
  /// Returns false when there is no piece at the from location.
  fn is_connection_between(&self, from: &Location, to: &Location, player: &Player) -> Result<bool, ErrorKind>{

    let delta_x = to.x - from.x;
    let delta_y = to.y - from.y;

    let other_player = player.switch();

    let proxy = self.board.proxy(from);

    Ok(match (delta_x, delta_y) {
      (2, 1) => {
        !((proxy.is_color_at(&Location::new( 0, 1), &other_player)  ||
           proxy.is_color_at(&Location::new( 1, 1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new( 2, 0), &other_player)))
      },
      (2, -1) => {
        !((proxy.is_color_at(&Location::new( 0,-1), &other_player)  ||
           proxy.is_color_at(&Location::new( 1,-1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new( 2, 0), &other_player)))
      },
      (-2, 1) => {
        !((proxy.is_color_at(&Location::new( 0, 1), &other_player)  ||
           proxy.is_color_at(&Location::new(-1, 1), &other_player)) &&
          (proxy.is_color_at(&Location::new(-1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new(-2, 0), &other_player)))
      },
      (-2, -1) => {
        !((proxy.is_color_at(&Location::new( 0,-1), &other_player)  ||
           proxy.is_color_at(&Location::new(-1,-1), &other_player)) &&
          (proxy.is_color_at(&Location::new(-1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new(-2, 0), &other_player)))
      },
      (1, 2) => {
        !((proxy.is_color_at(&Location::new( 1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new( 1, 1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 0, 1), &other_player)  ||
           proxy.is_color_at(&Location::new( 0, 2), &other_player)))
      },
      (1, -2) => {
        !((proxy.is_color_at(&Location::new( 1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new( 1,-1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 0,-1), &other_player)  ||
           proxy.is_color_at(&Location::new( 0,-2), &other_player)))
      },
      (-1, 2) => {
        !((proxy.is_color_at(&Location::new(-1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new(-1, 1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 0, 1), &other_player)  ||
           proxy.is_color_at(&Location::new( 0, 2), &other_player)))
      },
      (-1, -2) => {
        !((proxy.is_color_at(&Location::new(-1, 0), &other_player)  ||
           proxy.is_color_at(&Location::new(-1,-1), &other_player)) &&
          (proxy.is_color_at(&Location::new( 0,-1), &other_player)  ||
           proxy.is_color_at(&Location::new( 0,-2), &other_player)))
      },
      _ => return Ok(false)
    })
  }
}

impl Default for Position {
  fn default() -> Self {
    
    let board = Board::default();
    let mut edges = HashSet::new();

    for x in 0..board.size_x {
      edges.insert(Location::new(x as isize, 0));
      edges.insert(Location::new(x as isize, board.size_y as isize - 1));
    }

    for y in 1..board.size_y-1{
      edges.insert(Location::new(0, y as isize));
      edges.insert(Location::new(board.size_x as isize - 1, y as isize));
    }

    return Self{
      board,
      turn: Player::default(),
      
      legal_white_moves: edges.clone(),
      legal_black_moves: edges,
    }
  }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "   ")?;
      for i in 0..self.board.size_x{
        write!(f, "{:2}", i)?;
      }
      write!(f, "\n")?;
      for x in 0..self.board.size_x{
        write!(f, "{:2} ", x)?;

        for y in 0..self.board.size_y{
          match &self.board.board[y][x]{        
            BoardValue::Empty => {
              let location = Location::new(y as isize,x as isize);
              let black_contains = self.legal_black_moves.contains(&location);
              let white_contains = self.legal_white_moves.contains(&location);
              
              match (black_contains, white_contains){
                ( true, true) => write!(f, "\x1b[036m {}\x1b[0m", "x")?,
                ( true,false) => write!(f, "\x1b[031m {}\x1b[0m", "x")?,
                (false, true) => write!(f, "\x1b[033m {}\x1b[0m", "x")?,
                _ => write!(f, "  ")?, 
              }
            },
            BoardValue::Filled(i) => match i {
              Player::White => write!(f, "\x1b[033m W\x1b[0m")?,
              Player::Black => write!(f, "\x1b[031m B\x1b[0m")?
            }
          };
        }
        write!(f, "\n")?;
      }

      write!(f, "\n both: \x1b[036m {} \x1b[0m only black: \x1b[031m {} \x1b[0m only white: \x1b[033m {} \x1b[0m", "x", "x", "x")?;

      Ok(())
    }
}

#[cfg(test)]
mod tests{
  use crate::position::{Position, Player};
  use crate::location::Location;
  use crate::board::BoardValue;

  #[test]
  fn test_capture_pieces(){
    let mut position = Position::default();
    position.make_move(Location::new(0,0)).unwrap();
    position.make_move(Location::new(1,0)).unwrap();
    position.make_move(Location::new(2,1)).unwrap();
    position.make_move(Location::new(0,1)).unwrap();

    println!("{}",position);

    assert_eq!(position.board.get_at(&Location::new(2,1)).unwrap(), &BoardValue::Empty);
  }

  #[test]
  fn test_insert_piece_error(){
    let mut position = Position::default();
    
    // make a move that's invalid
    assert!(position.make_move(Location::new(5,5)).is_err());

    // make a move that's valid
    assert!(position.make_move(Location::new(0,0)).is_ok());
    
    // can't repeat the same move
    assert!(position.make_move(Location::new(0,0)).is_err());
  }

  // #[test]
  // fn test_print_position(){
  //   let mut position = Position::default();

  //   position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
  //   position.board.insert_piece(&Player::Black, &Location::new(6,5)).unwrap();

  //   println!("{}", position);
  // }

  #[test]
  fn test_get_cut_off_squares(){
    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(6,5)).unwrap();
  
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 4);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 3)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 7)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::White, &Location::new(6,6)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(6,5)).unwrap();
      
      assert_eq!(position.get_cut_off_squares(&Location::new(5,5)).unwrap().len(), 2);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 3)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 7)));
    }
  }

  #[test]
  fn test_get_cloud(){
    
    {
      let mut position = Position::default();
      position.make_move(Location::new(0,0)).unwrap();
      position.make_move(Location::new(1,0)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(0,0)).unwrap().len(), 1);
      assert_eq!(position.get_connection_cloud(&Location::new(0,0)).unwrap()[0], Location::new(0,2));
    }

    {
      let mut position = Position::default();
      position.make_move(Location::new(0,0)).unwrap();
      position.make_move(Location::new(2,0)).unwrap();

      assert_eq!(position.get_connection_cloud(&Location::new(0,0)).unwrap().len(), 1);
      assert_eq!(position.get_connection_cloud(&Location::new(0,0)).unwrap()[0], Location::new(0,1));
    }

    {
      let mut position = Position::default();
      position.make_move(Location::new(0,5)).unwrap();
      position.make_move(Location::new(0,3)).unwrap();

      assert_eq!(position.get_connection_cloud(&Location::new(0,5)).unwrap().len(), 1);
      assert_eq!(position.get_connection_cloud(&Location::new(0,5)).unwrap()[0], Location::new(1, 5));
    }

    {
      let mut position = Position::default();
      position.make_move(Location::new(0,5)).unwrap();
      position.make_move(Location::new(0,4)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(0,5)).unwrap().len(), 2);
      assert!(position.get_connection_cloud(&Location::new(0,5)).unwrap().contains(&Location::new(2, 5)));
      assert!(position.get_connection_cloud(&Location::new(0,5)).unwrap().contains(&Location::new(1, 6)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(0,5)).unwrap();
      position.board.insert_piece(&Player::White, &Location::new(0,3)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(0,5)).unwrap().len(), 0);
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(0,5)).unwrap();
      position.board.insert_piece(&Player::White, &Location::new(0,3)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(0,5)).unwrap().len(), 0);
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(4,4)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 2);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(6, 5)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(4,5)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 4);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(6, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 3)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(6, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 7)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(6,5)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 4);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 3)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(5, 7)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(5,6)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 4);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(3, 5)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(6, 4)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(7, 5)));
    }

    {
      let mut position = Position::default();
      position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
      position.board.insert_piece(&Player::Black, &Location::new(5,4)).unwrap();
      
      assert_eq!(position.get_connection_cloud(&Location::new(5,5)).unwrap().len(), 4);
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(4, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(3, 5)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(6, 6)));
      assert!(position.get_connection_cloud(&Location::new(5,5)).unwrap().contains(&Location::new(7, 5)));
    }
  }

  #[test]  
  fn test_connections_around(){
    let position = Position::default();    

    //center
    let connections_around1 = position.connections_around(&Location::new(6,6));

    assert!(connections_around1.contains(&Location::new(8,5)));
    assert!(connections_around1.contains(&Location::new(8,7)));
    assert!(connections_around1.contains(&Location::new(4,5)));
    assert!(connections_around1.contains(&Location::new(4,7)));
    assert!(connections_around1.contains(&Location::new(7,8)));
    assert!(connections_around1.contains(&Location::new(5,8)));
    assert!(connections_around1.contains(&Location::new(7,4)));
    assert!(connections_around1.contains(&Location::new(5,4)));

    //edge
    let connections_around2 = position.connections_around(&Location::new(0,0));
    
    assert!(connections_around2.contains(&Location::new(2,1)));
    assert!(connections_around2.contains(&Location::new(1,2)));

  }

  #[test]
  fn test_get_reachable(){
    let mut position = Position::default();
    position.board.insert_piece(&Player::White, &Location::new(5,5)).unwrap();
    position.board.insert_piece(&Player::White, &Location::new(0,0)).unwrap();

    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(4,3),
      &Player::White
    ).unwrap());

    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(7,4),
      &Player::White
    ).unwrap());

    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(3,6),
      &Player::White
    ).unwrap());
    
    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(6,7),
      &Player::White
    ).unwrap());
    
    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(3,4),
      &Player::White
    ).unwrap());

    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(4,7),
      &Player::White
    ).unwrap());

    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(6,3),
      &Player::White
    ).unwrap());
    
    assert!(position.is_connection_between(
      &Location::new(5,5), 
      &Location::new(7,6),
      &Player::White
    ).unwrap());
    
    assert!(!position.is_connection_between(
      &Location::new(0,0), 
      &Location::new(2,2),
      &Player::White
    ).unwrap());
  }
}



// O O N O O
// O N N N O
// N N Z N N
// O N N N O
// O O N O O


// O B A O O
// O C B O O
// O D W O O
// O O O O O
// O O O O O

// connection = !((A | B) & (C | D))
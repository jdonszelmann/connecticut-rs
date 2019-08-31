use std::ops::Add;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct Location {
  pub x: isize,
  pub y: isize
}

impl Location {
  pub fn new(x: isize, y: isize) -> Self {
    Self {x, y}
  }

  // pub fn new(x: usize, y: usize) -> Self {
  //   Self {x as isize, y as isize}
  // }
}

impl Add<&Location> for &Location{
  type Output = Location;

  fn add (self, location: &Location) -> Self::Output {
    Self::Output {
      x: location.x as isize + self.x,
      y: location.y as isize + self.y
    }
  }
}


#[allow(dead_code)]
struct Engine<'e> {
  evaluation_function: &'e Fn() -> f64,
  search_depth: u8,
}



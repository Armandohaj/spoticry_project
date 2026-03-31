pub struct IdGenerator {
    pub current: u32,
}

pub fn new_id_generator() -> IdGenerator {
    IdGenerator { current: 0 }
}

pub fn next_id(id_gen: &mut IdGenerator) -> u32 {
    id_gen.current += 1;
    id_gen.current
}
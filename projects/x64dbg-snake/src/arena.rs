use core::ops::{Index, IndexMut};

pub const SIZE: usize = 16;
pub const WALL: u8 = b'#';
pub const SNAKE: u8 = b'S';
pub const FOOD: u8 = b'*';
pub const EMPTY: u8 = b' ';
pub const HEAD: u8 = b'H';

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub usize, pub usize);

impl Pos {
    pub const fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    pub fn offset(&self) -> usize {
        self.1 * SIZE + self.0
    }

    pub fn is_wall(&self) -> bool {
        self.0 == 0 || self.0 == SIZE - 1 || self.1 == 0 || self.1 == SIZE - 1
    }

    pub fn is_inside(&self) -> bool {
        self.0 > 0 && self.0 < SIZE - 1 && self.1 > 0 && self.1 < SIZE - 1
    }

    pub fn distance_to(&self, other: Pos) -> usize {
        let dx = if self.0 > other.0 { self.0 - other.0 } else { other.0 - self.0 };
        let dy = if self.1 > other.1 { self.1 - other.1 } else { other.1 - self.1 };
        dx + dy
    }
}

#[repr(C, align(4096))]
pub struct Arena {
    data: [u8; (SIZE + 2) * SIZE],
    food_count: u32,
}

impl Arena {
    pub const fn new() -> Self {
        Self {
            data: [EMPTY; (SIZE + 2) * SIZE],
            food_count: 0,
        }
    }

    pub fn get(&self, pos: Pos) -> u8 {
        self.data[pos.offset()]
    }

    pub fn set(&mut self, pos: Pos, value: u8) {
        self.data[pos.offset()] = value;
    }

    pub fn clear(&mut self) {
        for i in 0..(SIZE + 2) * SIZE {
            self.data[i] = EMPTY;
        }
    }

    pub fn draw_border(&mut self) {
        for i in 0..SIZE {
            self.set(Pos(i, 0), WALL);
            self.set(Pos(i, SIZE - 1), WALL);
            self.set(Pos(0, i), WALL);
            self.set(Pos(SIZE - 1, i), WALL);
        }
    }

    pub fn increment_food(&mut self) {
        self.food_count += 1;
    }

    pub fn food_count(&self) -> u32 {
        self.food_count
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn render_stats(&mut self) {
        let row = SIZE;
        let mut idx = 0;
        
        let text = b"FOOD: ";
        for i in 0..6 {
            self.data[row * SIZE + i] = text[i];
        }
        
        let mut n = self.food_count;
        let mut digits = [0u8; 10];
        let mut len = 0;
        if n == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while n > 0 {
                digits[len] = (b'0' + (n % 10) as u8);
                n /= 10;
                len += 1;
            }
        }
        
        for i in 0..len {
            self.data[row * SIZE + 6 + i] = digits[len - 1 - i];
        }
    }
}

impl Index<Pos> for Arena {
    type Output = u8;
    fn index(&self, pos: Pos) -> &u8 {
        &self.data[pos.offset()]
    }
}

impl IndexMut<Pos> for Arena {
    fn index_mut(&mut self, pos: Pos) -> &mut u8 {
        &mut self.data[pos.offset()]
    }
}
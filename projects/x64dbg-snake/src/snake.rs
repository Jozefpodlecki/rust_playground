use heapless::Vec;

use crate::arena::{Pos, SIZE};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub fn delta(&self) -> (isize, isize) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[repr(transparent)]
pub struct SnakeBody(Vec<Pos, { SIZE * SIZE }>);

impl SnakeBody {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, pos: Pos) {
        let _ = self.0.push(pos);
    }

    pub fn pop(&mut self) -> Option<Pos> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn head(&self) -> Pos {
        self.0[0]
    }

    pub fn tail(&self) -> Pos {
        self.0[self.0.len() - 1]
    }

    pub fn iter(&self) -> core::slice::Iter<Pos> {
        self.0.iter()
    }

    pub fn contains(&self, pos: Pos) -> bool {
        self.0.contains(&pos)
    }
}

pub struct Snake {
    body: SnakeBody,
    direction: Direction,
    grow: bool,
}

impl Snake {
    pub fn new(head: Pos, direction: Direction) -> Self {
        let mut body = SnakeBody::new();
        let (dx, dy) = direction.opposite().delta();
        body.push(head);
        
        // Calculate second position safely
        let second = Pos(
            if dx < 0 { head.0 - 1 } else { head.0 + dx as usize },
            if dy < 0 { head.1 - 1 } else { head.1 + dy as usize },
        );
        body.push(second);
        
        Self { body, direction, grow: false }
    }

    pub fn head(&self) -> Pos {
        self.body.head()
    }

    pub fn body(&self) -> &SnakeBody {
        &self.body
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, dir: Direction) {
        if dir != self.direction.opposite() {
            self.direction = dir;
        }
    }

    pub fn grow(&mut self) {
        self.grow = true;
    }

    pub fn next_head(&self) -> Pos {
        let (dx, dy) = self.direction.delta();
        let head = self.head();
        Pos(head.0.wrapping_add(dx as usize), head.1.wrapping_add(dy as usize))
    }

    pub fn move_forward(&mut self) {
        let new_head = self.next_head();
        self.body.0.insert(0, new_head).unwrap_or_default();
        if !self.grow {
            let _ = self.body.0.pop();
        }
        self.grow = false;
    }

    pub fn collides_with(&self, pos: Pos) -> bool {
        self.body.contains(pos)
    }

    pub fn collides_with_self(&self) -> bool {
        let head = self.head();
        for i in 1..self.body.len() {
            if self.body.0[i] == head {
                return true;
            }
        }
        false
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }
}
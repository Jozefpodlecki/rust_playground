use heapless::Vec;
use crate::arena::{Arena, Pos, SIZE};
use crate::snake::{Direction, Snake};

#[derive(Clone, Copy)]
pub struct Node {
    pub pos: Pos,
    pub dist: usize,
}

impl Node {
    pub fn new(pos: Pos, dist: usize) -> Self {
        Self { pos, dist }
    }
}

pub struct Path(Vec<Pos, { SIZE * SIZE }>);

impl Path {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, pos: Pos) {
        let _ = self.0.push(pos);
    }

    pub fn first(&self) -> Option<Pos> {
        self.0.first().copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn reverse(&mut self) {
        self.0.reverse();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct Queue(Vec<Node, { SIZE * SIZE }>);

impl Queue {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, node: Node) {
        let _ = self.0.push(node);
    }

    pub fn pop(&mut self) -> Option<Node> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct BoundedMove;

impl BoundedMove {
    pub fn next_pos(from: Pos, dir: Direction) -> Option<Pos> {
        let (dx, dy) = dir.delta();
        let x = from.0 as isize + dx;
        let y = from.1 as isize + dy;
        if x >= 0 && x < SIZE as isize && y >= 0 && y < SIZE as isize {
            Some(Pos(x as usize, y as usize))
        } else {
            None
        }
    }
}

pub enum Algorithm {
    Bfs,
    Greedy,
    Hybrid,
}

pub trait Pathfinder {
    fn find_path(&self, snake: &Snake, target: Pos, arena: &Arena) -> Option<Path>;
}
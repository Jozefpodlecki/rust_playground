use crate::arena::{Arena, Pos};
use crate::snake::Snake;
use crate::ai::Pathfinder;
use super::types::Path;
use super::bfs::BfsPathfinder;
use super::greedy::GreedyPathfinder;

#[derive(Debug)]
pub struct HybridPathfinder {
    bfs: BfsPathfinder,
    greedy: GreedyPathfinder,
}

impl HybridPathfinder {
    pub fn new() -> Self {
        Self {
            bfs: BfsPathfinder,
            greedy: GreedyPathfinder,
        }
    }
}

impl Pathfinder for HybridPathfinder {
    fn find_path(&self, snake: &Snake, target: Pos, arena: &Arena) -> Option<Path> {
        if let Some(path) = self.bfs.find_path(snake, target, arena) {
            return Some(path);
        }
        self.greedy.find_path(snake, target, arena)
    }
}
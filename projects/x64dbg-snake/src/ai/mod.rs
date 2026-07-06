mod types;
mod bfs;
mod greedy;
mod hybrid;

use toolkit::stack_trait::{Buf, Stacked};
pub use types::*;
pub use bfs::BfsPathfinder;
pub use greedy::GreedyPathfinder;
pub use hybrid::HybridPathfinder;

use crate::arena::{Arena, Pos};
use crate::snake::{Direction, Snake};

pub type Buf64 = Buf<u8, 64>;
pub type StackedPathfinder = Stacked<dyn Pathfinder, Buf64>;

pub struct Ai(StackedPathfinder);

impl Ai {
    pub fn new(pathfinder: StackedPathfinder) -> Self {
        Self(pathfinder)
    }

    pub fn next_move(&self, snake: &Snake, food: Pos, arena: &Arena) -> Direction {
        let head = snake.head();

        for dir in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            if let Some(next) = BoundedMove::next_pos(head, dir) {
                if next == food {
                    return dir;
                }
            }
        }

        if let Some(path) = self.0.find_path(snake, food, arena) {
            if let Some(next) = path.first() {
                return Self::direction_to(head, next);
            }
        }

        Self::emergency_move(snake, arena)
    }

    fn emergency_move(snake: &Snake, arena: &Arena) -> Direction {
        let head = snake.head();

        for dir in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            if let Some(next) = BoundedMove::next_pos(head, dir) {
                if next.is_inside() && arena.get(next) != crate::arena::WALL && !snake.collides_with(next) {
                    return dir;
                }
            }
        }

        Direction::Up
    }

    fn direction_to(from: Pos, to: Pos) -> Direction {
        if to.0 > from.0 {
            Direction::Right
        } else if to.0 < from.0 {
            Direction::Left
        } else if to.1 > from.1 {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}

pub fn create_ai(algorithm: Algorithm) -> Ai {
    match algorithm {
        Algorithm::Bfs => {
            let pathfinder = Stacked::new(BfsPathfinder).unwrap();
            Ai::new(pathfinder)
        }
        Algorithm::Greedy => {
            let pathfinder = Stacked::new(GreedyPathfinder).unwrap();
            Ai::new(pathfinder)
        }
        Algorithm::Hybrid => {
            let pathfinder = Stacked::new(HybridPathfinder::new()).unwrap();
            Ai::new(pathfinder)
        }
    }
}
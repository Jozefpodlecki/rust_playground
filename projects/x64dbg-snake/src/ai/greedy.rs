use crate::arena::{Arena, Pos, SIZE};
use crate::snake::{Direction, Snake};
use crate::ai::Pathfinder;
use super::types::{BoundedMove, Path};

#[derive(Debug)]
pub struct GreedyPathfinder;

impl Pathfinder for GreedyPathfinder {

    fn find_path(&self, snake: &Snake, target: Pos, arena: &Arena) -> Option<Path> {
        let head = snake.head();
        let mut path = Path::new();
        let mut current = head;
        let mut visited = [false; SIZE * SIZE];

        for _ in 0..SIZE * SIZE {
            path.push(current);
            visited[current.offset()] = true;

            if current == target {
                return Some(path);
            }

            let mut best_dir = Direction::Up;
            let mut best_dist = usize::MAX;
            let mut found = false;

            for dir in Self::DIRECTIONS {
                if let Some(next) = BoundedMove::next_pos(current, dir) {
                    let idx = next.offset();
                    if !visited[idx]
                        && next.is_inside()
                        && arena.get(next) != crate::arena::WALL
                        && !snake.collides_with(next)
                    {
                        let dist = next.distance_to(target);
                        if dist < best_dist {
                            best_dist = dist;
                            best_dir = dir;
                            found = true;
                        }
                    }
                }
            }

            if !found {
                return None;
            }

            let (dx, dy) = best_dir.delta();
            current = Pos(
                (current.0 as isize + dx) as usize,
                (current.1 as isize + dy) as usize,
            );
        }

        None
    }
}

impl GreedyPathfinder {
    const DIRECTIONS: [Direction; 4] = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
}
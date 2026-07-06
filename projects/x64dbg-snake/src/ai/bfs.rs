use crate::ai::Pathfinder;
use crate::arena::{Arena, Pos, SIZE};
use crate::snake::{Direction, Snake};
use super::types::{BoundedMove, Node, Path, Queue};

#[derive(Debug)]
pub struct BfsPathfinder;

impl Pathfinder for BfsPathfinder {
    fn find_path(&self, snake: &Snake, target: Pos, arena: &Arena) -> Option<Path> {
        let start = snake.head();
        let mut queue = Queue::new();
        let mut visited = [false; SIZE * SIZE];
        let mut parent = [Pos(0, 0); SIZE * SIZE];

        queue.push(Node::new(start, 0));
        visited[start.offset()] = true;

        while let Some(node) = queue.pop() {
            if node.pos == target {
                return Self::build_path(start, target, &parent);
            }

            for dir in Self::DIRECTIONS {
                if let Some(next) = BoundedMove::next_pos(node.pos, dir) {
                    let idx = next.offset();
                    if next.is_inside()
                        && !visited[idx]
                        && arena.get(next) != crate::arena::WALL
                        && !snake.collides_with(next)
                    {
                        visited[idx] = true;
                        parent[idx] = node.pos;
                        queue.push(Node::new(next, node.dist + 1));
                    }
                }
            }
        }

        None
    }
}

impl BfsPathfinder {
    fn build_path(start: Pos, target: Pos, parent: &[Pos; SIZE * SIZE]) -> Option<Path> {
        let mut path = Path::new();
        let mut cur = target;
        while cur != start {
            path.push(cur);
            cur = parent[cur.offset()];
        }
        path.reverse();
        Some(path)
    }
}

impl BfsPathfinder {
    const DIRECTIONS: [Direction; 4] = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
}
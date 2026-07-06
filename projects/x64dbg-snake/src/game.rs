use crate::{
    ai::Ai,
    arena::{Arena, FOOD, HEAD, SNAKE, SIZE},
    random::Rng,
    snake::{Direction, Snake},
};

#[repr(transparent)]
pub struct Score(u32);

impl Score {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
pub struct Ticks(u32);

impl Ticks {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

pub struct Game {
    pub arena: Arena,
    pub snake: Snake,
    pub food: crate::arena::Pos,
    pub score: Score,
    pub ticks: Ticks,
    pub game_over: bool,
    rng: Rng,
    ai: Ai,
}

impl Game {
    pub fn new(ai: Ai) -> Self {
        let rng = Rng::new(0x9E3779B97F4A7C15);
        let head = crate::arena::Pos(SIZE / 2, SIZE / 2);
        let snake = Snake::new(head, Direction::Down);
        let food = rng.pos_near(head, 3, 6);

        Self {
            arena: Arena::new(),
            snake,
            food,
            score: Score::new(),
            ticks: Ticks::new(),
            game_over: false,
            rng,
            ai,
        }
    }

    pub fn tick(&mut self) {
        if self.game_over {
            return;
        }

        self.ticks.increment();

        let next_dir = self.ai.next_move(&self.snake, self.food, &self.arena);
        self.snake.set_direction(next_dir);
        self.snake.move_forward();

        let head = self.snake.head();

        if head.is_wall() {
            self.game_over = true;
            return;
        }

        if self.snake.collides_with_self() {
            self.game_over = true;
            return;
        }

        if head == self.food {
            self.snake.grow();
            self.score.increment();
            self.arena.increment_food();
            self.spawn_food();
        }

        self.render();
    }

    fn spawn_food(&mut self) {
        loop {
            let pos = self.rng.pos();
            if !self.snake.collides_with(pos) && !pos.is_wall() {
                self.food = pos;
                break;
            }
        }
    }

    pub fn render(&mut self) {
        self.arena.clear();
        self.arena.draw_border();
        self.arena.render_stats();

        for pos in self.snake.body().iter() {
            self.arena.set(*pos, SNAKE);
        }
        self.arena.set(self.snake.head(), HEAD);
        self.arena.set(self.food, FOOD);
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.arena.as_ptr()
    }
}
pub enum State {
    Init
}

pub struct Source {
    state: State,
}

impl Iterator for Source {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
        // match self.state {
        //     0 => {
        //         self.state += 1;
        //         Some(1)
        //     }
        //     1 => {
        //         self.state += 1;
        //         Some(2)
        //     }
        //     2 => {
        //         self.state += 1;
        //         Some(3)
        //     }
        //     _ => None,
        // }
    }
}

impl Source {
    pub fn new() -> Self {
        Self {
            state: State::Init
        }
    }
}
use crate::db::repositories::repositories::Repositories;


pub struct Simulator {
    repositories: Repositories
}

impl Simulator {
    pub fn new(repositories: Repositories) -> Self {
        Self { repositories }
    }

    pub fn setup(&mut self) {
        
    }

    pub fn tick(&mut self) {
        
    }
}
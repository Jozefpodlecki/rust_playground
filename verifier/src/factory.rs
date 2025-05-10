use crate::{exercise_one::ExerciseOneVerifierService, service::VerifierService};


pub fn get_by_exercise_name(name: &str) -> impl VerifierService {
    match name {
        "test" => ExerciseOneVerifierService::new(),
        _ => panic!("Unknown")
    }
}
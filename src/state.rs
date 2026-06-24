use crate::prelude::*;

#[derive(Default, Debug)]
pub struct State {
    pub valid: Vec<String>,
    pub missing: Vec<String>,
    pub invalid: Vec<String>,

    pub extra: Vec<String>,

    pub validation_length: usize,
    pub validation_progress: usize,
    pub extra_progress: usize,
    pub extra_length: usize,
}

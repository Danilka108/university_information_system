use self::repo::UniversityRepository;
use std::num::NonZeroI32;

pub mod exceptions;
mod repo;
mod service;

pub use service::UniversityService;
pub type BoxedUniversityRepository = Box<dyn UniversityRepository>;

#[derive(Debug, Clone)]
pub struct University<I = NonZeroI32> {
    pub id: I,
    pub name: String,
}

impl PartialEq for University {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for University {}

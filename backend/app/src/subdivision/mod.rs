pub mod exceptions;
mod repositories;
pub mod service;

use self::repositories::{SubdivisionMemberRepository, SubdivisionTagRepository};
use crate::{person::Person, tag::Tag, university::University};

pub use repositories::SubdivisionRepository;
pub type BoxedSubdivisionRepository = Box<dyn SubdivisionRepository>;
pub type BoxedSubdivisionMemberRepository = Box<dyn SubdivisionMemberRepository>;
pub type BoxedSubdivisionTagRepository = Box<dyn SubdivisionTagRepository>;

#[derive(Debug, Clone)]
pub struct Subdivision {
    id: SubdivisionId,
}

impl PartialEq for Subdivision {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Subdivision {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubdivisionId {
    pub university: University,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubdivisionTag(Subdivision, Tag);

#[derive(Debug, Clone)]
pub struct SubdivisionMember {
    pub id: (Subdivision, Person),
    pub role: String,
}

impl PartialEq for SubdivisionMember {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for SubdivisionMember {}

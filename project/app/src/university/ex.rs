use utils::repo::{case::Case, ex::FromRepoEx};

use super::{Entity, EntityAttr};

#[derive(Debug, thiserror::Error)]
pub enum Exception {
    #[error("university does not exist")]
    DoesNotExist,
    #[error("university name is already in use")]
    NameAlreadyInUse,
    #[error("university already exists")]
    AlreadyExists,
}

impl FromRepoEx<Entity> for Exception {
    fn from_repo_ex<Ok>(repo_ex: &utils::repo::ex::Exception<Entity>) -> Option<Self> {
        if Case::does_not_exist()
            .with_fields([EntityAttr::Id])
            .eq_to(&repo_ex)
        {
            return Exception::DoesNotExist.into();
        }

        if Case::unique_constraint_violated()
            .with_fields([EntityAttr::Id])
            .eq_to(&repo_ex)
        {
            return Exception::AlreadyExists.into();
        }

        if Case::unique_constraint_violated()
            .with_fields([EntityAttr::Name])
            .eq_to(&repo_ex)
        {
            return Exception::NameAlreadyInUse.into();
        }

        None
    }
}
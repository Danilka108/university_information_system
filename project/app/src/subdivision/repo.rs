use crate::{person, tag, university};

use super::{Entity, EntityId};

#[async_trait::async_trait]
pub trait Repo {
    async fn save(&mut self, entity: Entity) -> Result<Entity, anyhow::Error>;

    async fn delete(&mut self, entity: &Entity) -> Result<(), anyhow::Error>;

    async fn find(&mut self, id: EntityId) -> Result<Option<Entity>, anyhow::Error>;

    async fn find_by_name(&mut self, name: String) -> Result<Option<Entity>, anyhow::Error>;

    async fn list_by_university(
        &mut self,
        university_id: university::EntityId,
    ) -> Result<Vec<Entity>, anyhow::Error>;

    async fn list_by_tags(
        &mut self,
        tags_ids: impl IntoIterator<Item = tag::EntityId> + Send,
    ) -> Result<Vec<Entity>, anyhow::Error>;

    async fn list_by_members(
        &mut self,
        tags_ids: impl IntoIterator<Item = person::EntityId> + Send,
    ) -> Result<Vec<Entity>, anyhow::Error>;
}

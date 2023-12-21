mod models;

use app::{
    subdivision::{self, Entity, EntityId},
    tag, university,
};
use sea_query::{
    Alias, Asterisk, DynIden, Expr, JoinType, Query, SeaRc, SelectStatement, SimpleExpr,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    fetch_all, fetch_one,
    subdivision::models::{SelectResult, SubdivisionTagsIden, SubdivisionsIden},
    PgTransaction,
};

use self::models::{SubdivisionMembers, SubdivisionMembersIden, SubdivisionTags, Subdivisions};

pub struct PgSubdivisionRepo {
    txn: Arc<Mutex<PgTransaction<'static>>>,
}

impl PgSubdivisionRepo {
    async fn insert(&self, entity: Entity) -> Result<Subdivisions, anyhow::Error> {
        let mut query = Query::insert();
        let query = query
            .into_table(SubdivisionsIden::Table)
            .columns([SubdivisionsIden::Name, SubdivisionsIden::UniversityId])
            .values_panic([entity.name.into(), entity.university_id.value.into()])
            .returning_all();

        fetch_one::<Subdivisions>(&self.txn, query).await
    }

    async fn update(&self, entity: Entity) -> Result<Subdivisions, anyhow::Error> {
        let mut query = Query::update();
        let query = query
            .table(SubdivisionsIden::Table)
            .values([
                (SubdivisionsIden::Name, entity.name.into()),
                (
                    SubdivisionsIden::UniversityId,
                    entity.university_id.value.into(),
                ),
            ])
            .and_where(Expr::col(SubdivisionsIden::Id).is(entity.id.value))
            .returning_all();

        fetch_one::<Subdivisions>(&self.txn, query).await
    }

    async fn delete_tags(&self, id: i32) -> Result<Vec<SubdivisionTags>, anyhow::Error> {
        let mut query = Query::delete();
        let query = query
            .from_table(SubdivisionTagsIden::Table)
            .and_where(Expr::col(SubdivisionTagsIden::SubdivisionId).is(id))
            .returning_all();

        fetch_all(&self.txn, query).await
    }

    async fn insert_tags(
        &self,
        id: i32,
        tags: Vec<tag::EntityId>,
    ) -> Result<Vec<SubdivisionTags>, anyhow::Error> {
        let mut inserted_tags = Vec::new();

        for tag_name in tags {
            let mut query = Query::insert();
            let query = query
                .into_table(SubdivisionTagsIden::Table)
                .columns([
                    SubdivisionTagsIden::SubdivisionId,
                    SubdivisionTagsIden::TagName,
                ])
                .values_panic([id.into(), tag_name.value.into()])
                .returning_all();

            let tag = fetch_one::<SubdivisionTags>(&self.txn, query).await?;
            inserted_tags.push(tag);
        }

        Ok(inserted_tags)
    }

    // async fn select_tags(&self, id: i32) -> Result<Vec<SubdivisionTags>, anyhow::Error> {
    //     let mut query = Query::select();
    //     let query = query
    //         .column(Asterisk)
    //         .from(SubdivisionTagsIden::Table)
    //         .and_where(Expr::col(SubdivisionTagsIden::SubdivisionId).is(id));

    //     fetch_all(&self.txn, query).await
    // }

    async fn delete_members(&self, id: i32) -> Result<Vec<SubdivisionMembers>, anyhow::Error> {
        let mut query = Query::delete();
        let query = query
            .from_table(SubdivisionMembersIden::Table)
            .and_where(Expr::col(SubdivisionMembersIden::SubdivisionId).is(id))
            .returning_all();

        fetch_all(&self.txn, query).await
    }

    async fn insert_members(
        &self,
        id: i32,
        members: Vec<subdivision::Member>,
    ) -> Result<Vec<SubdivisionMembers>, anyhow::Error> {
        let mut inserted_members = Vec::new();

        for member in members {
            let mut query = Query::insert();
            let query = query
                .into_table(SubdivisionMembersIden::Table)
                .columns([
                    SubdivisionMembersIden::SubdivisionId,
                    SubdivisionMembersIden::PersonId,
                    SubdivisionMembersIden::Role,
                ])
                .values_panic([id.into(), member.person_id.value.into(), member.role.into()])
                .returning_all();

            let member = fetch_one::<SubdivisionMembers>(&self.txn, query).await?;
            inserted_members.push(member);
        }

        Ok(inserted_members)
    }

    // async fn select_members(&self, id: i32) -> Result<Vec<SubdivisionMembers>, anyhow::Error> {
    //     let mut query = Query::select();
    //     let query = query
    //         .column(Asterisk)
    //         .from(SubdivisionMembersIden::Table)
    //         .and_where(Expr::col(SubdivisionMembersIden::SubdivisionId).is(id));

    //     fetch_all(&self.txn, query).await
    // }

    // select * from subdivisions as s join (select m.subdivision_id, m.person_id, m.role, t.tag_name from subdivision_members as m full outer join subdivision_tags as t on m.subdivision_id = t.subdivision_id) as r on s.id = r.subdivision_id where s.x = y;
    async fn select(&self, where_expr: SimpleExpr) -> Result<Vec<SelectResult>, anyhow::Error> {
        let subdivision_table = SubdivisionsIden::Table;
        let subdivision_id = SubdivisionMembersIden::SubdivisionId;

        let mut query = Query::select();

        let (subquery_alias, subquery) = Self::join_members_and_tags();

        let on = Expr::col((subquery_alias.clone(), subdivision_id))
            .equals((subdivision_table, subdivision_id));

        query
            .from(subdivision_table)
            .column(Asterisk)
            .join_subquery(JoinType::InnerJoin, subquery, subquery_alias.clone(), on)
            .and_where(where_expr);

        fetch_all(&self.txn, &query).await
    }

    fn join_members_and_tags() -> (DynIden, SelectStatement) {
        let tag_table = SubdivisionTagsIden::Table;
        let tag_subdivision_id = SubdivisionTagsIden::SubdivisionId;
        let tag_name = SubdivisionTagsIden::TagName;

        let member_table = SubdivisionMembersIden::Table;
        let member_role = SubdivisionMembersIden::Role;
        let member_subdivision_id = SubdivisionMembersIden::SubdivisionId;
        let member_person_id = SubdivisionMembersIden::PersonId;

        let alias: DynIden = SeaRc::new(Alias::new("members_and_tags"));
        let mut query = Query::select();

        let on = Expr::col((member_table, member_subdivision_id))
            .equals((tag_table, tag_subdivision_id));

        query
            .column((tag_table, tag_name))
            .column((member_table, member_role))
            .column((member_table, member_subdivision_id))
            .column((member_table, member_person_id))
            .from(member_table)
            .full_outer_join(tag_table, on);

        (alias, query)
    }

    fn entity_from_select_result(join_res: Vec<SelectResult>) -> Option<Entity> {
        let (models, members_and_tags) = join_res
            .into_iter()
            .map(|v| (v.subdivision, (v.member, v.tag)))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let Some(model) = models.into_iter().take(1).next() else {
            return None;
        };

        let (members, tags) = members_and_tags.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();

        Some(model.into_entity(tags, members))
    }
}

#[async_trait::async_trait]
impl subdivision::Repo for PgSubdivisionRepo {
    async fn save(&mut self, entity: Entity) -> Result<Entity, anyhow::Error> {
        let model = if self.find(entity.id).await?.is_some() {
            self.update(entity.clone()).await?
        } else {
            self.insert(entity.clone()).await?
        };

        let _ = self.delete_tags(model.id).await?;
        let _ = self.delete_members(model.id).await?;

        let tags = self.insert_tags(model.id, entity.tags).await?;
        let members = self.insert_members(model.id, entity.members).await?;

        Ok(model.into_entity(tags, members))
    }

    async fn delete(&mut self, entity: &Entity) -> Result<(), anyhow::Error> {
        let mut query = Query::delete();
        let query = query
            .from_table(SubdivisionsIden::Table)
            .and_where(Expr::col(SubdivisionsIden::Id).is(entity.id.value))
            .returning_all();

        let _ = fetch_one::<Subdivisions>(&self.txn, query).await?;
        Ok(())
    }

    async fn find(&mut self, id: EntityId) -> Result<Option<Entity>, anyhow::Error> {
        let res = self
            .select(Expr::col(SubdivisionsIden::Id).is(id.value))
            .await?;

        Ok(Self::entity_from_select_result(res))
    }

    async fn find_by_name(&mut self, name: String) -> Result<Option<Entity>, anyhow::Error> {
        let res = self
            .select(Expr::col(SubdivisionsIden::Name).is(name))
            .await?;

        Ok(Self::entity_from_select_result(res))
    }

    async fn list_by_university(
        &mut self,
        university_id: university::EntityId,
    ) -> Result<Vec<Entity>, anyhow::Error> {
        let results = self
            .select(Expr::col(SubdivisionsIden::UniversityId).is(university_id.value))
            .await?;

        let mut groups = HashMap::<i32, Vec<SelectResult>>::new();
        for result in results {
            groups
                .entry(result.subdivision.id)
                .or_insert(vec![])
                .push(result);
        }

        let entities = groups
            .into_iter()
            .map(|(_, v)| Self::entity_from_select_result(v))
            .filter_map(|v| v)
            .collect();

        Ok(entities)
    }
}
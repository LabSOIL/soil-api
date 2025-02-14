use async_trait::async_trait;
use sea_orm::{
    entity::prelude::*, Condition, DatabaseConnection, EntityTrait, Order, PaginatorTrait,
    QuerySelect, QueryTrait,
};
use uuid::Uuid;

#[async_trait]
pub trait CRUDResource: Sized + Send + Sync
where
    <Self::EntityType as EntityTrait>::Model: Sync,
    <<Self::EntityType as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<uuid::Uuid>,
{
    type EntityType: EntityTrait + Sync;
    type ColumnType: ColumnTrait + std::fmt::Debug;
    type ModelType: ModelTrait;
    type ActiveModelType: sea_orm::ActiveModelTrait;
    type ApiModel: From<Self::ModelType>;
    type CreateModel: Into<Self::ActiveModelType>;
    type UpdateModel: Send + Sync;

    const ID_COLUMN: Self::ColumnType;
    const RESOURCE_NAME_SINGULAR: &str;
    const RESOURCE_NAME_PLURAL: &str;

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr>;

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr>;

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr>;

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr>;

    // async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr>;
    // async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr>;

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        let res = <Self::EntityType as EntityTrait>::delete_by_id(id)
            .exec(db)
            .await?;
        Ok(res.rows_affected as usize)
    }

    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
        Self::EntityType::delete_many()
            .filter(Self::ID_COLUMN.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(ids)
    }

    // async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64;
    // async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
    //     Self::EntityType::find()
    //         .filter(condition)
    //         .count(db)
    //         .await
    //         .unwrap()
    // }
    async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
        let query = <Self::EntityType as EntityTrait>::find().filter(condition);
        // .into();
        PaginatorTrait::count(query, db).await.unwrap()
        // <Self::EntityType as PaginatorTrait>::count(query, db)
        // .await
        // .unwrap()
    }

    // async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
    //     // Explicitly annotate the query type so that the compiler knows which method to use.
    //     let query: sea_orm::Select<Self::EntityType> =
    //         <Self::EntityType as EntityTrait>::find().filter(condition);
    //     // Fully qualify the call to use QuerySelect::count (which consumes `query`)
    //     <sea_orm::Select<Self::EntityType> as PaginatorTrait<Self::EntityType>>::count(query, db)
    //         .await
    //         .unwrap()
    // }

    fn default_index_column() -> Self::ColumnType;
    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)];
    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)];
}

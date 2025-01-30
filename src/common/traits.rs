use async_trait::async_trait;
use axum::response::IntoResponse;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait, Order};
use uuid::Uuid;

// Define a trait that encapsulates the necessary operations
#[async_trait]
pub trait ApiResource: Sized {
    type EntityType: EntityTrait;
    type ColumnType: ColumnTrait;
    type ModelType: ModelTrait;
    type ActiveModelType: sea_orm::ActiveModelTrait;
    type ApiModel: IntoResponse + From<Self::ModelType>;
    type CreateModel: Into<Self::ActiveModelType>;
    type UpdateModel: Send + Sync;

    const RESOURCE_NAME: &str;

    // Function to get all records with filtering, sorting, and pagination
    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr>;

    // Function to get a single record by ID
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

    // Function to delete a record by ID
    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr>;
    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr>;
    async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64;

    // let total_count: u64 = <T::EntityType as EntityTrait>::find()
    //     .filter(condition.clone())
    //     .select_only()
    //     .column(T::default_index_column())
    //     .count(&db)
    //     .await
    //     .unwrap_or(0);
    fn default_index_column() -> Self::ColumnType;
    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)];
    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)];
}

use async_trait::async_trait;
use axum::response::IntoResponse;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ModelTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Select,
};
use uuid::Uuid;

// Define a trait that encapsulates the necessary operations
#[async_trait]
pub trait ApiResource: Sized {
    // Associated types for Entity, Model, and ActiveModel
    type EntityType: EntityTrait;
    type ModelType: ModelTrait;
    type ActiveModelType: ActiveModelTrait<Entity = Self::EntityType>;

    // Associated type for API response model
    type ApiModel: From<Self::EntityType> + IntoResponse + serde::Serialize;

    // Function to get all records with filtering, sorting, and pagination
    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: impl ColumnTrait,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self::ApiModel>;

    // Function to get a single record by ID
    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Option<Self::ApiModel>;

    // Function to insert a new record
    async fn create(
        db: &DatabaseConnection,
        active_model: Self::ActiveModelType,
    ) -> Result<Self::ApiModel, sea_orm::DbErr>;

    // Function to update an existing record
    async fn update(
        db: &DatabaseConnection,
        active_model: Self::ActiveModelType,
    ) -> Result<Self::ApiModel, sea_orm::DbErr>;

    // Function to delete a record by ID
    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, sea_orm::DbErr>;

    fn default_sort_column() -> impl sea_orm::ColumnTrait;
    fn sortable_columns<'a>() -> &'a [(&'a str, impl sea_orm::ColumnTrait)];
    fn filterable_columns<'a>() -> &'a [(&'a str, impl sea_orm::ColumnTrait)];
}

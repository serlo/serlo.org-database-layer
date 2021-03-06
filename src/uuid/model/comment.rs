use async_trait::async_trait;
use futures::join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::format_alias;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub author_id: i32,
    pub title: Option<String>,
    pub date: DateTime,
    pub archived: bool,
    pub content: String,
    pub parent_id: i32,
    pub children_ids: Vec<i32>,
}

macro_rules! fetch_one_comment {
    ($id: expr, $executor: expr) => {
       sqlx::query!(
            r#"
                SELECT u.trashed, c.author_id, c.title, c.date, c.archived, c.content, c.parent_id, c.uuid_id, p.title as parent_title
                    FROM comment c
                    LEFT JOIN comment p ON p.id = c.parent_id
                    JOIN uuid u ON u.id = c.id
                    WHERE c.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_children {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT id
                    FROM comment
                    WHERE parent_id = ?
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_comment {
    ($id: expr, $comment: expr, $children: expr, $context: expr) => {{
        let comment = $comment.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let children = $children?;
        let context = $context?;

        Ok(Uuid {
            id: $id,
            trashed: comment.trashed != 0,
            alias: format_alias(
                context.as_deref(),
                $id,
                Some(
                    comment
                        .title
                        .as_ref()
                        .or_else(|| comment.parent_title.as_ref())
                        .unwrap_or(&format!("{}", $id))
                        .as_str(),
                ),
            ),
            concrete_uuid: ConcreteUuid::Comment(Comment {
                __typename: "Comment".to_string(),
                author_id: comment.author_id as i32,
                title: comment.title,
                date: comment.date.into(),
                archived: comment.archived != 0,
                content: comment.content.unwrap_or_else(|| "".to_string()),
                parent_id: comment.parent_id.or(comment.uuid_id).unwrap() as i32,
                children_ids: children.iter().map(|child| child.id as i32).collect(),
            }),
        })
    }};
}

#[async_trait]
impl UuidFetcher for Comment {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let comment = fetch_one_comment!(id, pool);
        let children = fetch_all_children!(id, pool);
        let context = Comment::fetch_context(id, pool);

        let (comment, children, context) = join!(comment, children, context);

        to_comment!(id, comment, children, context)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let comment = fetch_one_comment!(id, &mut transaction).await;
        let children = fetch_all_children!(id, &mut transaction).await;
        let context = Comment::fetch_context_via_transaction(id, &mut transaction).await;

        transaction.commit().await?;

        to_comment!(id, comment, children, context)
    }
}

impl Comment {
    pub async fn fetch_context(id: i32, pool: &MySqlPool) -> Result<Option<String>, UuidError> {
        let object = sqlx::query!(
            r#"
                SELECT uuid_id as id
                    FROM (
                        SELECT id, uuid_id FROM comment c
                        UNION ALL
                        SELECT c.id, p.uuid_id FROM comment p LEFT JOIN comment c ON c.parent_id = p.id
                    ) t
                    WHERE id = ? AND uuid_id IS NOT NULL
            "#,
            id
        )
        .fetch_one(pool).await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        Uuid::fetch_context(object.id.unwrap() as i32, pool).await
    }

    pub async fn fetch_context_via_transaction<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<String>, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let object = sqlx::query!(
            r#"
                SELECT uuid_id as id
                    FROM (
                        SELECT id, uuid_id FROM comment c
                        UNION ALL
                        SELECT c.id, p.uuid_id FROM comment p LEFT JOIN comment c ON c.parent_id = p.id
                    ) t
                    WHERE id = ? AND uuid_id IS NOT NULL
            "#,
            id
        )
        .fetch_one(&mut transaction).await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let context =
            Uuid::fetch_context_via_transaction(object.id.unwrap() as i32, &mut transaction)
                .await?;
        transaction.commit().await?;
        Ok(context)
    }
}

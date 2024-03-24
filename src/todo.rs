use crate::error::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};

#[derive(Deserialize)]
pub struct CreateTodo {
    body: String,
}

// We don't need to construct a CreateTodo; we just need to deserialize it when we receive one in an API call.
impl CreateTodo {
    pub fn body(&self) -> &str {
        self.body.as_ref()
    }
}

// We don't need to construct a UpdateTodo; we just need to deserialize it when we receive one in an API call.
#[derive(Deserialize)]
pub struct UpdateTodo {
    body: String,
    completed: bool,
}

impl UpdateTodo {
    pub fn body(&self) -> &str {
        self.body.as_ref()
    }

    pub fn completed(&self) -> bool {
        self.completed
    }
}

// We're deriving the Serialize trait from the serde crate and sqlx::FromRow,
// which allows us to get a `Todo` from a SQLx query.
#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i64,
    body: String,
    completed: bool,
    // We use the chrono::NaiveDateTime type to map SQL timestamp into Rust objects.
    created_at: NaiveDateTime,
}

impl Todo {
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Todo>, Error> {
        // Selects all todos from the todos table
        query_as("select * from todos")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn read(dbpool: SqlitePool, id: i64) -> Result<Todo, Error> {
        // Selects one todo from the todos table with a matching id field
        query_as("select * from todos where id = ?")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    // We've added a new type here, CreateTodo, which we haven't defined yet.
    // It contains the todo body, which we need to create a todo.
    pub async fn create(dbpool: SqlitePool, new_todo: CreateTodo) -> Result<Todo, Error> {
        // We use the returning * SQL cause to retrieve the record immediately after it's inserted.
        query_as("insert into todos (body) values (?) returning *")
            .bind(new_todo.body())
            // We execute the query with fetch_one() because we expect this to return one row.
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    // We've added another new type here, UpdateTodo, which contains the two fields we allow to be updated.
    pub async fn update(
        dbpool: SqlitePool,
        id: i64,
        updated_todo: UpdateTodo,
    ) -> Result<Todo, Error> {
        // We're using the returning * SQL clause to retrieve the updated record immediately. Notice how we set the updated_at
        // field to the current date and time.
        query_as("update todos set body = ?, completed = ?, updated_at = datetime('now') where id = ? returning *")
            // Each value is bound in the order they're declared within the SQL statement, using the ? token to bind values.
            // This syntax varies, depending on the SQL implementation.
            // When we use bind() to bind values to the SQL statement, we need to pay attention to the order of the values because
            // they're bound in the order they're specified.
            .bind(updated_todo.body())
            .bind(updated_todo.completed())
            .bind(id)
            // We expect to fetch one row when this query is executed.
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(dbpool: SqlitePool, id: i64) -> Result<(), Error> {
        // The delete is destructive; nothing is left to return if it succeeds.
        query("delete from todos where id = ?")
            .bind(id)
            // Here, we use execute() to execute the query, which is used for queries that don't return records.
            .execute(&dbpool)
            .await?;
        // We return unit upon success(i.e., no previous errors).
        Ok(())
    }
}

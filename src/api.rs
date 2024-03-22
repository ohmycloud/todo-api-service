use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;
use crate::error::Error;
use crate::todo::{CreateTodo, Todo, UpdateTodo};

pub async fn ping(
    // The State extractor gives us the database connection pool from the axum state.
    State (dbpool): State<SqlitePool>,
) -> Result<String, Error> {
    use sqlx::Connection;

    // We need to acquire a connection from the database pool first.
    let mut conn = dbpool.acquire().await?;

    // The ping() method will check if the database connection is OK
    // In the case of SQLite, this checks that the SQLite background threads are alive.
    conn.ping()
        .await
        // Upon success, ping() returns unit, so we just map it to the string ok, which is returned as our response.
        .map(|_| "ok".to_string())
        // We use the From trait to map sqlx::Error to our error types.
        .map_err(Into::into)
}

pub async fn todo_list(
    State(dbpool): State<SqlitePool>,
) -> Result<Json<Vec<Todo>>, Error> { // Note how we're returning a JSON object of Vec<Todo> or, possibly, an error.
    // The Todo::list() method returns a plain Vec<Todo>, so we map that to a Json object using Json::from,
    // which relies on the Serialize trait we derived for Todo
    Todo::list(dbpool).await.map(Json::from)
}

pub async fn todo_read(
    State(dbpool): State<SqlitePool>,
    // A path parameter, which we access using the Path extractor. axum takes care of mapping the ID from the /v1/todos/:id router path
    // to the named parameter in a type-safe manner.
    Path(id): Path<i64>,
) -> Result<Json<Todo>, Error> {
    Todo::read(dbpool, id).await.map(Json::from)
}

pub async fn todo_create(
    State(dbpool): State<SqlitePool>,
    // Here, we introduce the CreateTodo struct, which we're getting from the request body using
    // the Json extractor, which uses the Deserialize implementation we derived using the serde crate.
    Json(new_todo): Json<CreateTodo>,
) -> Result<Json<Todo>, Error> {
    Todo::create(dbpool, new_todo).await.map(Json::from)
}

pub async fn todo_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<i64>,
    // The UpdateTodo struct which we're getting from the request body using the Json extractor,
    // which uses the Deserialize implementation we derived using the serde crate.
    Json(updated_todo): Json<UpdateTodo>,
) -> Result<Json<Todo>, Error> {
    Todo::update(dbpool, id, updated_todo).await.map(Json::from)
}

pub async fn todo_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), Error> {
    Todo::delete(dbpool, id).await
}
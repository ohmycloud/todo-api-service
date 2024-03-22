pub async fn create_router(
    // the database pool is passed into the router, which takes ownership
    dbpool: sqlx::Pool<sqlx::Sqlite>,
) -> axum::Router {
    use crate::api::{
        ping, todo_create, todo_delete, todo_list, todo_read, todo_update,
    };
    use axum::{routing::get, Router};
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::TraceLayer;

    Router::new()
        // our liveness health check merely returns a 200 status with the body ok.
        .route("/alive", get(|| async { "ok" }))
        // Our readiness health check makes a GET request with the ping() handler.
        .route("/ready", get(ping))
        // The API routes are nested under the /v1 path.
        .nest(
            "/v1",
            Router::new()
                // Here, we permit two methods for the /v1/todos path - either GET or POST
                // which call the todo_list() and todo_create() handlers, respectively.
                // We can change the methods together using a handy fluent interface.
                .route("/todos", get(todo_list).post(todo_create))
                // The path parameter :id maps to the todo's ID. GET, PUT, or DELETE methods for /v1/todos/:id
                // map to todo_read(), todo_update(), and todo_delete, respectively.
                .route(
                    "/todos/:id",
                    get(todo_read).put(todo_update)
                        .delete(todo_delete),
                ),
        )
        // We hand the database connection pool off to the router to be passed into handlers as state
        .with_state(dbpool)
        // A CORS layer is added to demonstrate how to apply CORS headers
        .layer(CorsLayer::new().allow_methods(Any)
            .allow_origin(Any))
        // We need to add the HTTP tracing layer from tower_http to get request traces.
        .layer(TraceLayer::new_for_http())
}
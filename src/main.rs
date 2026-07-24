#[cfg(feature = "ssr")]
async fn connect_database(database_url: &str) -> sqlx::PgPool {
    use leptos::logging::{error, warn};
    use std::time::Duration;

    const MAX_RETRIES: u32 = 3;
    const RETRY_INTERVAL: Duration = Duration::from_secs(10);
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

    let pool_options = || {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(CONNECT_TIMEOUT)
    };

    match pool_options().connect(database_url).await {
        Ok(pool) => return pool,
        Err(err) => {
            warn!(
                "Database not available: {err}. Retrying up to {MAX_RETRIES} times every {}s.",
                RETRY_INTERVAL.as_secs()
            );
        }
    }

    for attempt in 1..=MAX_RETRIES {
        tokio::time::sleep(RETRY_INTERVAL).await;

        match pool_options().connect(database_url).await {
            Ok(pool) => return pool,
            Err(err) if attempt == MAX_RETRIES => {
                error!(
                    "Database unavailable after {MAX_RETRIES} retries: {err}. Shutting down."
                );
                std::process::exit(1);
            }
            Err(err) => {
                warn!("Database not available (retry {attempt}/{MAX_RETRIES}): {err}");
            }
        }
    }

    unreachable!()
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use churchrm::app::*;
    use churchrm::state::AppState;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = connect_database(&database_url).await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let app_state = AppState::new(pool);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}

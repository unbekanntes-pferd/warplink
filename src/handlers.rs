pub mod links {
    use axum::{
        extract::{Path, State, Host},
        response::{Html, IntoResponse, Redirect},
        Form,
    };
    use tracing::{debug, error, info};
    use url::Url;

    use crate::models::{
        CreateWarpLinkRequest, WarpLink, WarpLinkError, WarpLinkErrorResponse, WarpLinkState,
    };

    fn generate_short_link() -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    async fn get_short_link(
        state: WarpLinkState,
        short_link: &str,
    ) -> Result<Option<WarpLink>, WarpLinkError> {
        sqlx::query_as!(
            WarpLink,
            "
                SELECT * FROM warp_link WHERE short_link = $1
            ",
            short_link
        )
        .fetch_optional(state.pool())
        .await
        .map_err(|err| WarpLinkError::DatabaseError(err.to_string()))
    }

    async fn insert_short_link(
        state: WarpLinkState,
        payload: CreateWarpLinkRequest,
    ) -> Result<WarpLink, WarpLinkError> {
        let short_link = loop {
            let short_link = generate_short_link();
            let link = get_short_link(state.clone(), &short_link).await?;

            if let None = link {
                break short_link;
            }
        };

        sqlx::query_as!(
            WarpLink,
            "
                INSERT INTO warp_link (short_link, long_link)
                VALUES ($1, $2)
                RETURNING *
            ",
            short_link,
            payload.long_link
        )
        .fetch_one(state.pool())
        .await
        .map_err(|err| WarpLinkError::DatabaseError(err.to_string()))
    }

    pub async fn register_short_link(
        State(state): State<WarpLinkState>,
        Host(host): Host,
        Form(payload): Form<CreateWarpLinkRequest>,
    ) -> Result<Html<String>, WarpLinkErrorResponse> {
        // handle invalid url
        if let Err(err) = Url::parse(payload.long_link.as_str()) {
            error!("Invalid url: {}", err);
            let details = format!("Invalid url: {} ({})", payload.long_link, err);
            return Err(WarpLinkErrorResponse::new_bad_request(Some(details)));
        }

        let parsed_url = Url::parse(payload.long_link.as_str()).unwrap();

        // handle invalid scheme
        if !parsed_url.scheme().starts_with("http") {
            error!("Invalid scheme: {} (http(s) required)", parsed_url.scheme());
            let details = format!("Invalid scheme: {}", parsed_url.scheme());
            return Err(WarpLinkErrorResponse::new_bad_request(Some(details)));
        }

        let warp_link = insert_short_link(state, payload).await;

        if let Ok(link) = warp_link {
            info!("Created new link with id {}", link.short_link);

            let protocol = if host.starts_with("localhost") {
                "http"
            } else {
                "https"
            };

            let full_link = format!("{}://{}/{}", protocol, host, link.short_link);

            let html_res = format!(
                r##"
                <div class="result-box">
                <a href="{}" target="_blank">
                <span id="shortLink">
                {}
                </span>
                </a>
                </div>
                "##,
                full_link,
                full_link
            );
            Ok(Html(html_res))
        } else {
            let err = warp_link.unwrap_err();
            error!("Database error: {}", err);
            let details = format!("Database error: {}", err);
            Err(WarpLinkErrorResponse::new_internal_error(Some(details)))
        }
    }

    pub async fn redirect_to_long_link(
        State(state): State<WarpLinkState>,
        Path(short_link): Path<String>,
    ) -> Result<impl IntoResponse, WarpLinkErrorResponse> {
        let warp_link = get_short_link(state, &short_link).await;

        let warp_link = if let Ok(link) = warp_link {
            link
        } else {
            let err = warp_link.unwrap_err();
            error!("Database error: {}", err);
            let details = format!("Database error: {}", err);
            return Err(WarpLinkErrorResponse::new_internal_error(Some(details)))
        };

        if let Some(link) = warp_link {
            debug!("Redirecting to {}", link.long_link);
            Ok(Redirect::to(link.long_link.as_str()))
        } else {
            error!("Link with id {} not found.", short_link);
            let details = format!("Link with id {} not found.", short_link);
            Err(WarpLinkErrorResponse::new_not_found(Some(details)))
        }
    }
}

pub mod health {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Json};
    use serde::{Deserialize, Serialize};
    use tracing::{debug, error, info, warn};

    use crate::models::WarpLinkState;

    #[derive(Deserialize, Serialize)]
    pub enum Status {
        #[serde(rename = "pass")]
        Ok,
        #[serde(rename = "warn")]
        Degraded,
        #[serde(rename = "fail")]
        Unavailable,
    }

    impl Status {
        pub fn ok() -> Self {
            Status::Ok
        }

        pub fn degraded() -> Self {
            Status::Degraded
        }

        pub fn unavailable() -> Self {
            Status::Unavailable
        }
    }

    #[derive(Serialize)]
    pub struct Health {
        pub status: Status,
    }

    pub async fn health_check(State(state): State<WarpLinkState>) -> impl IntoResponse {
        debug!("Checking health.");

        if state.pool().is_closed() {
            error!("Database connection pool is closed.");
            let health = Status::unavailable();
            return (StatusCode::SERVICE_UNAVAILABLE, Json(health));
        }

        if state.pool().try_acquire().is_none() {
            warn!("Database connection pool is full.");
            let health = Status::degraded();
            return (StatusCode::OK, Json(health));
        }

        let health = Status::ok();
        info!("Health check OK.");
        (StatusCode::OK, Json(health))
    }
}

pub mod pages {
    use axum::response::Html;

    pub async fn index() -> Html<String> {
        let content = include_str!("../pages/index.html");

        Html(content.to_string())
    }
}

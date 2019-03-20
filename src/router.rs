use actix_web::{http::Method, App};

use crate::controller;
use crate::util::async_await::compat;
use crate::AppState;

pub fn router(app: App<AppState>) -> App<AppState> {
    app.prefix("/api/v1")
        .route(
            "/users/login",
            Method::GET,
            compat(controller::users::login),
        ).route(
            "/users/{id}/metadata",
            Method::GET,
            compat(controller::users::metadata::show_user),
        ).route(
            "/users/tokens",
            Method::GET,
            compat(controller::users::token::list_tokens),
        ).route(
            "/users/tokens/create",
            Method::PUT,
            compat(controller::users::token::create_token),
        ).route(
            "/users/tokens/{token_id}",
            Method::DELETE,
            compat(controller::users::token::remove_token),
        ).route(
            "/packages/search",
            Method::GET,
            compat(controller::packages::search),
        ).route(
            "/packages/groups",
            Method::GET,
            compat(controller::packages::metadata::list_groups),
        ).route(
            "/packages/{group}/metadata",
            Method::GET,
            compat(controller::packages::metadata::show_group),
        ).route(
            "/packages/{group}/packages",
            Method::GET,
            compat(controller::packages::metadata::list_packages),
        ).route(
            "/packages/{group}/{package}/metadata",
            Method::GET,
            compat(controller::packages::metadata::show_package),
        ).route(
            "/packages/{group}/{package}/versions",
            Method::GET,
            compat(controller::packages::metadata::list_versions),
        ).route(
            "/packages/{group}/{package}/owners",
            Method::GET,
            compat(controller::packages::metadata::list_owners),
        ).route(
            "/packages/{group}/{package}/{version}/metadata",
            Method::GET,
            compat(controller::packages::metadata::show_version),
        ).route(
            "/packages/{group}/{package}/{version}/readme",
            Method::GET,
            compat(controller::packages::metadata::show_readme),
        ).route(
            "/packages/{group}/{package}/{version}/dependencies",
            Method::GET,
            compat(controller::packages::metadata::list_dependencies),
        ).route(
            "/packages/{group}/{package}/{version}/download",
            Method::GET,
            compat(controller::packages::download),
        ).route(
            "/packages/{group}/{package}/{version}/yank",
            Method::PATCH,
            compat(controller::packages::yank),
        ).route(
            "/packages/{group}/{package}/{version}/publish",
            Method::PUT,
            compat(controller::packages::publish),
        )
    // .route(
    //     "/packages/{group}/{package}/{version}/downloads",
    //     Method::GET,
    //     package::controller::download,
}

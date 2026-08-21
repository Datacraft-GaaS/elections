use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "eel-boundary-svc",
        version = "1.0.0",
        description = "Module M18: Boundary Transparency. Public and admin API for eel boundary data."
    ),
    paths(
        crate::handlers::list_constituencies,
        crate::handlers::get_constituency,
        crate::handlers::list_wards,
        crate::handlers::list_counties,
        crate::handlers::list_stations,
        crate::handlers::get_station,
        crate::handlers::list_boundary_changes,
        crate::handlers::publish_boundary_snapshot,
    ),
    tags(
        (name = "constituencies", description = "Constituency boundary data"),
        (name = "wards", description = "Ward boundary data"),
        (name = "counties", description = "County boundary data"),
        (name = "stations", description = "Polling station data"),
        (name = "changes", description = "Boundary change history"),
        (name = "admin", description = "Boundary publishing (restricted)"),
    )
)]
pub struct BoundaryApiDoc;

// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

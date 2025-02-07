// Copyright 2023 Zinc Labs Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use actix_web::http;

use crate::{
    common::{
        infra::config::ALERTS_DESTINATIONS,
        meta::{alerts::templates::Template, authz::Authz},
        utils::auth::{remove_ownership, set_ownership},
    },
    service::db,
};

pub async fn save(org_id: &str, name: &str, mut template: Template) -> Result<(), anyhow::Error> {
    if template.body.is_empty() {
        return Err(anyhow::anyhow!("Alert template body empty"));
    }
    if !name.is_empty() {
        template.name = name.to_owned();
    }
    if template.name.is_empty() {
        return Err(anyhow::anyhow!("Alert template name is required"));
    }

    match db::alerts::templates::set(org_id, &mut template).await {
        Ok(_) => {
            if name.is_empty() {
                set_ownership(org_id, "templates", Authz::new(&template.name)).await;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn get(org_id: &str, name: &str) -> Result<Template, anyhow::Error> {
    db::alerts::templates::get(org_id, name)
        .await
        .map_err(|_| anyhow::anyhow!("Alert template not found"))
}

pub async fn list(org_id: &str) -> Result<Vec<Template>, anyhow::Error> {
    db::alerts::templates::list(org_id).await
}

pub async fn delete(org_id: &str, name: &str) -> Result<(), (http::StatusCode, anyhow::Error)> {
    for dest in ALERTS_DESTINATIONS.iter() {
        if dest.key().starts_with(org_id) && dest.value().template.eq(&name) {
            return Err((
                http::StatusCode::FORBIDDEN,
                anyhow::anyhow!(
                    "Alert template is in use for destination {}",
                    &dest.value().name.clone()
                ),
            ));
        }
    }

    if db::alerts::templates::get(org_id, name).await.is_err() {
        return Err((
            http::StatusCode::NOT_FOUND,
            anyhow::anyhow!("Alert template not found {}", name),
        ));
    }
    match db::alerts::templates::delete(org_id, name).await {
        Ok(_) => {
            remove_ownership(org_id, "templates", Authz::new(name)).await;
            Ok(())
        }
        Err(e) => Err((http::StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

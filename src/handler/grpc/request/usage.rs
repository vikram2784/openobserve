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

use async_trait::async_trait;
use config::CONFIG;
use tonic::{Request, Response, Status};

use crate::handler::grpc::cluster_rpc::{usage_server::Usage, UsageRequest, UsageResponse};

#[derive(Debug, Default)]
pub struct UsageServerImpl;

#[async_trait]
impl Usage for UsageServerImpl {
    async fn report_usage(
        &self,
        request: Request<UsageRequest>,
    ) -> Result<Response<UsageResponse>, Status> {
        let metadata = request.metadata().clone();
        let req = request.into_inner();
        let report_to_stream = req.stream_name;
        let report_to_org_id = metadata.get(&CONFIG.grpc.org_header_key);
        let in_data = req.data.unwrap_or_default();
        log::info!("UsageServer: report_usage received data");
        let resp = crate::service::logs::otlp_grpc::usage_ingest(
            report_to_org_id.unwrap().to_str().unwrap(),
            &report_to_stream,
            in_data.data.into(),
            0_usize,
        )
        .await;

        match resp {
            Ok(_) => {
                let reply = UsageResponse {
                    status_code: 200,
                    message: "OK".to_string(),
                };
                Ok(Response::new(reply))
            }
            Err(err) => {
                let reply = UsageResponse {
                    status_code: 500,
                    message: err.to_string(),
                };
                Ok(Response::new(reply))
            }
        }
    }
}

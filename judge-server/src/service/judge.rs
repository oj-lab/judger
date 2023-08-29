use std::{fs, path::PathBuf};

use crate::error::ServiceError;
use actix_web::{post, web, HttpResponse};
use judge_core::{
    compiler::Language,
    judge::{
        self,
        builder::{JudgeBuilder, JudgeBuilderInput},
        JudgeConfig,
    },
    package::PackageType,
};
use utoipa::ToSchema;

#[derive(utoipa::OpenApi)]
#[openapi(paths(run_judge), components(schemas(RunJudgeBody)))]
pub struct JudgeApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/judge").service(run_judge));
}

// TODO: Remove the first `_` when the segment is actually used
#[derive(Debug, ToSchema, Deserialize)]
pub struct RunJudgeBody {
    src: String,
    src_language: Language,
    package_slug: PathBuf,
}

#[utoipa::path(
    context_path = "/api/v1/judge",
    request_body(content = RunJudgeBody, content_type = "application/json", description = "The info a judge task should refer to"),
    responses(
        (status = 200, description = "Judge run successfully")
    )
)]
#[post("")]
pub async fn run_judge(body: web::Json<RunJudgeBody>) -> Result<HttpResponse, ServiceError> {
    log::debug!("receive body: {:?}", body);

    let uuid = uuid::Uuid::new_v4();
    let runtime_path = PathBuf::from("/tmp").join(uuid.to_string());
    fs::create_dir_all(runtime_path.clone()).unwrap();
    fs::write(runtime_path.clone().join("src"), body.src.clone()).unwrap();

    tokio::spawn(async move {
        let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
            package_type: PackageType::ICPC,
            package_path: body.package_slug.clone(),
            runtime_path: runtime_path.clone(),
            src_language: body.src_language,
            src_path: runtime_path.clone().join("src"),
        });
        if new_builder_result.is_err() {
            println!(
                "Failed to new builder result: {:?}",
                new_builder_result.err()
            );
            return;
        }
        let builder = new_builder_result.unwrap();
        println!("Builder created: {:?}", builder);
        for idx in 0..builder.testdata_configs.len() {
            let judge_config = JudgeConfig {
                test_data: builder.testdata_configs[idx].clone(),
                program: builder.program_config.clone(),
                checker: builder.checker_config.clone(),
                runtime: builder.runtime_config.clone(),
            };
            let result = judge::common::run_judge(&judge_config);
            println!("Judge result: {:?}", result);
        }

        println!("BatchJudge finished")
    });

    Ok(HttpResponse::Ok().finish())
}

use std::{fs, path::PathBuf};

use crate::error::ServiceError;
use actix_web::{post, web, HttpResponse};

use judge_core::{
    compiler::Language,
    error::JudgeCoreError,
    judge::{
        self,
        builder::{JudgeBuilder, JudgeBuilderInput},
        result::JudgeResultInfo,
        JudgeConfig,
    },
    package::PackageType,
};
use tokio::task::JoinHandle;
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
    src_language: Language
}

#[utoipa::path(
    context_path = "/api/v1/judge",
    request_body(content = RunJudgeBody, content_type = "application/json", description = "The info a judge task should refer to"),
    responses(
        (status = 200, description = "Judge run successfully")
    )
)]
#[post("/{package_slug}")]
pub async fn run_judge(
    path: web::Path<String>,
    body: web::Json<RunJudgeBody>,
    problem_package_dir: web::Data<PathBuf>,
) -> Result<HttpResponse, ServiceError> {
    let package_slug = path.into_inner();
    log::debug!("receive body: {:?}", body);

    let uuid = uuid::Uuid::new_v4();
    let runtime_path = PathBuf::from("/tmp").join(uuid.to_string());
    let src_file_name = format!("src.{}", body.src_language.get_extension());
    println!("runtime_path: {:?}", runtime_path);
    fs::create_dir_all(runtime_path.clone()).map_err(|e| {
        println!("Failed to create runtime dir: {:?}", e);
        ServiceError::InternalError(anyhow::anyhow!("Failed to create runtime dir"))
    })?;
    fs::write(runtime_path.clone().join(&src_file_name), body.src.clone()).map_err(
        |e| {
            println!("Failed to write src file: {:?}", e);
            ServiceError::InternalError(anyhow::anyhow!("Failed to write src file"))
        },
    )?;

    let handle: JoinHandle<Result<Vec<JudgeResultInfo>, JudgeCoreError>> =
        tokio::spawn(async move {
            let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
                package_type: PackageType::ICPC,
                package_path: problem_package_dir.join(package_slug.clone()),
                runtime_path: runtime_path.clone(),
                src_language: body.src_language,
                src_path: runtime_path.clone().join(&src_file_name),
            });
            if new_builder_result.is_err() {
                println!(
                    "Failed to new builder result: {:?}",
                    new_builder_result.err()
                );
                return Ok(vec![]);
            }
            let builder = new_builder_result?;
            println!("Builder created: {:?}", builder);
            let mut results: Vec<JudgeResultInfo> = vec![];
            for idx in 0..builder.testdata_configs.len() {
                let judge_config = JudgeConfig {
                    test_data: builder.testdata_configs[idx].clone(),
                    program: builder.program_config.clone(),
                    checker: builder.checker_config.clone(),
                    runtime: builder.runtime_config.clone(),
                };
                let result = judge::common::run_judge(&judge_config)?;
                println!("Judge result: {:?}", result);
                results.push(result);
            }

            println!("BatchJudge finished");
            Ok(results)
        });

    match handle.await.unwrap() {
        Ok(results) => Ok(HttpResponse::Ok().json(results)),
        Err(e) => {
            println!("Failed to await handle: {:?}", e);
            Err(ServiceError::InternalError(anyhow::anyhow!("Judge failed")))
        }
    }
}

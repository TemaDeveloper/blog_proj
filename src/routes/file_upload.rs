
use std::time::Duration;

use aws_sdk_s3::{primitives::ByteStream, Client};
use axum::{body::Bytes, extract::{DefaultBodyLimit, Multipart, State}, response::{IntoResponse, Response}, routing::post, Router};
use dotenv::dotenv;
use tower::ServiceBuilder;
use http::{header, StatusCode};
use ::serde::Serialize;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};
use uuid::Uuid;

pub async fn configure_aws_s3_client() -> Client{
    dotenv().ok();
    let aws_configuration = aws_config::load_from_env().await;
    let aws_s3_client = aws_sdk_s3::Client::new(&aws_configuration);
    aws_s3_client
}

pub async fn upload_router() -> Router{

    let aws_client = configure_aws_s3_client().await;
    Router::new()
        .route("/upload", post(upload_hander))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1023))
        .layer(ServiceBuilder::new().layer(TimeoutLayer::new(Duration::from_secs(120)))) // Set a 2-minute timeout
        .with_state(aws_client)

}

#[derive(Serialize)]
struct File {
  key: String,
  successful: bool,
  url: String,
  file_name: String,
  content_type: String,
  #[serde(skip_serializing)]
  bytes: Bytes,
}

async fn upload_hander(
  State(s3_client): State<aws_sdk_s3::Client>,
  mut multipart: Multipart,
) -> Result<Response, Response> {
  let mut files = vec![];
  let bucket_name = std::env::var("AWS_S3_BUCKET").unwrap_or_default();

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| {
        eprintln!("Error reading multipart field: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong uploading a file").into_response()
    })?
  {
    if let Some("files") = field.name() {
      let file_name = field.file_name().unwrap_or_default().to_owned();
      let content_type = field.file_name().unwrap_or_default().to_owned();
      let key = Uuid::new_v4().to_string();
      let url = format!("https://{bucket_name}.s3.amazonaws.com/{key}");
    
      println!("{url}");

      let bytes = field
        .bytes()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Was not converted to bytes").into_response())?;

      files.push(File {
        file_name,
        content_type,
        bytes,
        key,
        url,
        successful: false,
      })
    }
  }

  for file in &mut files {
    let body = ByteStream::from(file.bytes.to_vec());

    let res = s3_client
      .put_object()
      .bucket(&bucket_name)
      .content_type(&file.content_type)
      .content_length(file.bytes.len() as i64)
      .key(&file.key)
      .body(body)
      .send()
      .await;

    file.successful = res.is_ok();
  }

  Ok(
    (
      StatusCode::OK,
      [(header::CONTENT_TYPE, "application/json")],
      serde_json::json!(files).to_string(),
    )
      .into_response(),
  )
}
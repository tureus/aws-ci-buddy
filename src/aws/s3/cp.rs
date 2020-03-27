use std::convert::TryInto;

use rusoto_core::{ByteStream, Region};
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use tokio::io::AsyncWriteExt;
use tokio::stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

use crate::EcrDockerLoginError;

#[derive(PartialEq)]
enum Direction {
    ToS3,
    FromS3,
}

pub async fn s3_cp(src: &str, tgt: &str) -> Result<(), EcrDockerLoginError> {
    let client = S3Client::new(Region::default());

    let direction = match (src.starts_with("s3://"), tgt.starts_with("s3://")) {
        (true, false) => Direction::FromS3,
        (false, true) => Direction::ToS3,
        (_, _) => {
            return Err(EcrDockerLoginError::Other(format!(
                "one of source or target must be an S3 path"
            )))
        }
    };

    match direction {
        Direction::FromS3 => from_s3(client, src, tgt).await?,
        Direction::ToS3 => to_s3(client, src, tgt).await?,
    };

    Ok(())
}

async fn from_s3(client: S3Client, src: &str, tgt: &str) -> Result<(), EcrDockerLoginError> {
    let src_url: url::Url = Url::parse(src)?;

    let bucket = if let Some(host) = src_url.host() {
        host.to_string()
    } else {
        return Err(EcrDockerLoginError::Other("src url missing host".into()));
    };

    let key = src_url.path();
    if key.len() == 0 {
        return Err(EcrDockerLoginError::Other(
            "src url missing path (aka, no S3 key)".into(),
        ));
    }
    let key = super::clean_prefix(key);
    let key = if let Some(key) = key {
        key
    } else {
        return Err(EcrDockerLoginError::Other(format!("must provide key")));
    };

    let res = client
        .get_object(GetObjectRequest {
            bucket,
            key,
            ..Default::default()
        })
        .await?;

    let mut streaming_body = res.body.unwrap();

    use tokio::fs::File;
    let mut file = File::create(tgt).await?;

    while let Some(Ok(chunk)) = streaming_body.next().await {
        file.write(&chunk[..]).await?;
    }

    Ok(())
}

async fn to_s3(client: S3Client, src: &str, tgt: &str) -> Result<(), EcrDockerLoginError> {
    let tgt_url: url::Url = Url::parse(tgt)?;

    let bucket = if let Some(host) = tgt_url.host() {
        host.to_string()
    } else {
        return Err(EcrDockerLoginError::Other("src url missing host".into()));
    };

    let key = tgt_url.path();
    if key.len() == 0 {
        return Err(EcrDockerLoginError::Other(
            "src url missing path (aka, no S3 key)".into(),
        ));
    }
    let key = super::clean_prefix(key);
    let key = if let Some(key) = key {
        key
    } else {
        return Err(EcrDockerLoginError::Other(format!("must provide key")));
    };

    use tokio::fs::File;
    let file = File::open(src).await?;
    let metadata = file.metadata().await?;
    let file_len = (metadata.len() as u64).try_into().unwrap();
    let file_reader =
        FramedRead::new(file, BytesCodec::new()).map(|chunk| chunk.map(|x| x.freeze()));
    let file_stream = ByteStream::new(file_reader);

    client
        .put_object(PutObjectRequest {
            bucket,
            key,
            body: Some(file_stream),
            content_length: Some(file_len),
            ..Default::default()
        })
        .await?;

    Ok(())
}

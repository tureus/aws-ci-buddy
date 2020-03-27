use crate::EcrDockerLoginError;
use rusoto_core::Region;
use rusoto_s3::{ListObjectsV2Request, S3Client, S3};
use url::Url;

pub async fn s3_ls(path: Option<&str>) -> Result<(), EcrDockerLoginError> {
    if let Some(path) = path {
        s3_ls_objects(path).await?;
    } else {
        s3_ls_buckets().await?;
    }

    Ok(())
}

async fn s3_ls_objects(path: &str) -> Result<(), EcrDockerLoginError> {
    let path = if path.starts_with("s3://") {
        path.to_string()
    } else {
        format!("s3://{}", path)
    };

    let url = Url::parse(&path[..])?;

    let bucket = if let Some(host) = url.host() {
        host.to_string()
    } else {
        return Err(EcrDockerLoginError::Other(format!(
            "path must have a bucket name"
        )));
    };

    let prefix = super::clean_prefix(url.path());

    let client = S3Client::new(Region::default());

    let mut continuation_token = None;

    loop {
        let res = client
            .list_objects_v2(ListObjectsV2Request {
                bucket: bucket.clone(),
                continuation_token: continuation_token.clone(),
                prefix: prefix.clone(),
                delimiter: None,
                encoding_type: None,
                fetch_owner: None,
                max_keys: Some(1000),
                request_payer: None,
                start_after: None,
            })
            .await?;

        if let Some(contents) = res.contents {
            for object in contents {
                println!(
                    "{} {}",
                    object.key.as_ref().unwrap(),
                    object.last_modified.as_ref().unwrap()
                )
            }
        }

        continuation_token = res.next_continuation_token;
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(())
}

async fn s3_ls_buckets() -> Result<(), EcrDockerLoginError> {
    let client = S3Client::new(Region::default());

    let res = client.list_buckets().await?;

    if let Some(buckets) = res.buckets {
        if buckets.len() == 0 {
            println!("no buckets");
        }
        for bucket in buckets {
            let name = bucket.name.unwrap_or("N/A".into());
            let creation_date = bucket.creation_date.unwrap_or("N/A".into());

            println!("{}  {}", name, creation_date)
        }
    } else {
        println!("no buckets")
    }

    Ok(())
}

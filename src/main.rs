use aws_ci_buddy::EcrDockerLoginError;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), EcrDockerLoginError> {
    let mut app = App::new("aws-ci-buddy")
        .version("0.1")
        .author("Xavier Lange <xrlange@gmail.com>")
        .about("Make it easy to common CI-related tasks without a big dependency")
        .subcommand(
            SubCommand::with_name("ecr")
                .arg(Arg::with_name("get-login").help("Get an ecr-login string")),
        )
        .subcommand(
            SubCommand::with_name("s3")
                .subcommand(
                    SubCommand::with_name("cp")
                        .arg(Arg::with_name("source").takes_value(true))
                        .arg(Arg::with_name("target").takes_value(true)),
                )
                .subcommand(
                    SubCommand::with_name("ls").arg(Arg::with_name("path").takes_value(true)),
                ),
        );
    let matches = app.clone().get_matches();

    if let Some(ecr_cmd) = matches.subcommand_matches("ecr") {
        if ecr_cmd.value_of("get-login").is_some() {
            aws_ci_buddy::ecr_login().await?;
        } else {
            app.print_help().unwrap();
        }
    } else if let Some(s3_cmd) = matches.subcommand_matches("s3") {
        if let Some(cp_cmd) = s3_cmd.subcommand_matches("cp") {
            match (cp_cmd.value_of("source"), cp_cmd.value_of("target")) {
                (Some(src), Some(tgt)) => {
                    aws_ci_buddy::s3_cp(src, tgt).await?;
                }
                _ => {
                    app.print_help().unwrap();
                }
            }
        } else if let Some(ls_cmd) = s3_cmd.subcommand_matches("ls") {
            aws_ci_buddy::s3_ls(ls_cmd.value_of("path")).await?;
        } else {
            app.print_help().unwrap();
        }
    } else {
        app.print_help().unwrap();
    }

    // if matches.

    // panic!("matches: {:#?}", matches);

    // ecr_login().await?;

    Ok(())
}

use crate::graphdata::request::*;
use clap::Parser;
use custom_logger::*;
use semver::Version;
use std::fs;
use std::process;

mod api;
mod buildah;
mod graphdata;
mod isc;
mod upgradepath;

use api::schema::*;
use buildah::build_image::*;
use isc::generate::*;
use upgradepath::calculate::*;

// main entry point (use async)
#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let args_check: Vec<_> = std::env::args().collect();
    // 4 args from-version, to-version, channel, arch
    if args_check.len() < 4 {
        eprintln!("Usage: rust-release-introspection-tool --help");
        std::process::exit(1);
    }

    let from_version = args.from_version.to_string();
    let to_version = args.to_version.to_string();

    if !Version::parse(&from_version).is_ok() || !Version::parse(&to_version).is_ok() {
        eprintln!("ensure from-version and to-version are valid semver versions");
        std::process::exit(1);
    }

    let arch = args.arch.to_string();
    let channel = args.channel.to_string();
    let level = args.loglevel.unwrap().to_string();
    let force_update = args.force_update;
    let graph = args.graph;

    // convert to enum
    let res_log_level = match level.as_str() {
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    // setup logging
    let log = &Logging {
        log_level: res_log_level,
    };

    log.info(&format!("from_version: {}", from_version));
    log.info(&format!("to_version: {}", to_version));
    log.info(&format!("arch: {}", arch.clone()));

    let file_name = format!("cache/{}_{}.json", channel, arch);
    let mut json_data: String = String::new();

    // first check if we have this json on disk
    if force_update {
        log.info("force-update detected executing https api request");
        let url = format!(
            "https://api.openshift.com/api/upgrades_info/v1/graph?arch={}&channel={}&version={}",
            arch, channel, from_version
        );
        // setup the request interface
        let g_con = ImplUpgradePathInterface {};
        let res = g_con.get_graphdata(url.clone()).await;
        if res.is_ok() {
            json_data = res.unwrap();
            // we can now save the json to file
            fs::write(file_name.clone(), json_data.clone())
                .expect("unable to write file json payload");
        }
    } else {
        log.info("reading from cache");
        let res = fs::read_to_string(file_name.clone());
        if res.is_ok() {
            json_data = res.unwrap();
        } else {
            log.error(&format!(
                "file not found {} (use the --force-update flag to download it)",
                file_name.clone()
            ));
            process::exit(1);
        }
    }

    if graph {
        build_image().await;
    }

    // parse and calculate the upgrade path
    let g = Graph::new();
    let graphdata = g.parse_json_graphdata(json_data.clone()).unwrap();
    let images = g.get_upgrade_path(log, from_version, to_version, graphdata);
    let v2_yml = IscV2Alpha1::new().to_yaml(channel, images.clone());
    log.debug(&format!("{}", v2_yml.clone()));
    let v3_yml = IscV3Alpha1::new().to_yaml(images.clone());
    log.debug(&format!("{}", v3_yml.clone()));
}

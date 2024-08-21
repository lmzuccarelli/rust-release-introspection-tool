// rust-release-upgrade-path-tool cli struct
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "release-upgrade-path-tool")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "Used to find the upgrade path for eus releases (openshift specific)", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// semver format of from version
    #[arg(short, long, value_name = "from-version", default_value = "")]
    pub from_version: String,

    /// semver format of to version
    #[arg(short, long, value_name = "to-version", default_value = "")]
    pub to_version: String,

    /// architecture to use (amd64, arm64)
    #[arg(short, long, value_name = "arch", default_value = "amd64")]
    pub arch: String,

    /// channel (eus or stable) with version
    #[arg(long, value_name = "channel", default_value = "eus-4.14")]
    pub channel: String,

    /// graph - generate a graph container
    #[arg(short, long, value_name = "graph", default_value = "false")]
    pub graph: bool,

    /// set the loglevel. Valid arguments are info, debug, trace
    #[arg(value_enum, long, value_name = "loglevel", default_value = "info")]
    pub loglevel: Option<String>,

    /// force-check if set will call the api to get the latest json payload
    #[arg(short, long, value_name = "force-update", default_value = "false")]
    pub force_update: bool,
}

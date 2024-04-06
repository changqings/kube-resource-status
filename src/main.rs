use kube::Client;
use std::str::FromStr;

use anyhow::Result;
use bpaf::Bpaf;
use tabled::{locator::ByColumnName, Disable, Style, Table};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// a tool that provide kubernetes cluster resource information, including cpu, memory, storage and number of pods.
struct Options {
    #[bpaf(short('u'), long)]
    /// show the real utilization
    utilization: bool,
    #[bpaf(short('l'), long)]
    /// filter spesific node using it's label
    selector: Option<String>,
    #[bpaf(short('t'), long("type"))]
    /// filter based on resource type (eg: node, namespace), default: node
    resource_type: Option<String>,
    #[bpaf(short('s'), long)]
    /// filter by cpu, mem, storage or pods
    sort_by: Option<String>,
}

mod kubernetes;
mod utils;

#[cfg(test)]
mod utils_test;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = options().run();
    let mut sort_by = utils::Filter::None;
    let mut resource_type = kubernetes::ResourceType::Node;

    if let Some(rt) = opts.resource_type {
        resource_type = kubernetes::ResourceType::from_str(&rt).unwrap();
    }

    if let Some(s) = opts.sort_by {
        sort_by = utils::Filter::from_str(&s).unwrap();
    }

    let mut resource_req = Vec::new();

    let client = Client::try_default().await?;

    kubernetes::collect_info(
        client.clone(),
        &mut resource_req,
        resource_type,
        opts.utilization,
        opts.selector,
    )
    .await;

    let data = utils::parse_resource_data(resource_req, sort_by);
    let mut table = Table::new(&data);

    table.with(Style::rounded());
    if !opts.utilization {
        table.with(Disable::column(ByColumnName::new("cpu usage")));
        table.with(Disable::column(ByColumnName::new("mem usage")));
    }

    println!("{}", table);
    Ok(())
}

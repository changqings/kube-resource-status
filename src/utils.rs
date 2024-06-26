use std::cmp::Reverse;
use std::str::FromStr;

use super::kubernetes;

pub enum Filter {
    Cpu,
    Mem,
    Storage,
    Pods,
    None,
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "" => Ok(Filter::None),
            "cpu" => Ok(Filter::Cpu),
            "mem" => Ok(Filter::Mem),
            "storage" => Ok(Filter::Storage),
            "pods" => Ok(Filter::Pods),
            _ => Err(format!("invalid filter {}", s)),
        }
    }
}

pub fn parse_resource_data(
    rrs: Vec<kubernetes::ResouceRequests>,
    sort_by: Filter,
) -> Vec<kubernetes::ResourceStatus> {
    let mut data: Vec<kubernetes::ResouceRequests> = rrs.clone();
    let mut result = Vec::new();

    match sort_by {
        Filter::Cpu => data.sort_by_key(|r| Reverse(r.cpu_requests)),
        Filter::Mem => data.sort_by(|a, b| b.mem_requests.partial_cmp(&a.mem_requests).unwrap()),
        Filter::Storage => {
            data.sort_by(|a, b| b.storage_requests.partial_cmp(&a.storage_requests).unwrap())
        }
        Filter::Pods => data.sort_by_key(|r| Reverse(r.pods)),
        _ => (),
    }

    for rr in data {
        let cpu_req_percentage = (rr.cpu_requests as f32 / rr.cpu_total as f32) * 100.0;
        let mem_req_percentage = (rr.mem_requests / rr.mem_total) * 100.0;
        let cpu_usage_percentage = (rr.cpu_usage as f32 / rr.cpu_total as f32) * 100.0;
        let mem_usage_percentage = (rr.mem_usage / rr.mem_total) * 100.0;
        let storage_req_percentage = (rr.storage_requests / rr.storage_total) * 100.0;

        let rs = kubernetes::ResourceStatus::new(
            format!("{}", rr.name),
            format!("{}m ({:.2}%)", rr.cpu_requests, cpu_req_percentage),
            format!("{}m ({:.2}%)", rr.cpu_usage, cpu_usage_percentage),
            format!("{}Mi ({:.2}%)", rr.mem_requests, mem_req_percentage),
            format!("{:.2}Mi ({:.2}%)", rr.mem_usage, mem_usage_percentage),
            format!("{}Mi ({:.2}%)", rr.storage_requests, storage_req_percentage),
            format!("{} / {}", rr.pods, rr.pods_total),
        );
        result.push(rs);
    }

    return result;
}

pub async fn add_data(
    resource_name: String, //resource_name, pod or namespace
    cpu_requests: u32,
    cpu_total: u32,
    cpu_usage: u32,
    mem_requests: f32,
    mem_total: f32,
    mem_usage: f32,
    storage_requests: f32,
    storage_total: f32,
    pods: usize,
    pods_total: usize,
    rrs: &mut Vec<kubernetes::ResouceRequests>,
) {
    rrs.push(kubernetes::ResouceRequests::new(
        resource_name,
        cpu_requests,
        cpu_total,
        cpu_usage,
        mem_requests,
        mem_total,
        mem_usage,
        storage_requests,
        storage_total,
        pods,
        pods_total,
    ));
}

pub fn parse_cpu_requests(cpu: String) -> u32 {
    if cpu.contains(".") {
        let n = cpu.replace(".", "");
        return n.parse::<u32>().unwrap() * 100;
    } else if let Some((n, _unit)) = cpu.split_once("m") {
        return n.parse::<u32>().unwrap();
    } else if let Some((n, _unit)) = cpu.split_once("n") {
        return (n.parse::<f32>().unwrap() / 1000000.0) as u32;
    } else {
        return cpu.parse::<u32>().unwrap() * 1000;
    }
}

pub fn parse_capacity_requests(mem: String) -> f32 {
    if let Some((n, _unit)) = mem.split_once("Ki") {
        return n.parse::<f32>().unwrap() / 1024.0;
    } else if let Some((n, _unit)) = mem.split_once("Mi") {
        return n.parse::<f32>().unwrap();
    } else if let Some((n, _unit)) = mem.split_once("Gi") {
        return n.parse::<f32>().unwrap() * 1024.0;
    } else if let Some((n, _unit)) = mem.split_once("Ti") {
        return n.parse::<f32>().unwrap() * 1024.0 * 1024.0;
    } else if let Some((n, _unit)) = mem.split_once("m") {
        return n.parse::<f32>().unwrap() / 1024.0 / 1024.0 / 1024.0;
    } else if let Some((n, _unit)) = mem.split_once("k") {
        return n.parse::<f32>().unwrap() / 1000.0 * 0.953674;
    } else if let Some((n, _unit)) = mem.split_once("M") {
        return n.parse::<f32>().unwrap() * 0.953674;
    } else if let Some((n, _unit)) = mem.split_once("G") {
        return n.parse::<f32>().unwrap() * 0.953674 * 1000.0;
    } else if let Some((n, _unit)) = mem.split_once("T") {
        return n.parse::<f32>().unwrap() * 0.953674 * 1000.0 * 1000.0;
    } else {
        return mem.parse::<f32>().unwrap() / 1024.0 / 1024.0;
    }
}

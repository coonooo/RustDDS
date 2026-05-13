use std::{
  io,
  net::{IpAddr, SocketAddr},
};
use log::error;
// 移除 pnet 相关引用

use crate::structure::locator::Locator;

pub fn get_local_multicast_locators(port: u16) -> Vec<Locator> {
  let saddr = SocketAddr::new("239.255.0.1".parse().unwrap(), port);
  vec![Locator::from(saddr)]
}

pub fn get_local_unicast_locators(port: u16) -> Vec<Locator> {
  match if_addrs::get_if_addrs() {
    Ok(ifaces) => ifaces
      .iter()
      .filter(|iface| !iface.is_loopback())
      .map(|iface| Locator::from(SocketAddr::new(iface.ip(), port)))
      .collect(),
    Err(e) => {
      error!("Cannot get local network interfaces: get_if_addrs() : {e:?}");
      vec![]
    }
  }
}

/// 枚举可用于多播的本地接口 IP。
pub fn get_local_multicast_ip_addrs() -> io::Result<Vec<IpAddr>> {
  // if_addrs::get_if_addrs() 获取系统接口列表
  let interfaces = if_addrs::get_if_addrs()?;
  
  Ok(get_local_multicast_ip_addrs_inner(interfaces))
}

/// 注意：if_addrs::Interface 类型本身包含了 IP。
fn get_local_multicast_ip_addrs_inner(interfaces: Vec<if_addrs::Interface>) -> Vec<IpAddr> {
  interfaces
    .into_iter()
    .filter(|iface| !iface.is_loopback()) // 过滤回环
    .map(|iface| iface.ip())
    .filter(|ip| ip.is_ipv4()) // 目前仅支持 IPv4
    .collect()
}
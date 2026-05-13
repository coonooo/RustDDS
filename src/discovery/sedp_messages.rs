use std::{
  io,
  net::{IpAddr, SocketAddr},
};

use log::{error, warn};

use crate::structure::locator::Locator;

pub fn get_local_multicast_locators(port: u16) -> Vec<Locator> {
  let saddr = SocketAddr::new("239.255.0.1".parse().unwrap(), port);
  vec![Locator::from(saddr)]
}

pub fn get_local_unicast_locators_filtered(
  port: u16,
  only_networks: Option<&[IpAddr]>,
) -> Vec<Locator> {
  match if_addrs::get_if_addrs() {
    Ok(ifaces) => {
      let result = get_local_unicast_locators_inner(&ifaces, port, only_networks);
      if result.is_empty() {
        if let Some(nets) = only_networks {
          warn!(
            "only_networks filter {:?} matched no unicast interfaces; this participant will be \
             invisible to peers.",
            nets,
          );
        }
      }
      result
    }
    Err(e) => {
      error!("Cannot get local network interfaces: get_if_addrs() : {e:?}");
      vec![]
    }
  }
}

fn get_local_unicast_locators_inner(
  ifaces: &[if_addrs::Interface],
  port: u16,
  only_networks: Option<&[IpAddr]>,
) -> Vec<Locator> {
  ifaces
    .iter()
    .filter(|iface| !iface.is_loopback()) // 通常排除回环地址
    .filter(|iface| only_networks.is_none_or(|nets| nets.contains(&iface.ip())))
    .map(|iface| Locator::from(SocketAddr::new(iface.ip(), port)))
    .collect()
}

/// 枚举可用于多播的本地接口 IP。
/// 移除 pnet 后，我们主要依赖 if_addrs 来获取接口信息。
pub fn get_local_multicast_ip_addrs_filtered(
  only_networks: Option<&[IpAddr]>,
) -> io::Result<Vec<IpAddr>> {
  let interfaces = if_addrs::get_if_addrs()?;
  Ok(get_local_multicast_ip_addrs_inner(
    interfaces,
    only_networks,
  ))
}

/// 内部实现，使用 if_addrs::Interface 代替 pnet 类型。
fn get_local_multicast_ip_addrs_inner(
  interfaces: Vec<if_addrs::Interface>,
  only_networks: Option<&[IpAddr]>,
) -> Vec<IpAddr> {
  interfaces
    .into_iter()
    // 排除回环接口，因为通常不在回环上进行多播发现
    .filter(|iface| !iface.is_loopback())
    .map(|iface| iface.ip())
    // 过滤 IPv4（保持原逻辑，RustDDS 目前似乎偏好 IPv4）
    .filter(|ip| ip.is_ipv4())
    // 如果有指定的网络过滤列表
    .filter(|ip| only_networks.is_none_or(|nets| nets.contains(ip)))
    .collect()
}

#[cfg(test)]
mod tests {
  use std::net::{IpAddr, Ipv4Addr, SocketAddr};
  use crate::structure::locator::Locator;
  use if_addrs::{Interface, IfAddr, Ifv4Addr, IfOperStatus};

  // 辅助函数：快速创建 if_addrs 接口实例
  fn mock_ipv4_iface(name: &str, ip: Ipv4Addr, is_loopback: bool) -> Interface {
    Interface {
      name: name.to_string(),
      addr: IfAddr::V4(Ifv4Addr {
        ip,
        netmask: Ipv4Addr::new(255, 255, 255, 0),
        broadcast: None,
        prefixlen: 24,
      }),
      index: None,
      oper_status: IfOperStatus::Up,
    }
  }

  #[test]
  fn test_get_local_multicast_ip_addrs() {
    let eth0 = mock_ipv4_iface("eth0", Ipv4Addr::new(192, 168, 0, 137), false);
    let lo = mock_ipv4_iface("lo", Ipv4Addr::new(127, 0, 0, 1), true);

    let interfaces = vec![lo, eth0];
    let ips = super::get_local_multicast_ip_addrs_inner(interfaces, None);

    assert_eq!(ips.len(), 1);
    assert!(ips.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 0, 137))));
  }

  #[test]
  fn multicast_filter_respects_only_networks() {
    let interfaces = vec![
      mock_ipv4_iface("eth0", Ipv4Addr::new(192, 168, 0, 10), false),
      mock_ipv4_iface("eth1", Ipv4Addr::new(10, 0, 0, 10), false),
    ];

    let only_networks = [IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10))];
    let ips = super::get_local_multicast_ip_addrs_inner(interfaces, Some(&only_networks));

    assert_eq!(ips, vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10))]);
  }
}
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
    .filter(|ip| only_networks.is_none_or(|nets| nets.contains(&ip.ip())))
    .map(|ip| Locator::from(SocketAddr::new(ip.ip(), port)))
    .collect()
}

/// Enumerates local interfaces that we may use for multicasting.
///
/// 【已替换：不再使用 pnet，纯 if_addrs 实现】
pub fn get_local_multicast_ip_addrs_filtered(
  only_networks: Option<&[IpAddr]>,
) -> io::Result<Vec<IpAddr>> {
    let ifaces = if_addrs::get_if_addrs()?;
    
    let ips = ifaces
        .into_iter()
        .filter(|iface| {
            // 过滤：非回环 + 支持组播（标准UDP组播都支持）
            !iface.is_loopback() 
        })
        .filter(|iface| {
            // only_networks 过滤
            only_networks.is_none_or(|nets| {
                nets.contains(&iface.ip())
            })
        })
        .map(|iface| iface.ip())
        .filter(|ip| ip.is_ipv4())
        .collect();

    Ok(ips)
}


#[cfg(test)]
mod tests {
  use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
  };

  use crate::structure::locator::Locator;

  #[test]
  fn unicast_filter_respects_only_networks() {
    let only_networks = [IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10))];
    let ifaces = vec![
      if_addrs::Interface {
        name: "eth0".to_string(),
        addr: if_addrs::IfAddr::V4(if_addrs::Ifv4Addr {
          ip: Ipv4Addr::new(192, 168, 0, 10),
          netmask: Ipv4Addr::new(255, 255, 255, 0),
          prefixlen: 24,
          broadcast: None,
        }),
        index: None,
        oper_status: if_addrs::IfOperStatus::Up,
      },
      if_addrs::Interface {
        name: "eth1".to_string(),
        addr: if_addrs::IfAddr::V4(if_addrs::Ifv4Addr {
          ip: Ipv4Addr::new(10, 0, 0, 10),
          netmask: Ipv4Addr::new(255, 255, 255, 0),
          prefixlen: 24,
          broadcast: None,
        }),
        index: None,
        oper_status: if_addrs::IfOperStatus::Up,
      },
    ];

    let filtered = super::get_local_unicast_locators_inner(&ifaces, 7412, Some(&only_networks));

    assert_eq!(
      filtered,
      vec![Locator::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10)),
        7412,
      ))]
    );
  }
}
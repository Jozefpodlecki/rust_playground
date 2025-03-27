use std::{ffi::OsString, net::IpAddr, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use ipnetwork::IpNetwork;
use sysinfo::{Pid, System};
use netstat::*;
use tokio::runtime::Handle;
use anyhow::*;

use crate::{aws_iprange::{AwsIpRange, IpPrefix}, models::Message};

pub struct ProcessWatcher {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>,
}

impl ProcessWatcher {
    pub fn new() -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, tx: Sender<Message>) {

        let process_name = OsString::from("LOSTARK.exe");
        let port = 6040;
        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || Self::check_periodically(
            process_name,
            port,
            close_flag, tx));

        self.handle = Some(handle);
    }

    // pub fn check_periodically_old(
    //     process_name: OsString,
    //     port: u16,
    //     close_flag: Arc<AtomicBool>, tx: Sender<Message>) -> Result<()> {
    //     let mut system = System::new_all();
    //     let check_timeout = Duration::from_secs(15);
    //     let mut process_ids = vec![];
    //     let aws_ip_range = AwsIpRange::new();
    //     let rt = Handle::current();
    //     let mut last_message = Message::Unknown;

    //     loop {
            
    //         if close_flag.load(Ordering::Relaxed) {
    //             break;
    //         }

    //         system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&process_ids), true);
        
    //         let processes: Vec<_> = system.processes_by_name(&process_name).collect();
    //         let process = processes.first();

    //         if let Some(process) = process {

    //             last_message = Message::ProcessRunning;
    //             tx.send(last_message.clone())?;

    //             let process_id = process.pid();

    //             if !process_ids.is_empty() {
    //                 process_ids.remove(0);
    //             }

    //             process_ids.push(process_id);
    //             let process_id = process_id.as_u32();

    //             let address_family_flags = AddressFamilyFlags::IPV4;
    //             let proto = ProtocolFlags::TCP;
    //             let ip_addrs = get_sockets_info(address_family_flags, proto)
    //                 .ok()
    //                 .and_then(|socket| socket.into_iter().find(|socket| socket.associated_pids.contains(&process_id)))
    //                 .into_iter()
    //                 .filter_map(|info| {
    //                     if let ProtocolSocketInfo::Tcp(tcp) = info.protocol_socket_info {
    //                         (tcp.remote_port == port).then(|| tcp.remote_addr)
    //                     }
    //                     else {
    //                         None
    //                     }
    //                 });

    //                 for ip_addr in ip_addrs {
                        
    //                     let aws_ip_ranges = rt.block_on(async {
    //                         aws_ip_range.get().await
    //                     })?;
                        
    //                     for prefix in aws_ip_ranges.prefixes {
    //                         let ip: IpAddr = ip_addr.to_string().parse()?;
    //                         let network: IpNetwork = prefix.ip_prefix.parse()?;

    //                         if network.contains(ip) {
    //                             last_message = Message::ProcessListening(prefix.region);
    //                             tx.send(last_message.clone())?;
    //                         }
    //                     }
                        
    //                 }

    //         }
    //         else {

    //             match &last_message {
    //                 Message::Unknown => {
    //                     last_message = Message::ProcessNotRunning;
    //                     tx.send(last_message.clone())?;
    //                 },
    //                 Message::ProcessNotRunning => {},
    //                 Message::ProcessRunning => {
    //                     last_message = Message::ProcesStopped;
    //                     tx.send(last_message.clone())?;
    //                 },
    //                 Message::ProcessListening(_) => {
    //                     last_message = Message::ProcessNotRunning;
    //                     tx.send(last_message.clone())?;
    //                 },
    //                 Message::ProcesStopped => {},
    //             };

    //             sleep(check_timeout);
    //             continue;
    //         }
    //     }

    //     Ok(())
    // }


    fn check_periodically(
        process_name: OsString,
        port: u16,
        close_flag: Arc<AtomicBool>,
        tx: Sender<Message>,
    ) -> Result<()> {
        let mut system = System::new_all();
        let check_timeout = Duration::from_secs(15);
        let aws_ip_range = AwsIpRange::new();
        let rt = Handle::current();
        let mut last_message = Message::Unknown;
        let mut process_ids = Vec::<Pid>::new();

        while !close_flag.load(Ordering::Relaxed) {
            system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&process_ids), true);
            let processes: Vec<_> = system.processes_by_name(&process_name).collect();

            if let Some(process) = processes.first() {
                let process_id = process.pid();
                process_ids.retain(|&pid| pid != process_id);
                process_ids.push(process_id);

                Self::send_message(&tx, &mut last_message, Message::ProcessRunning)?;

                let ip_addrs = Self::find_process_ips(process_id.as_u32(), port)?;
                for ip_addr in ip_addrs {
                    let aws_ip_ranges = rt.block_on(async { aws_ip_range.get().await })?;
                    if let Some(region) = Self::match_aws_ip(&aws_ip_ranges.prefixes, ip_addr)? {
                        Self::send_message(&tx, &mut last_message, Message::ProcessListening(region))?;
                    }
                }
            } else {
                Self::handle_process_stopped(&tx, &mut last_message)?;
                sleep(check_timeout);
            }
        }

        Ok(())
    }

    fn find_process_ips(process_id: u32, port: u16) -> Result<Vec<IpAddr>> {
        let address_family_flags = AddressFamilyFlags::IPV4;
        let proto = ProtocolFlags::TCP;

        let sockets = get_sockets_info(address_family_flags, proto)
            .ok()
            .unwrap_or_default();

        let ip_addrs = sockets.into_iter()
            .find(|socket| socket.associated_pids.contains(&process_id))
            .into_iter()
            .filter_map(|info| {
                if let ProtocolSocketInfo::Tcp(tcp) = info.protocol_socket_info {
                    (tcp.remote_port == port).then(|| tcp.remote_addr)
                } else {
                    None
                }
            })
            .collect();

        Ok(ip_addrs)
    }

    fn match_aws_ip(prefixes: &[IpPrefix], ip_addr: IpAddr) -> Result<Option<String>> {
        for prefix in prefixes {
            let network: IpNetwork = prefix.ip_prefix.parse()?;
            if network.contains(ip_addr) {
                return Ok(Some(prefix.region.clone()));
            }
        }
        Ok(None)
    }

    fn send_message(tx: &Sender<Message>, last_message: &mut Message, new_message: Message) -> Result<()> {
        if *last_message != new_message {
            tx.send(new_message.clone())?;
            *last_message = new_message;
        }
        Ok(())
    }

    fn handle_process_stopped(tx: &Sender<Message>, last_message: &mut Message) -> Result<()> {
        let new_message = match last_message {
            Message::Unknown => Message::ProcessNotRunning,
            Message::ProcessNotRunning => return Ok(()),
            Message::ProcessRunning => Message::ProcesStopped,
            Message::ProcessListening(_) => Message::ProcessNotRunning,
            Message::ProcesStopped => return Ok(()),
        };

        Self::send_message(tx, last_message, new_message)
    }


    pub fn stop(&mut self) -> Result<()> {
        self.close_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|err| anyhow::anyhow!("{:?}", err))??;
        }

        Ok(())
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
/*
 *
 * Copyright (c) 2023 Universidade de Vigo
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation;
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Author: Miguel Rodríguez Pérez <miguel@det.uvigo.gal>
 *
 */

use std::{
    io::{Read, Write},
    net::{Ipv6Addr, SocketAddr, TcpListener},
    thread,
};

use clap::Parser;
use socket2::{Domain, Socket, Type};
use tcpmt::{Answer, AnswerOrder, Operation, TlvIterator};

#[derive(Debug, Parser)]
struct Args {
    /// Port number
    #[arg(value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// Answer order
    #[arg(short, long)]
    message_last: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // We need to use the socket2 create to properly support Windows
    let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;
    socket.set_only_v6(false)?;
    socket.set_reuse_address(true)?;
    socket.bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, args.port)).into())?;
    socket.listen(128)?;
    let listener: TcpListener = socket.into();

    let order = match args.message_last {
        true => AnswerOrder::MessageLast,
        false => AnswerOrder::MessageFirst,
    };

    loop {
        let (mut stream, addr) = listener.accept()?;
        thread::spawn(move || {
            let mut acc = 0i64;
            let mut buffer = [0u8; 2048];
            loop {
                match stream.read(&mut buffer) {
                    Ok(len) if len > 0 => {
                        for tlv in TlvIterator::process(&buffer[..len]) {
                            let res = tlv
                                .try_into()
                                .and_then(|op: Operation| (op.reduce().map(|res| (op, res))));
                            let answer = match res {
                                Ok((operation, result)) => {
                                    acc = acc.saturating_add(result);

                                    println!("{addr}: {operation} = {result}");
                                    (acc, None)
                                }
                                Err(ref e) => {
                                    eprintln!("{addr}: Could not calculate answer. {}", e.clone());
                                    (acc, Some(e.to_string()))
                                }
                            };

                            if stream
                                .write_all(&Answer::from(answer).encode(order))
                                .is_err()
                            {
                                // Problably the connection to the client has been lost
                                return;
                            }
                        }
                    }
                    _ => return, // Probably the client has closed the connection
                }
            }
        });
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
/*
 *
 * Copyright (c) 2023–2025 Universidade de Vigo
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

use operation::OperationError;
use std::array::TryFromSliceError;
use std::num::{ParseIntError, TryFromIntError};
use std::str::Utf8Error;
use tlv::TlvError;

use thiserror::Error;

mod answer;
mod operation;
mod tlv;

pub use answer::{Answer, AnswerOrder};
pub use operation::Operation;
pub use tlv::Tlv;
pub use tlv::TlvIterator;

#[derive(Error, Debug)]
pub enum TCPLibError {
    #[error("Operation error")]
    OperationError(#[from] OperationError),
    #[error("Unsupported operation {0}")]
    UnsupportedOperation(String),
    #[error("Could not parse operation")]
    Parse,
    #[error("Not enough data in TLV")]
    NotEnoughData(#[from] TryFromSliceError),
    #[error("Invalid parameter")]
    InvalidParameter(#[from] TryFromIntError),
    #[error("Could not parse integer")]
    ParseIntError(#[from] ParseIntError),
    #[error("Wrong encoding for error message")]
    ParseStringError(#[from] Utf8Error),
    #[error("Could not parse TLV")]
    ParseTlvError(#[from] TlvError),
    #[error("Something wrong")]
    Generic,
}

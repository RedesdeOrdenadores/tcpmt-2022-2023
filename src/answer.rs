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

use crate::{tlv::TlvType, TCPLibError, Tlv, TlvIterator};
use std::{fmt::Display, str};

#[derive(Debug)]
pub struct Answer {
    pub acc: Numberi64,
    pub message: Option<InvalidOperation>,
}

impl Answer {
    pub fn encode(self) -> Box<[u8]> {
        let mut data = self.message.map_or(vec![], |v| v.encode().to_vec());
        data.extend_from_slice(&self.acc.encode());

        Tlv::new(TlvType::Answer, &data).unwrap().encode()
    }
}

impl<'a> TryFrom<Tlv<'a>> for Answer {
    type Error = TCPLibError;

    fn try_from(tlv: Tlv<'a>) -> Result<Self, Self::Error> {
        if tlv.tag == TlvType::Answer && tlv.length > 0 {
            let mut message: Option<InvalidOperation> = None;
            let mut acc_tlv: Option<Numberi64> = None;
            for ref tlv in TlvIterator::process(tlv.data) {
                match tlv.tag {
                    TlvType::Numi64 => acc_tlv = Some(tlv.try_into()?),
                    TlvType::Invalid => message = Some(tlv.try_into()?),
                    _ => {} // Just ignore extra TLVs
                };
            }

            if let Some(acc) = acc_tlv {
                return Ok(Answer { acc, message });
            }
        }
        Err(TCPLibError::Generic)
    }
}

impl From<(i64, Option<String>)> for Answer {
    fn from((acc, message): (i64, Option<String>)) -> Self {
        Answer {
            acc: acc.into(),
            message: message.map(|m| m.as_str().into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Numberi64(pub i64);

impl<'a> TryFrom<&Tlv<'a>> for Numberi64 {
    type Error = TCPLibError;

    fn try_from(tlv: &Tlv) -> Result<Self, Self::Error> {
        if tlv.tag == TlvType::Numi64 && tlv.length == 8 {
            Ok(Numberi64(i64::from_be_bytes(tlv.data.try_into()?)))
        } else {
            Err(TCPLibError::Generic)
        }
    }
}

impl<'a> TryFrom<Tlv<'a>> for Numberi64 {
    type Error = TCPLibError;

    fn try_from(tlv: Tlv) -> Result<Self, Self::Error> {
        (&tlv).try_into()
    }
}

impl Numberi64 {
    pub fn encode(self) -> Box<[u8]> {
        Tlv::new(TlvType::Numi64, &self.0.to_be_bytes())
            .unwrap()
            .encode()
    }
}

impl From<i64> for Numberi64 {
    fn from(num: i64) -> Self {
        Self(num)
    }
}

impl Display for Numberi64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidOperation(Box<str>);

impl<'a> TryFrom<&Tlv<'a>> for InvalidOperation {
    type Error = TCPLibError;

    fn try_from(tlv: &Tlv<'a>) -> Result<Self, Self::Error> {
        if tlv.tag == TlvType::Invalid && tlv.length > 0 {
            Ok(InvalidOperation(str::from_utf8(tlv.data)?.into()))
        } else {
            Err(TCPLibError::Generic)
        }
    }
}

impl InvalidOperation {
    pub fn encode(self) -> Box<[u8]> {
        Tlv::new(TlvType::Invalid, self.0.as_bytes())
            .unwrap()
            .encode()
    }
}

impl From<&str> for InvalidOperation {
    fn from(message: &str) -> Self {
        Self(message.into())
    }
}

impl Display for InvalidOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{answer::Numberi64, Tlv};

    #[test]
    fn parse_answer_1() {
        let tlv: Result<Tlv, _> = (&[16u8, 8, 0, 0, 0, 0, 0, 0, 0, 1][..]).try_into();
        assert!(tlv.is_ok());
        let answer: Result<Numberi64, _> = tlv.unwrap().try_into();
        assert!(answer.is_ok());
        assert_eq!(answer.unwrap(), 1.into());
    }

    #[test]
    fn parse_answer_minus1() {
        let tlv: Result<Tlv, _> =
            (&[16u8, 8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..]).try_into();
        assert!(tlv.is_ok());
        let answer: Result<Numberi64, _> = tlv.unwrap().try_into();
        assert!(answer.is_ok());
        assert_eq!(answer.unwrap(), (-1).into());
    }

    #[test]
    fn parse_answer_err_short() {
        let tlv: Result<Tlv, _> = (&[16u8, 7, 0, 0, 0, 0, 0, 0, 0, 1][..]).try_into();
        assert!(tlv.is_ok());
        let answer: Result<Numberi64, _> = tlv.unwrap().try_into();
        assert!(answer.is_err());
    }

    #[test]
    fn encode_answer() {
        assert_eq!(Numberi64(1).encode()[..], [16u8, 8, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(
            Numberi64(-1).encode()[..],
            [16u8, 8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }
}

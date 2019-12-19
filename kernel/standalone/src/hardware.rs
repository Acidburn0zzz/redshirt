// Copyright (C) 2019  Pierre Krieger
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Implements the `hardware` interface.
//!
//! The `hardware` interface is particular in that it can only be implemented using a "hosted"
//! implementation.

use crate::arch;

use alloc::vec::Vec;
use core::{convert::TryFrom as _, marker::PhantomData};
use parity_scale_codec::{DecodeAll, Encode as _};
use redshirt_hardware_interface::ffi::{HardwareAccessResponse, HardwareMessage, Operation};

/// State machine for `hardware` interface messages handling.
pub struct HardwareHandler<TMsgId> {
    marker: PhantomData<TMsgId>,
}

impl<TMsgId> HardwareHandler<TMsgId>
where
    TMsgId: Send + 'static,
{
    /// Initializes the new state machine for hardware accesses.
    pub fn new() -> Self {
        HardwareHandler {
            marker: PhantomData,
        }
    }

    /// Processes a message on the `hardware` interface, and optionally returns an answer to
    /// immediately send back.
    pub fn hardware_message(
        &self,
        message_id: Option<TMsgId>,
        message: &[u8],
    ) -> Option<Result<Vec<u8>, ()>> {
        match HardwareMessage::decode_all(&message) {
            Ok(HardwareMessage::HardwareAccess(operations)) => {
                let mut response = Vec::with_capacity(operations.len());
                for operation in operations {
                    unsafe {
                        if let Some(outcome) = perform_operation(operation) {
                            response.push(outcome);
                        }
                    }
                }

                if !response.is_empty() {
                    Some(Ok(response.encode()))
                } else {
                    None
                }
            }
            Ok(HardwareMessage::InterruptWait(int_id)) => unimplemented!(), // TODO:
            Err(_) => Some(Err(())),
        }
    }

    /*/// Returns the next message to answer, and the message to send back.
    pub fn next_answer(&self) -> impl Future<Output = (TMsgId, Vec<u8>)> {

    }*/
}

unsafe fn perform_operation(operation: Operation) -> Option<HardwareAccessResponse> {
    match operation {
        Operation::PhysicalMemoryWriteU8 { address, data } => {
            if let Ok(mut address) = usize::try_from(address) {
                for byte in data {
                    (address as *mut u8).write_volatile(byte);
                    address = address.checked_add(1).unwrap();
                }
            }
            None
        }
        Operation::PhysicalMemoryWriteU16 { address, data } => {
            if let Ok(mut address) = usize::try_from(address) {
                for word in data {
                    (address as *mut u16).write_volatile(word);
                    address = address.checked_add(2).unwrap();
                }
            }
            None
        }
        Operation::PhysicalMemoryWriteU32 { address, data } => {
            if let Ok(mut address) = usize::try_from(address) {
                for dword in data {
                    (address as *mut u32).write_volatile(dword);
                    address = address.checked_add(4).unwrap();
                }
            }
            None
        }
        Operation::PhysicalMemoryReadU8 { mut address, len } => {
            // TODO: try allocate `len` but don't panic if `len` is too large
            let mut out = Vec::with_capacity(len as usize); // TODO: don't use `as`
            for _ in 0..len {
                out.push((address as *mut u8).read_volatile());
                address = address.checked_add(1).unwrap();
            }
            Some(HardwareAccessResponse::PhysicalMemoryReadU8(out))
        }
        Operation::PhysicalMemoryReadU16 { mut address, len } => {
            // TODO: try allocate `len` but don't panic if `len` is too large
            let mut out = Vec::with_capacity(len as usize); // TODO: don't use `as`
            for _ in 0..len {
                out.push((address as *mut u16).read_volatile());
                address = address.checked_add(2).unwrap();
            }
            Some(HardwareAccessResponse::PhysicalMemoryReadU16(out))
        }
        Operation::PhysicalMemoryReadU32 { mut address, len } => {
            // TODO: try allocate `len` but don't panic if `len` is too large
            let mut out = Vec::with_capacity(len as usize); // TODO: don't use `as`
            for _ in 0..len {
                out.push((address as *mut u32).read_volatile());
                address = address.checked_add(4).unwrap();
            }
            Some(HardwareAccessResponse::PhysicalMemoryReadU32(out))
        }
        Operation::PortWriteU8 { port, data } => {
            arch::write_port_u8(port, data);
            None
        }
        Operation::PortWriteU16 { port, data } => {
            arch::write_port_u16(port, data);
            None
        }
        Operation::PortWriteU32 { port, data } => {
            arch::write_port_u32(port, data);
            None
        }
        Operation::PortReadU8 { port } => {
            Some(HardwareAccessResponse::PortReadU8(arch::read_port_u8(port)))
        }
        Operation::PortReadU16 { port } => Some(HardwareAccessResponse::PortReadU16(
            arch::read_port_u16(port),
        )),
        Operation::PortReadU32 { port } => Some(HardwareAccessResponse::PortReadU32(
            arch::read_port_u32(port),
        )),
    }
}
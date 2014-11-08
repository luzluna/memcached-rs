// The MIT License (MIT)

// Copyright (c) 2014 Y. T. CHUNG <zonyitoo@gmail.com>

// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Memcached protocol

use std::fmt::{Show, Formatter, mod};
use std::collections::TreeMap;
use std::io;

use version;

pub use self::binary::BinaryProto;

mod binarydef;
mod binary;

/// Memcached response status
#[deriving(Clone, Show, Eq, PartialEq)]
pub enum Status {
    NoError,
    KeyNotFound,
    KeyExists,
    ValueTooLarge,
    InvalidArguments,
    ItemNotStored,
    IncrDecrOnNonNumericValue,
    VBucketBelongsToOtherServer,
    AuthenticationError,
    AuthenticationContinue,
    UnknownCommand,
    OutOfMemory,
    NotSupported,
    InternalError,
    Busy,
    TemporaryFailure,
}

impl Status {
    /// Get the binary code of the status
    pub fn code(&self) -> u16 {
        match *self {
            NoError => binarydef::STATUS_NO_ERROR,
            KeyNotFound => binarydef::STATUS_KEY_NOT_FOUND,
            KeyExists => binarydef::STATUS_KEY_EXISTS,
            ValueTooLarge => binarydef::STATUS_VALUE_TOO_LARGE,
            InvalidArguments => binarydef::STATUS_INVALID_ARGUMENTS,
            ItemNotStored => binarydef::STATUS_ITEM_NOT_STORED,
            IncrDecrOnNonNumericValue => binarydef::STATUS_INCR_OR_DECR_ON_NON_NUMERIC_VALUE,
            VBucketBelongsToOtherServer => binarydef::STATUS_VBUCKET_BELONGS_TO_OTHER_SERVER,
            AuthenticationError => binarydef::STATUS_AUTHENTICATION_ERROR,
            AuthenticationContinue => binarydef::STATUS_AUTHENTICATION_CONTINUE,
            UnknownCommand => binarydef::STATUS_UNKNOWN_COMMAND,
            OutOfMemory => binarydef::STATUS_OUT_OF_MEMORY,
            NotSupported => binarydef::STATUS_NOT_SUPPORTED,
            InternalError => binarydef::STATUS_INTERNAL_ERROR,
            Busy => binarydef::STATUS_BUSY,
            TemporaryFailure => binarydef::STATUS_TEMPORARY_FAILURE,
        }
    }

    /// Generate a Status from binary code
    pub fn from_code(code: u16) -> Option<Status> {
        match code {
            binarydef::STATUS_NO_ERROR => Some(NoError),
            binarydef::STATUS_KEY_NOT_FOUND => Some(KeyNotFound),
            binarydef::STATUS_KEY_EXISTS => Some(KeyExists),
            binarydef::STATUS_VALUE_TOO_LARGE => Some(ValueTooLarge),
            binarydef::STATUS_INVALID_ARGUMENTS => Some(InvalidArguments),
            binarydef::STATUS_ITEM_NOT_STORED => Some(ItemNotStored),
            binarydef::STATUS_INCR_OR_DECR_ON_NON_NUMERIC_VALUE => Some(IncrDecrOnNonNumericValue),
            binarydef::STATUS_VBUCKET_BELONGS_TO_OTHER_SERVER => Some(VBucketBelongsToOtherServer),
            binarydef::STATUS_AUTHENTICATION_ERROR => Some(AuthenticationError),
            binarydef::STATUS_AUTHENTICATION_CONTINUE => Some(AuthenticationContinue),
            binarydef::STATUS_UNKNOWN_COMMAND => Some(UnknownCommand),
            binarydef::STATUS_OUT_OF_MEMORY => Some(OutOfMemory),
            binarydef::STATUS_NOT_SUPPORTED => Some(NotSupported),
            binarydef::STATUS_INTERNAL_ERROR => Some(InternalError),
            binarydef::STATUS_BUSY => Some(Busy),
            binarydef::STATUS_TEMPORARY_FAILURE => Some(TemporaryFailure),
            _ => None
        }
    }

    /// Get a short description
    pub fn desc(&self) -> &'static str {
        match *self {
            NoError => "no error",
            KeyNotFound => "key not found",
            KeyExists => "key exists",
            ValueTooLarge => "value too large",
            InvalidArguments => "invalid argument",
            ItemNotStored => "item not stored",
            IncrDecrOnNonNumericValue => "incr or decr on non-numeric value",
            VBucketBelongsToOtherServer => "vbucket belongs to other server",
            AuthenticationError => "authentication error",
            AuthenticationContinue => "authentication continue",
            UnknownCommand => "unknown command",
            OutOfMemory => "out of memory",
            NotSupported => "not supported",
            InternalError => "internal error",
            Busy => "busy",
            TemporaryFailure => "temporary failure",
        }
    }
}

/// Protocol type
pub enum ProtoType {
    Binary,
}

#[deriving(Clone)]
pub enum ErrorKind {
    MemCachedError(Status),
    IoError(io::IoErrorKind),
    OtherError,
}

pub type MemCachedResult<T> = Result<T, Error>;

#[deriving(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub desc: &'static str,
    pub detail: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, desc: &'static str, detail: Option<String>) -> Error {
        Error {
            kind: kind,
            desc: desc,
            detail: detail,
        }
    }
}

impl Show for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.detail {
            Some(ref detail) => write!(f, "{}", detail),
            None => write!(f, "{}", self.desc),
        }
    }
}

pub trait Proto: Operation + MultiOperation + ServerOperation + NoReplyOperation + CasOperation {
    fn clone(&self) -> Box<Proto + Send>;
}

pub trait Operation {
    fn set(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn add(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn delete(&mut self, key: &[u8]) -> MemCachedResult<()>;
    fn replace(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn get(&mut self, key: &[u8]) -> MemCachedResult<(Vec<u8>, u32)>;
    fn getk(&mut self, key: &[u8]) -> MemCachedResult<(Vec<u8>, Vec<u8>, u32)>;
    fn increment(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32) -> MemCachedResult<u64>;
    fn decrement(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32) -> MemCachedResult<u64>;
    fn append(&mut self, key: &[u8], value: &[u8]) -> MemCachedResult<()>;
    fn prepend(&mut self, key: &[u8], value: &[u8]) -> MemCachedResult<()>;
    fn touch(&mut self, key: &[u8], expiration: u32) -> MemCachedResult<()>;
}

pub trait CasOperation {
    fn set_cas(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32, cas: u64) -> MemCachedResult<u64>;
    fn add_cas(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<u64>;
    fn replace_cas(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32, cas: u64) -> MemCachedResult<u64>;
    fn get_cas(&mut self, key: &[u8]) -> MemCachedResult<(Vec<u8>, u32, u64)>;
    fn getk_cas(&mut self, key: &[u8]) -> MemCachedResult<(Vec<u8>, Vec<u8>, u32, u64)>;
    fn increment_cas(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32, cas: u64)
        -> MemCachedResult<(u64, u64)>;
    fn decrement_cas(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32, cas: u64)
        -> MemCachedResult<(u64, u64)>;
    fn append_cas(&mut self, key: &[u8], value: &[u8], cas: u64) -> MemCachedResult<u64>;
    fn prepend_cas(&mut self, key: &[u8], value: &[u8], cas: u64) -> MemCachedResult<u64>;
    fn touch_cas(&mut self, key: &[u8], expiration: u32, cas: u64) -> MemCachedResult<u64>;
}

pub trait ServerOperation {
    fn quit(&mut self) -> MemCachedResult<()>;
    fn flush(&mut self, expiration: u32) -> MemCachedResult<()>;
    fn noop(&mut self) -> MemCachedResult<()>;
    fn version(&mut self) -> MemCachedResult<version::Version>;
    fn stat(&mut self) -> MemCachedResult<TreeMap<String, String>>;
}

pub trait MultiOperation {
    fn set_multi(&mut self, kv: TreeMap<Vec<u8>, (Vec<u8>, u32, u32)>) -> MemCachedResult<()>;
    fn delete_multi(&mut self, keys: Vec<Vec<u8>>) -> MemCachedResult<()>;
    fn get_multi(&mut self, keys: Vec<Vec<u8>>) -> MemCachedResult<TreeMap<Vec<u8>, (Vec<u8>, u32)>>;
}

pub trait NoReplyOperation {
    fn set_noreply(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn add_noreply(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn delete_noreply(&mut self, key: &[u8]) -> MemCachedResult<()>;
    fn replace_noreply(&mut self, key: &[u8], value: &[u8], flags: u32, expiration: u32) -> MemCachedResult<()>;
    fn increment_noreply(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32) -> MemCachedResult<()>;
    fn decrement_noreply(&mut self, key: &[u8], amount: u64, initial: u64, expiration: u32) -> MemCachedResult<()>;
    fn append_noreply(&mut self, key: &[u8], value: &[u8]) -> MemCachedResult<()>;
    fn prepend_noreply(&mut self, key: &[u8], value: &[u8]) -> MemCachedResult<()>;
}

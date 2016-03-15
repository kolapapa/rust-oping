#![allow(dead_code)]

use std::mem::transmute;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::default::Default;

extern crate libc;
use libc::{AF_INET, AF_INET6};
use libc::c_int;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AddrFamily {
    IPV4,
    IPV6,
}

impl Default for AddrFamily {
    fn default() -> AddrFamily {
        AddrFamily::IPV4
    }
}

#[repr(C)]
enum PingOption {
    TIMEOUT = 1,
    TTL = 2,
    AF = 4,
    DATA = 8,
    SOURCE = 16,
    DEVICE = 32,
    QOS = 64,
}

#[repr(C)]
enum PingIterInfo {
    HOSTNAME = 1,
    ADDRESS = 2,
    FAMILY = 3,
    LATENCY = 4,
    SEQUENCE = 5,
    IDENT = 6,
    DATA = 7,
    USERNAME = 8,
    DROPPED = 9,
    RECV_TTL = 10,
    RECV_QOS = 11,
}

enum PingObj {}
enum PingObjIter {}

#[link(name =  "oping")]
extern {
    fn ping_construct() -> *mut PingObj;
    fn ping_destroy(obj: *mut PingObj);
    fn ping_setopt(obj: *mut PingObj, opt: PingOption, val: *mut u8) -> i32;
    fn ping_send(obj: *mut PingObj) -> i32;
    fn ping_host_add(obj: *mut PingObj, host: *const c_char) -> i32;
    fn ping_host_remove(obj: *mut PingObj, host: *const c_char) -> i32;
    fn ping_iterator_get(obj: *mut PingObj) -> *mut PingObjIter;
    fn ping_iterator_next(obj: *mut PingObjIter) -> *mut PingObjIter;
    fn ping_iterator_get_info(iter: *mut PingObjIter, info: PingIterInfo,
                              buf: *mut u8, size: *mut usize) -> i32;
    fn ping_get_error(obj: *mut PingObj) -> *const c_char;
}

#[derive(Debug)]
pub enum PingError {
    LibOpingError(String),
    NulByteError,
}

pub type PingResult<T> = Result<T, PingError>;

pub struct Ping {
    obj: *mut PingObj,
}

impl Drop for Ping {
    fn drop(&mut self) {
        unsafe { ping_destroy(self.obj) };
    }
}

macro_rules! try_c {
    ($obj: expr, $e: expr) => (
        if $e != 0 {
            let err = CStr::from_ptr(ping_get_error($obj));
            let s = String::from(err.to_str().unwrap());
            return Err(PingError::LibOpingError(s));
        }
    )
}


impl Ping {
    pub fn new() -> Ping {
        let obj = unsafe { ping_construct() };
        assert!(!obj.is_null());
        Ping {
            obj: obj,
        }
    }

    pub fn set_timeout(&mut self, timeout: f64) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                ping_setopt(self.obj, PingOption::TIMEOUT, transmute(&timeout)));
        }
        Ok(())
    }

    pub fn set_ttl(&mut self, ttl: i32) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                ping_setopt(self.obj, PingOption::TTL, transmute(&ttl)));
        }
        Ok(())
    }

    pub fn set_addr_family(&mut self, af: AddrFamily) -> PingResult<()> {
        let fam: c_int = match af {
            AddrFamily::IPV4 => AF_INET,
            AddrFamily::IPV6 => AF_INET6,
        };
        unsafe {
            try_c!(self.obj,
                ping_setopt(self.obj, PingOption::AF, transmute(&fam)));
        }
        Ok(())
    }

    pub fn set_qos(&mut self, qos: u8) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                ping_setopt(self.obj, PingOption::QOS, transmute(&qos)));
        }
        Ok(())
    }

    pub fn add_host(&mut self, hostname: &str) -> PingResult<()> {
        let cstr = match CString::new(hostname.as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(PingError::NulByteError),
        };
        unsafe {
            try_c!(self.obj,
                ping_host_add(self.obj, cstr.as_ptr()));
        }
        Ok(())
    }

    pub fn remove_host(&mut self, hostname: &str) -> PingResult<()> {
        let cstr = match CString::new(hostname.as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(PingError::NulByteError),
        };
        unsafe {
            try_c!(self.obj,
                ping_host_add(self.obj, cstr.as_ptr()));
        }
        Ok(())
    }

    // Returns number of replies received.
    pub fn send(&mut self) -> PingResult<i32> {
        unsafe {
            let result = ping_send(self.obj);
            if result < 0 {
                try_c!(self.obj, result);  // should return error.
                Ok(0)
            } else {
                Ok(result)
            }
        }
    }

    pub fn iter<'a>(&'a mut self) -> PingIter<'a> {
        let ptr = unsafe { ping_iterator_get(self.obj) };
        PingIter {
            iter: ptr,
            _lifetime: PhantomData,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PingIter<'a> {
    iter: *mut PingObjIter,
    _lifetime: PhantomData<&'a ()>,
}

#[derive(Clone, Debug, Default)]
pub struct PingItem {
    pub hostname: String,
    pub address: String,
    pub family: AddrFamily,
    pub latency_ms: f64,
    pub dropped: u32,
    pub seq: i32,
    pub recv_ttl: i32,
    pub recv_qos: u8,
}

macro_rules! get_str_field {
    ($iter: expr, $field: expr, $vec: expr) => (
        unsafe {
            let ptr = $vec.as_mut_ptr();
            let mut size: usize = $vec.capacity();
            if ping_iterator_get_info($iter, $field, ptr, &mut size as *mut usize) != 0 {
                return None;
            }
            CStr::from_ptr(ptr as *const i8).to_str().unwrap().to_string()
        }
    )
}

macro_rules! get_num_field {
    ($iter: expr, $field: expr, $vec: expr,$t: ty) => (
        unsafe {
            let ptr = $vec.as_mut_ptr();
            let mut size: usize = $vec.capacity();
            if ping_iterator_get_info($iter, $field, ptr, &mut size as *mut usize) != 0 {
                return None;
            }
            let cast_ptr: *const $t = transmute(ptr);
            *cast_ptr
        }
    )
}

impl<'a> Iterator for PingIter<'a> {
    type Item = PingItem;
    fn next(&mut self) -> Option<PingItem> {
        if self.iter == (0 as *mut PingObjIter) {
            return None;
        }

        let mut ret: PingItem = Default::default();
        let mut buf = Vec::<u8>::with_capacity(1024);

        ret.hostname = get_str_field!(self.iter, PingIterInfo::HOSTNAME, buf);
        ret.address = get_str_field!(self.iter, PingIterInfo::ADDRESS, buf);
        ret.family = match get_num_field!(self.iter, PingIterInfo::FAMILY, buf, i32) {
            libc::AF_INET => AddrFamily::IPV4,
            libc::AF_INET6 => AddrFamily::IPV6,
            _ => AddrFamily::IPV4
        };
        ret.latency_ms = get_num_field!(self.iter, PingIterInfo::LATENCY, buf, f64);
        ret.dropped = get_num_field!(self.iter, PingIterInfo::DROPPED, buf, u32);
        ret.seq = get_num_field!(self.iter, PingIterInfo::SEQUENCE, buf, i32);
        ret.recv_ttl = get_num_field!(self.iter, PingIterInfo::RECV_TTL, buf, i32);
        ret.recv_qos = get_num_field!(self.iter, PingIterInfo::RECV_QOS, buf, u8);

        self.iter = unsafe { ping_iterator_next(self.iter) };

        Some(ret)
    }
}

mod test {
    // N.B.: this test does not actually add any hosts or send pings, because
    // these actions usually require `root` privileges, and we want unit tests
    // to run as an ordinary user. As such we'll have to be content not to test
    // the host-add/remove, packet send/receive, or iterator functionality.

    #[test]
    fn test_basic_opts() {
        let mut p = ::Ping::new();
        assert!(p.set_timeout(5.0).is_ok());
        assert!(p.set_ttl(42).is_ok());
        assert!(p.set_addr_family(::AddrFamily::IPV4).is_ok());
        assert!(p.set_qos(42).is_ok());
    }
}

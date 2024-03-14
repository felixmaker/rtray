#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![doc = include_str!("../README.md")]
#![allow(non_upper_case_globals)]
#![allow(clippy::needless_doctest_main)]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::arc_with_non_send_sync)]

mod tray;

pub use tray::*;

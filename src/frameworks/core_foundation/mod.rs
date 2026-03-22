/*
 * Core Foundation base module.
 */

pub mod cf_allocator;
pub mod cf_data;
pub mod cf_dictionary;
pub mod cf_locale;
pub mod cf_number;
pub mod cf_string;
pub mod cf_type;
pub mod cf_run_loop_timer;

// Base types required by sub-modules via super::
pub type CFIndex = i32;
pub type CFOptionFlags = u32;
pub type CFHashCode = u32;
pub type CFTypeRef = crate::objc::id; 

pub const kCFNotFound: CFIndex = -1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CFRange {
    pub location: CFIndex,
    pub length: CFIndex,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum CFComparisonResult {
    LessThan = -1,
    EqualTo = 0,
    GreaterThan = 1,
}

// Memory management stubs
pub fn CFRetain(obj: CFTypeRef) -> CFTypeRef { obj }
pub fn CFRelease(_obj: CFTypeRef) { }

// Helper function for hashing
pub fn hash_helper(data: &str) -> CFHashCode {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish() as CFHashCode
}

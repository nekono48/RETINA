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

// Базовые типы, которые ищут cf_string.rs и другие через super::
pub type CFIndex = i32;
pub type CFOptionFlags = u32;
pub type CFHashCode = u32;
pub type CFTypeRef = crate::objc::id; // Обычно это указатель на объект

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

// Заглушки для функций управления памятью, если они нужны
pub fn CFRetain(obj: CFTypeRef) -> CFTypeRef { obj }
pub fn CFRelease(_obj: CFTypeRef) { }

// Если в проекте есть хелпер для хэширования, его тоже можно объявить тут
pub fn hash_helper(data: &str) -> CFHashCode {
    // Простейшая реализация хэша, если нет специфической
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish() as CFHashCode
}


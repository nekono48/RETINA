/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod cf_allocator;
pub mod cf_array;
pub mod cf_bundle;
pub mod cf_data;
pub mod cf_dictionary;
pub mod cf_locale;
pub mod cf_number;
pub mod cf_preferences;
pub mod cf_run_loop;
pub mod cf_run_loop_timer;
pub mod cf_socket;
pub mod cf_string;
pub mod cf_type;
pub mod cf_url;
pub mod time;

use crate::dyld::{FunctionExports, HostDylib};

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation",
    aliases: &[],
    class_exports: &[
        cf_run_loop_timer::CLASSES,
    ],
    constant_exports: &[
        cf_allocator::CONSTANTS,
        cf_bundle::CONSTANTS,
        cf_dictionary::CONSTANTS,
        cf_locale::CONSTANTS,
        cf_number::CONSTANTS,
        cf_preferences::CONSTANTS,
        cf_run_loop::CONSTANTS,
    ],
    function_exports: &[
        FUNCTIONS,
        cf_array::FUNCTIONS,
        cf_dictionary::FUNCTIONS,
        cf_bundle::FUNCTIONS,
        cf_socket::FUNCTIONS,
        cf_data::FUNCTIONS,
        cf_locale::FUNCTIONS,
        cf_number::FUNCTIONS,
        cf_preferences::FUNCTIONS,
        cf_run_loop::FUNCTIONS,
        cf_run_loop_timer::FUNCTIONS,
        cf_string::FUNCTIONS,
        cf_type::FUNCTIONS,
        cf_url::FUNCTIONS,
        time::FUNCTIONS,
    ],
};

pub use cf_type::{CFRelease, CFRetain, CFTypeRef};
pub type CFIndex = i32;

const FUNCTIONS: FunctionExports = &[
    crate::export_c_func!(CFShow(_)),
];

fn CFShow(env: &mut crate::Environment, obj: CFTypeRef) {
    use crate::frameworks::foundation::ns_string::to_rust_string;
    let description: crate::objc::id = crate::msg![env; obj description];
    log!("{}", to_rust_string(env, description));
}

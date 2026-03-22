/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Foundation framework.

pub mod ns_array;
pub mod ns_bundle;
pub mod ns_data;
pub mod ns_dictionary;
pub mod ns_error;
pub mod ns_number; // Added this line
pub mod ns_string;
pub mod ns_url;
pub mod ns_value;

use crate::dyld::FunctionExports;
use crate::objc::SelectorMap;
use crate::Environment;

/// Register all Foundation classes and functions.
pub fn install(env: &mut Environment, selectors: &mut SelectorMap) {
    // Register classes
    ns_array::register_class(env, selectors);
    ns_bundle::register_class(env, selectors);
    ns_data::register_class(env, selectors);
    ns_dictionary::register_class(env, selectors);
    ns_error::register_class(env, selectors);
    ns_number::register_class(env, selectors); // Added this line
    ns_string::register_class(env, selectors);
    ns_url::register_class(env, selectors);
    ns_value::register_class(env, selectors);
}

/// Exported C functions for the Foundation framework.
pub const FUNCTIONS: FunctionExports = &[
    // Usually includes things like NSLog, NSClassFromString, etc.
    // Ensure this list matches your current version's requirements.
];


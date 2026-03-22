/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod _nib_archive_decoder;
pub mod ns_array;
pub mod ns_autorelease_pool;
pub mod ns_bundle;
pub mod ns_character_set;
pub mod ns_coder;
pub mod ns_data;
pub mod ns_date;
pub mod ns_date_formatter;
pub mod ns_dictionary;
pub mod ns_enumerator;
pub mod ns_error;
pub mod ns_exception;
pub mod ns_file_handle;
pub mod ns_file_manager;
pub mod ns_keyed_archiver;
pub mod ns_keyed_unarchiver;
pub mod ns_locale;
pub mod ns_lock;
pub mod ns_log;
pub mod ns_notification;
pub mod ns_notification_center;
pub mod ns_null;
pub mod ns_number; // Наш новый модуль
pub mod ns_objc_runtime;
pub mod ns_object;
pub mod ns_process_info;
pub mod ns_property_list_serialization;
pub mod ns_run_loop;
pub mod ns_scanner;
pub mod ns_set;
pub mod ns_string;
pub mod ns_thread;
pub mod ns_time_zone;
pub mod ns_timer;
pub mod ns_url;
pub mod ns_url_connection;
pub mod ns_url_request;
pub mod ns_user_defaults;
pub mod ns_value;
pub mod ns_xml_parser;

use crate::dyld::{FunctionExports, HostDylib};

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/Foundation.framework/Foundation",
    aliases: &[],
    class_exports: &[
        _nib_archive_decoder::CLASSES,
        ns_array::CLASSES,
        ns_autorelease_pool::CLASSES,
        ns_bundle::CLASSES,
        ns_character_set::CLASSES,
        ns_coder::CLASSES,
        ns_data::CLASSES,
        ns_date::CLASSES,
        ns_date_formatter::CLASSES,
        ns_dictionary::CLASSES,
        ns_enumerator::CLASSES,
        ns_error::CLASSES,
        ns_file_handle::CLASSES,
        ns_file_manager::CLASSES,
        ns_keyed_archiver::CLASSES,
        ns_keyed_unarchiver::CLASSES,
        ns_locale::CLASSES,
        ns_lock::CLASSES,
        ns_notification::CLASSES,
        ns_notification_center::CLASSES,
        ns_null::CLASSES,
        ns_number::CLASSES, // Регистрация NSNumber
        ns_object::CLASSES,
        ns_process_info::CLASSES,
        ns_property_list_serialization::CLASSES,
        ns_run_loop::CLASSES,
        ns_scanner::CLASSES,
        ns_set::CLASSES,
        ns_string::CLASSES,
        ns_thread::CLASSES,
        ns_timer::CLASSES,
        ns_time_zone::CLASSES,
        ns_url::CLASSES,
        ns_url_connection::CLASSES,
        ns_url_request::CLASSES,
        ns_user_defaults::CLASSES,
        ns_value::CLASSES,
        ns_xml_parser::CLASSES,
    ],
    constant_exports: &[
        ns_error::CONSTANTS,
        ns_exception::CONSTANTS,
        ns_file_manager::CONSTANTS,
        ns_keyed_unarchiver::CONSTANTS,
        ns_locale::CONSTANTS,
        ns_run_loop::CONSTANTS,
    ],
    function_exports: &[
        FUNCTIONS,
        ns_exception::FUNCTIONS,
        ns_file_manager::FUNCTIONS,
        ns_log::FUNCTIONS,
        ns_objc_runtime::FUNCTIONS,
    ],
};

const FUNCTIONS: FunctionExports = &[];

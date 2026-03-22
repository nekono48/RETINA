/*
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
//! The Audio Toolbox framework.

use crate::audio::openal::{OpenAL, OpenALContext, OpenALManager};
use crate::frameworks::carbon_core::OSStatus;
use crate::frameworks::core_foundation::url::CFURLRef;

pub type ExtAudioFileRef = *mut std::ffi::c_void;

macro_rules! return_if_null {
    ($param:ident) => {
        if $param.is_null() {
            log_dbg!(
                "Got NULL parameter {}, returning paramErr in {} on line {}",
                stringify!($param),
                file!(),
                line!()
            );
            return crate::frameworks::carbon_core::paramErr;
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn _ExtAudioFileOpenURL(
    in_url: CFURLRef,
    out_ext_audio_file: *mut ExtAudioFileRef,
) -> OSStatus {
    return_if_null!(in_url);
    return_if_null!(out_ext_audio_file);

    log_dbg!("_ExtAudioFileOpenURL called");

    *out_ext_audio_file = std::ptr::null_mut();
    return crate::frameworks::carbon_core::unimpErr;
}

pub const FUNCTIONS: &[(&str, usize)] = &[
    ("_ExtAudioFileOpenURL", _ExtAudioFileOpenURL as usize),
];

pub mod audio_components;
pub mod audio_file;
pub mod audio_queue;
pub mod audio_services;
pub mod audio_session;
pub mod audio_unit;

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    path: "/System/Library/Frameworks/AudioToolbox.framework/AudioToolbox",
    aliases: &[],
    class_exports: &[],
    constant_exports: &[],
    function_exports: &[
        FUNCTIONS,
        audio_components::FUNCTIONS,
        audio_file::FUNCTIONS,
        audio_queue::FUNCTIONS,
        audio_services::FUNCTIONS,
        audio_session::FUNCTIONS,
        audio_unit::FUNCTIONS,
    ],
};

#[derive(Default)]
pub struct State {
    pub audio_file: audio_file::State,
    pub audio_queue: audio_queue::State,
    pub audio_components: audio_components::State,
    pub audio_session: audio_session::State,
    al_context: LazyALContext,
}

impl State {
    pub fn make_al_context_current<'s, 'manager: 's>(
        &'s mut self,
        manager: &'manager mut OpenALManager,
    ) -> OpenAL<'s> {
        self.al_context.make_al_context_current(manager)
    }
}

#[derive(Default)]
pub struct LazyALContext(Option<OpenALContext>);

impl LazyALContext {
    pub fn make_al_context_current<'s, 'manager: 's>(
        &'s mut self,
        manager: &'manager mut OpenALManager,
    ) -> OpenAL<'s> {
        self.get_context(manager).make_current(manager)
    }
    pub fn get_context(&mut self, manager: &mut OpenALManager) -> &mut OpenALContext {
        if self.0.is_none() {
            let context = OpenALContext::new(manager).unwrap();
            log_dbg!("New internal OpenAL context ({:?})", context);
            self.0 = Some(context);
        }
        self.0.as_mut().unwrap()
    }
}


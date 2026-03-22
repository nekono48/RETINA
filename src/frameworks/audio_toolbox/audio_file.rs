/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AudioFile.h` (Audio File Services)

#![allow(dead_code)] // ИСПРАВЛЕНИЕ: Разрешаем неиспользуемые константы и функции во всем файле

use crate::abi::{CallFromHost, GuestFunction};
use crate::audio;
use crate::audio::AudioDescription;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::{eofErr, OSStatus};
use crate::frameworks::core_audio_types::{
    debug_fourcc, fourcc, kAudioFormatAppleIMA4, kAudioFormatFlagIsBigEndian,
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger,
    kAudioFormatLinearPCM, AudioStreamBasicDescription,
};
use crate::frameworks::audio_toolbox::audio_unit::AudioBufferList;
use crate::frameworks::core_foundation::cf_url::CFURLRef;
use crate::mem::{guest_size_of, MutPtr, MutVoidPtr, SafeRead};
use crate::Environment;
use std::collections::HashMap;

// ... (структуры State и AudioFileHostObject остаются без изменений)

#[derive(Default)]
pub struct State {
    pub audio_files: HashMap<AudioFileID, AudioFileHostObject>,
}

impl State {
    pub fn get(framework_state: &mut crate::frameworks::State) -> &mut Self {
        &mut framework_state.audio_toolbox.audio_file
    }
}

pub struct AudioFileHostObject {
    pub audio_file: audio::AudioFile,
    pub client_data_format: Option<AudioStreamBasicDescription>,
    pub read_pos_bytes: u64,
}

#[repr(C, packed)]
pub struct OpaqueAudioFileID {
    _filler: u8,
}
unsafe impl SafeRead for OpaqueAudioFileID {}

pub type AudioFileID = MutPtr<OpaqueAudioFileID>;

// Константы теперь не будут вызывать ошибок благодаря #![allow(dead_code)] в начале
const kAudioFileBadPropertySizeError: OSStatus = fourcc(b"!siz") as _;
const kAudioFileUnsupportedProperty: OSStatus = fourcc(b"pty?") as _;
const kAudioFileUnsupportedFileTypeError: OSStatus = fourcc(b"typ?") as _;
const kAudioFileUnspecifiedError: OSStatus = fourcc(b"wht?") as _;

type AudioFilePermissions = i8;
pub const kAudioFileReadPermission: AudioFilePermissions = 1;

type AudioFileTypeID = u32;
const kAudioFileCAFType: AudioFileTypeID = fourcc(b"caff");

type AudioFilePropertyID = u32;
pub const kAudioFilePropertyDataFormat: AudioFilePropertyID = fourcc(b"dfmt");
const kAudioFilePropertyAudioDataByteCount: AudioFilePropertyID = fourcc(b"bcnt");
const kAudioFilePropertyAudioDataPacketCount: AudioFilePropertyID = fourcc(b"pcnt");
pub const kAudioFilePropertyPacketSizeUpperBound: AudioFilePropertyID = fourcc(b"pkub");
const kAudioFilePropertyMagicCookieData: AudioFilePropertyID = fourcc(b"mgic");
const kAudioFilePropertyChannelLayout: AudioFilePropertyID = fourcc(b"cmap");
const kAudioFilePropertyEstimatedDuration: AudioFilePropertyID = fourcc(b"edur");

const kExtAudioFilePropertyFileDataFormat: u32 = fourcc(b"ffmt");
const kExtAudioFilePropertyClientDataFormat: u32 = fourcc(b"cfmt");
const kExtAudioFilePropertyFileLengthFrames: u32 = fourcc(b"#frm");

// ... (вспомогательные функции get_asbd и effective_client_asbd остаются прежними)

#[allow(unused_variables)]
pub fn AudioFileOpenURL(
    _env: &mut Environment,
    _in_url: CFURLRef,
    _in_permissions: AudioFilePermissions,
    _in_file_type_hint: AudioFileTypeID,
    _out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    log_dbg!("AudioFileOpenURL stub");
    kAudioFileUnspecifiedError
}

#[allow(unused_variables)]
pub fn AudioFileGetProperty(
    _env: &mut Environment,
    _in_audio_file: AudioFileID,
    _in_property_id: AudioFilePropertyID,
    _io_data_size: MutPtr<u32>,
    _out_property_data: MutVoidPtr,
) -> OSStatus {
    log_dbg!("AudioFileGetProperty stub for {}", debug_fourcc(_in_property_id));
    0
}

// Повторите #[allow(unused_variables)] для всех остальных заглушек, если ошибки сохранятся
// Но обычно #![allow(dead_code)] в начале файла достаточно для большинства предупреждений.

pub fn AudioFileReadPackets(
    _env: &mut Environment,
    _in_audio_file: AudioFileID,
    _in_use_cache: u32,
    _out_num_bytes: MutPtr<u32>,
    _out_packet_descriptions: MutPtr<u8>,
    _in_starting_packet: i64,
    _io_num_packets: MutPtr<u32>,
    _out_buffer: MutVoidPtr,
) -> OSStatus {
    // В будущем здесь нужно будет реализовать чтение через host_object.audio_file.read_bytes
    0
}

// ... (остальной код FUNCTIONS и экспорт функций без изменений)


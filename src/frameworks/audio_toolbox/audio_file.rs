/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AudioFile.h` (Audio File Services)

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
use crate::frameworks::core_foundation::cf_url::CFURLRef;
use crate::frameworks::foundation::ns_url::to_rust_path;
use crate::mem::{guest_size_of, GuestUSize, MutPtr, MutVoidPtr, SafeRead};
use crate::Environment;
use std::collections::HashMap;

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
}

#[repr(C, packed)]
pub struct OpaqueAudioFileID {
    _filler: u8,
}
unsafe impl SafeRead for OpaqueAudioFileID {}

pub type AudioFileID = MutPtr<OpaqueAudioFileID>;

#[allow(dead_code)]
const kAudioFileFileNotFoundError: OSStatus = -43;
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

// Константа для нового свойства
const kExtAudioFileProperty_FileDataFormat: AudioFilePropertyID = fourcc(b"ffmt");

// --- Extended Audio File Services Implementation ---

pub fn ExtAudioFileOpenURL(
    env: &mut Environment,
    in_url: CFURLRef,
    out_ext_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    return_if_null!(in_url);
    return_if_null!(out_ext_audio_file);
    log_dbg!("ExtAudioFileOpenURL: redirecting to AudioFileOpenURL");
    AudioFileOpenURL(env, in_url, kAudioFileReadPermission, 0, out_ext_audio_file)
}

pub fn ExtAudioFileGetProperty(
    env: &mut Environment,
    in_ext_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    io_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    log_dbg!(
        "ExtAudioFileGetProperty(prop: {})",
        debug_fourcc(in_property_id)
    );

    // Если запрашивают ffmt, подменяем на dfmt, так как в нашей упрощенной схеме они идентичны
    let prop_id = if in_property_id == kExtAudioFileProperty_FileDataFormat {
        kAudioFilePropertyDataFormat
    } else {
        in_property_id
    };

    AudioFileGetProperty(env, in_ext_audio_file, prop_id, io_data_size, out_property_data)
}

// --- Audio File Services Implementation ---

pub fn AudioFileOpenURL(
    env: &mut Environment,
    in_file_ref: CFURLRef,
    in_permissions: AudioFilePermissions,
    in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    return_if_null!(in_file_ref);
    assert!(in_permissions == kAudioFileReadPermission);

    match in_file_type_hint {
        0 => {}
        kAudioFileCAFType => {
            log!("Ignoring 'caff' file type hint for AudioFileOpenURL()");
        }
        _ => unimplemented!(),
    }

    let path = to_rust_path(env, in_file_ref);
    let audio_file = match audio::AudioFile::open_for_reading(path, &env.fs) {
        Ok(audio_file) => audio_file,
        Err(error) => {
            log!("Warning: AudioFileOpenURL() for path {:?} failed", in_file_ref);
            return match error {
                audio::AudioFileOpenError::FileDecodeError => kAudioFileUnsupportedFileTypeError,
                _ => kAudioFileUnspecifiedError,
            };
        }
    };

    let host_object = AudioFileHostObject { audio_file };
    let guest_audio_file = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    State::get(&mut env.framework_state)
        .audio_files
        .insert(guest_audio_file, host_object);

    env.mem.write(out_audio_file, guest_audio_file);
    log_dbg!("AudioFileOpenURL() opened path {:?}, handle: {:?}", in_file_ref, guest_audio_file);
    0
}

pub fn AudioFileOpenWithCallbacks(
    env: &mut Environment,
    client_data: MutVoidPtr,
    read_callback: GuestFunction,
    _write_callback: GuestFunction,
    getsize_callback: GuestFunction,
    _setsize_callback: GuestFunction,
    in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    if _write_callback.to_ptr().is_null() || _setsize_callback.to_ptr().is_null() {
        log_dbg!("AudioFileOpenWithCallbacks() with unsupported write/set_size callbacks!");
    }
    if in_file_type_hint != 0 {
        log!("Ignoring file type hint for AudioFileOpenWithCallbacks()");
    }

    let size: i64 = getsize_callback.call_from_host(env, (client_data,));
    let size: u32 = size.try_into().unwrap();
    assert!(size != 0);

    let data_ptr: MutPtr<u8> = env.mem.alloc(size).cast();
    let bytes_read_ptr: MutPtr<u32> = env.mem.alloc(guest_size_of::<u32>()).cast();

    env.mem.write(bytes_read_ptr, 0);
    let status: OSStatus = read_callback.call_from_host(env, (client_data, 0_i64, size, data_ptr, bytes_read_ptr));
    if status != 0 { return status; }

    let data_vec = env.mem.bytes_at(data_ptr, env.mem.read(bytes_read_ptr)).to_vec();
    let Ok(audio_file) = audio::AudioFile::read_from_vec(data_vec) else {
        return kAudioFileUnsupportedFileTypeError;
    };

    let guest_audio_file = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    let host_object = AudioFileHostObject { audio_file };
    State::get(&mut env.framework_state).audio_files.insert(guest_audio_file, host_object);
    env.mem.write(out_audio_file, guest_audio_file);
    0
}

fn property_size(property_id: AudioFilePropertyID) -> GuestUSize {
    match property_id {
        kAudioFilePropertyDataFormat | kExtAudioFileProperty_FileDataFormat => {
            guest_size_of::<AudioStreamBasicDescription>()
        }
        kAudioFilePropertyAudioDataByteCount => guest_size_of::<u64>(),
        kAudioFilePropertyAudioDataPacketCount => guest_size_of::<u64>(),
        kAudioFilePropertyPacketSizeUpperBound => guest_size_of::<u32>(),
        kAudioFilePropertyEstimatedDuration => guest_size_of::<f64>(),
        _ => unimplemented!("Unimplemented property ID: {}", debug_fourcc(property_id)),
    }
}

fn AudioFileGetPropertyInfo(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    out_data_size: MutPtr<u32>,
    is_writable: MutPtr<u32>,
) -> OSStatus {
    return_if_null!(in_audio_file);
    if in_property_id == kAudioFilePropertyMagicCookieData || in_property_id == kAudioFilePropertyChannelLayout {
        if !out_data_size.is_null() { env.mem.write(out_data_size, 0); }
        if !is_writable.is_null() { env.mem.write(is_writable, 0); }
        return kAudioFileUnsupportedProperty;
    }
    if !out_data_size.is_null() {
        env.mem.write(out_data_size, property_size(in_property_id));
    }
    if !is_writable.is_null() { env.mem.write(is_writable, 0); }
    0
}

pub fn AudioFileGetProperty(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    io_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_audio_file);
    let required_size = property_size(in_property_id);
    if env.mem.read(io_data_size) < required_size {
        return kAudioFileBadPropertySizeError;
    }

    let host_object = State::get(&mut env.framework_state).audio_files.get_mut(&in_audio_file).unwrap();

    match in_property_id {
        kAudioFilePropertyDataFormat | kExtAudioFileProperty_FileDataFormat => {
            let audio::AudioDescription { sample_rate, format, bytes_per_packet, frames_per_packet, channels_per_frame, bits_per_channel } = host_object.audio_file.audio_description();
            let desc: AudioStreamBasicDescription = match format {
                audio::AudioFormat::LinearPcm { is_float, is_little_endian } => {
                    let is_packed = (bits_per_channel * channels_per_frame * frames_per_packet) == (bytes_per_packet * 8);
                    let format_flags = (u32::from(is_float) * kAudioFormatFlagIsFloat)
                        | (u32::from((!is_float) && matches!(bits_per_channel, 16 | 24)) * kAudioFormatFlagIsSignedInteger)
                        | (u32::from(is_packed) * kAudioFormatFlagIsPacked)
                        | (u32::from(!is_little_endian) * kAudioFormatFlagIsBigEndian);
                    AudioStreamBasicDescription {
                        sample_rate, format_id: kAudioFormatLinearPCM, format_flags,
                        bytes_per_packet, frames_per_packet, bytes_per_frame: bytes_per_packet / frames_per_packet,
                        channels_per_frame, bits_per_channel, _reserved: 0,
                    }
                }
                audio::AudioFormat::AppleIma4 => {
                    AudioStreamBasicDescription {
                        sample_rate, format_id: kAudioFormatAppleIMA4, format_flags: 0,
                        bytes_per_packet, frames_per_packet, bytes_per_frame: 0,
                        channels_per_frame, bits_per_channel, _reserved: 0,
                    }
                }
            };
            env.mem.write(out_property_data.cast(), desc);
        }
        kAudioFilePropertyAudioDataByteCount => {
            env.mem.write(out_property_data.cast(), host_object.audio_file.byte_count());
        }
        kAudioFilePropertyAudioDataPacketCount => {
            env.mem.write(out_property_data.cast(), host_object.audio_file.packet_count());
        }
        kAudioFilePropertyPacketSizeUpperBound => {
            env.mem.write(out_property_data.cast(), host_object.audio_file.packet_size_upper_bound());
        }
        kAudioFilePropertyEstimatedDuration => {
            let desc = host_object.audio_file.audio_description();
            let duration = host_object.audio_file.byte_count() as f64 * desc.frames_per_packet as f64 / (desc.bytes_per_packet as f64 * desc.sample_rate);
            env.mem.write(out_property_data.cast(), duration);
        }
        _ => unreachable!(),
    }
    0
}

fn AudioFileReadBytes(env: &mut Environment, in_audio_file: AudioFileID, _cache: bool, in_start: i64, io_bytes: MutPtr<u32>, out_buf: MutVoidPtr) -> OSStatus {
    return_if_null!(in_audio_file);
    let host_object = State::get(&mut env.framework_state).audio_files.get_mut(&in_audio_file).unwrap();
    let to_read = env.mem.read(io_bytes);
    let buf = env.mem.bytes_at_mut(out_buf.cast(), to_read);
    let read = host_object.audio_file.read_bytes(in_start.try_into().unwrap(), buf).unwrap();
    env.mem.write(io_bytes, read.try_into().unwrap());
    if read < to_read as usize { eofErr } else { 0 }
}

fn AudioFileReadPacketData(env: &mut Environment, file: AudioFileID, cache: bool, out_bytes: MutPtr<u32>, out_desc: MutVoidPtr, start: i64, io_packets: MutPtr<u32>, out_buf: MutVoidPtr) -> OSStatus {
    AudioFileReadPackets(env, file, cache, out_bytes, out_desc, start, io_packets, out_buf)
}

pub fn AudioFileReadPackets(env: &mut Environment, file: AudioFileID, cache: bool, out_bytes: MutPtr<u32>, _desc: MutVoidPtr, start: i64, io_packets: MutPtr<u32>, out_buf: MutVoidPtr) -> OSStatus {
    return_if_null!(file);
    let host_object = State::get(&mut env.framework_state).audio_files.get_mut(&file).unwrap();
    let p_size = host_object.audio_file.packet_size_fixed();
    let n_packets = env.mem.read(io_packets);
    let start_byte = i64::from(p_size) * start;
    let to_read = n_packets * p_size;
    env.mem.write(out_bytes, to_read);
    let res = AudioFileReadBytes(env, file, cache, start_byte, out_bytes, out_buf);
    let read = env.mem.read(out_bytes);
    env.mem.write(io_packets, read / p_size);
    res
}

pub fn AudioFileClose(env: &mut Environment, file: AudioFileID) -> OSStatus {
    return_if_null!(file);
    if State::get(&mut env.framework_state).audio_files.remove(&file).is_none() { return kAudioFileUnspecifiedError; }
    env.mem.free(file.cast());
    0
}

fn AudioFileStreamOpen(_env: &mut Environment, _data: MutVoidPtr, _lp: MutVoidPtr, _pp: MutVoidPtr, _hint: AudioFileTypeID, _out: MutVoidPtr) -> OSStatus {
    kAudioFileUnspecifiedError
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioFileOpenURL(_, _, _, _)),
    export_c_func!(ExtAudioFileOpenURL(_, _)),
    export_c_func!(ExtAudioFileGetProperty(_, _, _, _)),
    export_c_func!(AudioFileGetPropertyInfo(_, _, _, _)),
    export_c_func!(AudioFileGetProperty(_, _, _, _)),
    export_c_func!(AudioFileReadBytes(_, _, _, _, _)),
    export_c_func!(AudioFileReadPackets(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileReadPacketData(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileOpenWithCallbacks(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileClose(_)),
    export_c_func!(AudioFileStreamOpen(_, _, _, _, _)),
];


/*
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
//! AudioFile.h (Audio File Services)

use crate::abi::{CallFromHost, GuestFunction};
use crate::audio;
use crate::audio::AudioDescription;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::{eofErr, OSStatus};
use crate::frameworks::core_audio_types::{
    debug_fourcc, fourcc, kAudioFormatAppleIMA4, kAudioFormatFlagIsBigEndian,
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsPacked,
    kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
    AudioStreamBasicDescription,
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
const kAudioFilePropertyAudioDataByteCount: AudioFilePropertyID =
    fourcc(b"bcnt");
const kAudioFilePropertyAudioDataPacketCount: AudioFilePropertyID =
    fourcc(b"pcnt");
pub const kAudioFilePropertyPacketSizeUpperBound: AudioFilePropertyID =
    fourcc(b"pkub");
const kAudioFilePropertyMagicCookieData: AudioFilePropertyID = fourcc(b"mgic");
const kAudioFilePropertyChannelLayout: AudioFilePropertyID = fourcc(b"cmap");
const kAudioFilePropertyEstimatedDuration: AudioFilePropertyID =
    fourcc(b"edur");

const kExtAudioFilePropertyFileDataFormat: u32 = fourcc(b"ffmt");
const kExtAudioFilePropertyClientDataFormat: u32 = fourcc(b"cfmt");
const kExtAudioFilePropertyFileChannelLayout: u32 = fourcc(b"fclo");
const kExtAudioFilePropertyClientChannelLayout: u32 = fourcc(b"cclo");
const kExtAudioFilePropertyFileLengthFrames: u32 = fourcc(b"#frm");

fn make_asbd_from_audio_description(
    audio_file: &audio::AudioFile,
) -> AudioStreamBasicDescription {
    let AudioDescription {
        sample_rate,
        format,
        bytes_per_packet,
        frames_per_packet,
        channels_per_frame,
        bits_per_channel,
    } = audio_file.audio_description();

    match format {
        audio::AudioFormat::LinearPcm {
            is_float,
            is_little_endian,
        } => {
            let is_packed = (bits_per_channel * channels_per_frame
                * frames_per_packet) == (bytes_per_packet * 8);
            let format_flags = (u32::from(is_float) * kAudioFormatFlagIsFloat)
                | (u32::from((!is_float) && matches!(bits_per_channel, 16 | 24))
                    * kAudioFormatFlagIsSignedInteger)
                | (u32::from(is_packed) * kAudioFormatFlagIsPacked)
                | (u32::from(!is_little_endian) * kAudioFormatFlagIsBigEndian);

            AudioStreamBasicDescription {
                sample_rate,
                format_id: kAudioFormatLinearPCM,
                format_flags,
                bytes_per_packet,
                frames_per_packet,
                bytes_per_frame: bytes_per_packet / frames_per_packet,
                channels_per_frame,
                bits_per_channel,
                _reserved: 0,
            }
        }
        audio::AudioFormat::AppleIma4 => AudioStreamBasicDescription {
            sample_rate,
            format_id: kAudioFormatAppleIMA4,
            format_flags: 0,
            bytes_per_packet,
            frames_per_packet,
            bytes_per_frame: 0,
            channels_per_frame,
            bits_per_channel,
            _reserved: 0,
        },
    }
}

// --- Extended Audio File Services ---

pub fn ExtAudioFileOpenURL(
    env: &mut Environment,
    in_url: CFURLRef,
    out_ext_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    return_if_null!(in_url);
    return_if_null!(out_ext_audio_file);

    AudioFileOpenURL(env, in_url, kAudioFileReadPermission, 0, out_ext_audio_file)
}

pub fn ExtAudioFileGetProperty(
    env: &mut Environment,
    in_ext_audio_file: AudioFileID,
    in_property_id: u32,
    io_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    let host_object = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_ext_audio_file)
        .unwrap();

    match in_property_id {
        x if x == kExtAudioFilePropertyFileDataFormat
            || x == kExtAudioFilePropertyClientDataFormat =>
        {
            let req_size = guest_size_of::<AudioStreamBasicDescription>() as u32;
            if env.mem.read(io_data_size) != req_size {
                return kAudioFileBadPropertySizeError;
            }
            let desc = make_asbd_from_audio_description(&host_object.audio_file);
            env.mem.write(out_property_data.cast(), desc);
            0
        }
        x if x == kExtAudioFilePropertyFileLengthFrames => {
            if !io_data_size.is_null() {
                env.mem.write(io_data_size, guest_size_of::<i64>() as u32);
            }
            if !out_property_data.is_null() {
                let desc = host_object.audio_file.audio_description();
                let frames = host_object.audio_file.packet_count() as i64
                    * desc.frames_per_packet as i64;
                env.mem.write(out_property_data.cast(), frames);
            }
            0
        }
        x if x == kExtAudioFilePropertyFileChannelLayout
            || x == kExtAudioFilePropertyClientChannelLayout =>
        {
            if !io_data_size.is_null() {
                env.mem.write(io_data_size, 0);
            }
            kAudioFileUnsupportedProperty
        }
        _ => AudioFileGetProperty(
            env,
            in_ext_audio_file,
            in_property_id,
            io_data_size,
            out_property_data,
        ),
    }
}

pub fn ExtAudioFileSetProperty(
    env: &mut Environment,
    in_ext_audio_file: AudioFileID,
    in_property_id: u32,
    in_property_data_size: u32,
    in_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    match in_property_id {
        x if x == kExtAudioFilePropertyClientDataFormat
            || x == kExtAudioFilePropertyFileDataFormat =>
        {
            let min_size = guest_size_of::<AudioStreamBasicDescription>() as u32;
            if in_property_data_size < min_size {
                return kAudioFileBadPropertySizeError;
            }
            let desc: AudioStreamBasicDescription = env.mem.read(
                in_property_data.cast()
            );
            log_dbg!("ExtAudioFileSetProperty format stub: {:?}", desc);
            0
        }
        _ => {
            log_dbg!(
                "ExtAudioFileSetProperty stub: {}",
                debug_fourcc(in_property_id)
            );
            0
        }
    }
}

// --- Audio File Services ---

pub fn AudioFileOpenURL(
    env: &mut Environment,
    in_file_ref: CFURLRef,
    in_permissions: AudioFilePermissions,
    _in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    return_if_null!(in_file_ref);
    assert!(in_permissions == kAudioFileReadPermission);

    let path = to_rust_path(env, in_file_ref);
    let audio_file = match audio::AudioFile::open_for_reading(path, &env.fs) {
        Ok(af) => af,
        Err(_) => return kAudioFileUnspecifiedError,
    };

    let guest_af = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    State::get(&mut env.framework_state).audio_files.insert(
        guest_af,
        AudioFileHostObject { audio_file },
    );
    env.mem.write(out_audio_file, guest_af);
    0
}

pub fn AudioFileOpenWithCallbacks(
    env: &mut Environment,
    client_data: MutVoidPtr,
    read_callback: GuestFunction,
    _write_callback: GuestFunction,
    getsize_callback: GuestFunction,
    _setsize_callback: GuestFunction,
    _in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    let size: i64 = getsize_callback.call_from_host(env, (client_data,));
    let size_u32: u32 = size.try_into().unwrap();
    let data_ptr: MutPtr<u8> = env.mem.alloc(size_u32).cast();
    let bytes_read_ptr: MutPtr<u32> = env.mem.alloc_and_write(0u32).cast();

    let status: OSStatus = read_callback.call_from_host(
        env,
        (client_data, 0_i64, size_u32, data_ptr, bytes_read_ptr),
    );
    if status != 0 {
        return status;
    }

    let read_len = env.mem.read(bytes_read_ptr);
    let data_vec = env.mem.bytes_at(data_ptr, read_len).to_vec();
    let audio_file = audio::AudioFile::read_from_vec(data_vec).unwrap();

    let guest_af = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    State::get(&mut env.framework_state).audio_files.insert(
        guest_af,
        AudioFileHostObject { audio_file },
    );
    env.mem.write(out_audio_file, guest_af);
    0
}

fn property_size(property_id: AudioFilePropertyID) -> Option<GuestUSize> {
    match property_id {
        kAudioFilePropertyDataFormat => {
            Some(guest_size_of::<AudioStreamBasicDescription>())
        }
        kAudioFilePropertyAudioDataByteCount => Some(guest_size_of::<u64>()),
        kAudioFilePropertyAudioDataPacketCount => Some(guest_size_of::<u64>()),
        kAudioFilePropertyPacketSizeUpperBound => Some(guest_size_of::<u32>()),
        kAudioFilePropertyEstimatedDuration => Some(guest_size_of::<f64>()),
        _ => None,
    }
}

pub fn AudioFileGetPropertyInfo(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    out_data_size: MutPtr<u32>,
    is_writable: MutPtr<u32>,
) -> OSStatus {
    return_if_null!(in_audio_file);

    if let Some(size) = property_size(in_property_id) {
        if !out_data_size.is_null() {
            env.mem.write(out_data_size, size as u32);
        }
        if !is_writable.is_null() {
            env.mem.write(is_writable, 0);
        }
        0
    } else {
        kAudioFileUnsupportedProperty
    }
}

pub fn AudioFileGetProperty(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    io_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_audio_file);

    let Some(req_size) = property_size(in_property_id) else {
        return kAudioFileUnsupportedProperty;
    };

    if env.mem.read(io_data_size) != req_size as u32 {
        return kAudioFileBadPropertySizeError;
    }

    let host_object = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_audio_file)
        .unwrap();

    match in_property_id {
        kAudioFilePropertyDataFormat => {
            let desc = make_asbd_from_audio_description(&host_object.audio_file);
            env.mem.write(out_property_data.cast(), desc);
        }
        kAudioFilePropertyAudioDataByteCount => {
            env.mem.write(
                out_property_data.cast(),
                host_object.audio_file.byte_count(),
            );
        }
        kAudioFilePropertyAudioDataPacketCount => {
            env.mem.write(
                out_property_data.cast(),
                host_object.audio_file.packet_count(),
            );
        }
        kAudioFilePropertyPacketSizeUpperBound => {
            env.mem.write(
                out_property_data.cast(),
                host_object.audio_file.packet_size_upper_bound(),
            );
        }
        kAudioFilePropertyEstimatedDuration => {
            let desc = host_object.audio_file.audio_description();
            let dur = host_object.audio_file.byte_count() as f64
                * desc.frames_per_packet as f64
                / (desc.bytes_per_packet as f64 * desc.sample_rate);
            env.mem.write(out_property_data.cast(), dur);
        }
        _ => return kAudioFileUnsupportedProperty,
    }
    0
}

pub fn AudioFileReadBytes(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    _use_cache: bool,
    in_start: i64,
    io_size: MutPtr<u32>,
    out_buf: MutVoidPtr,
) -> OSStatus {
    let host_obj = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_audio_file)
        .unwrap();
    let to_read = env.mem.read(io_size);
    let buf = env.mem.bytes_at_mut(out_buf.cast(), to_read);

    let offset: u64 = in_start.try_into().unwrap();
    let read = host_obj.audio_file.read_bytes(offset, buf).unwrap();

    env.mem.write(io_size, read as u32);
    if read < to_read as usize {
        eofErr
    } else {
        0
    }
}

pub fn AudioFileReadPackets(
    env: &mut Environment,
    in_af: AudioFileID,
    cache: bool,
    out_bytes: MutPtr<u32>,
    _descriptions: MutVoidPtr,
    in_start_pkt: i64,
    io_pkts: MutPtr<u32>,
    out_buf: MutVoidPtr,
) -> OSStatus {
    let host_obj = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_af)
        .unwrap();
    let pkt_size = host_obj.audio_file.packet_size_fixed();
    let to_read = env.mem.read(io_pkts);
    let start_byte = in_start_pkt * pkt_size as i64;

    env.mem.write(out_bytes, to_read * pkt_size);
    AudioFileReadBytes(env, in_af, cache, start_byte, out_bytes, out_buf)
}

pub fn AudioFileReadPacketData(
    env: &mut Environment,
    in_af: AudioFileID,
    cache: bool,
    out_bytes: MutPtr<u32>,
    descs: MutVoidPtr,
    in_start_pkt: i64,
    io_pkts: MutPtr<u32>,
    out_buf: MutVoidPtr,
) -> OSStatus {
    AudioFileReadPackets(
        env, in_af, cache, out_bytes, descs, in_start_pkt, io_pkts, out_buf,
    )
}

pub fn AudioFileClose(env: &mut Environment, in_af: AudioFileID) -> OSStatus {
    if State::get(&mut env.framework_state)
        .audio_files
        .remove(&in_af)
        .is_some()
    {
        env.mem.free(in_af.cast());
        0
    } else {
        kAudioFileUnspecifiedError
    }
}

fn AudioFileStreamOpen(
    _env: &mut Environment,
    _: MutVoidPtr,
    _: MutVoidPtr,
    _: MutVoidPtr,
    _: u32,
    _: MutVoidPtr,
) -> OSStatus {
    kAudioFileUnspecifiedError
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioFileOpenURL(_, _, _, _)),
    export_c_func!(ExtAudioFileOpenURL(_, _)),
    export_c_func!(ExtAudioFileGetProperty(_, _, _, _)),
    export_c_func!(ExtAudioFileSetProperty(_, _, _, _)),
    export_c_func!(AudioFileGetPropertyInfo(_, _, _, _)),
    export_c_func!(AudioFileGetProperty(_, _, _, _)),
    export_c_func!(AudioFileReadBytes(_, _, _, _, _)),
    export_c_func!(AudioFileReadPackets(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileReadPacketData(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileOpenWithCallbacks(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileClose(_)),
    export_c_func!(AudioFileStreamOpen(_, _, _, _, _)),
];


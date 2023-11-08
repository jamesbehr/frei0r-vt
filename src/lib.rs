extern crate libc;

use rgb::{RGB8, RGBA8};
use std::ffi::{CStr, CString};

pub struct Vt {
    width: u32,
    height: u32,
    path: CString,
}

pub struct Theme {
    pub background: RGB8,
    pub foreground: RGB8,
    palette: [RGB8; 16],
}

#[repr(C)]
pub struct F0rPluginInfo {
    name: *const libc::c_char,
    author: *const libc::c_char,
    plugin_type: libc::c_int,
    color_model: libc::c_int,
    frei0r_version: libc::c_int,
    major_version: libc::c_int,
    minor_version: libc::c_int,
    num_params: libc::c_int,
    explanation: *const libc::c_char,
}

#[repr(C)]
pub struct F0rParamInfo {
    name: *const libc::c_char,
    param_type: libc::c_int,
    explanation: *const libc::c_char,
}

#[no_mangle]
pub fn f0r_init() -> libc::c_int {
    1
}

#[no_mangle]
pub fn f0r_deinit() {}

#[no_mangle]
pub unsafe extern "C" fn f0r_get_plugin_info(info: *mut F0rPluginInfo) {
    (*info).name = b"VT\0".as_ptr() as *const libc::c_char;
    (*info).author = b"James Behr\0".as_ptr() as *const libc::c_char;
    (*info).plugin_type = 1; // F0R_PLUGIN_TYPE_SOURCE
    (*info).color_model = 1; // F0R_COLOR_MODEL_RGBA8888
    (*info).frei0r_version = 1; // FREI0R_MAJOR_VERSION
    (*info).major_version = 0;
    (*info).minor_version = 1;
    (*info).num_params = 1;
    (*info).explanation = b"Generates a VT terminal screencast\0".as_ptr() as *const libc::c_char;
}

#[no_mangle]
pub extern "C" fn f0r_get_param_info(info: *mut F0rParamInfo, index: libc::c_int) {
    match index {
        0 => unsafe {
            (*info).name = b"Path\0".as_ptr() as *const libc::c_char;
            (*info).param_type = 4; // F0R_PARAM_STRING
            (*info).explanation = b"Path to screencast file\0".as_ptr() as *const libc::c_char;
        },
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn f0r_construct(width: u32, height: u32) -> *mut Vt {
    let path = CString::new("").unwrap();
    let inst = Box::new(Vt {
        width,
        height,
        path,
    });
    Box::into_raw(inst)
}

#[no_mangle]
pub unsafe extern "C" fn f0r_destruct(inst: *mut Vt) {
    let _ = Box::from_raw(inst);
}

#[no_mangle]
pub extern "C" fn f0r_set_param_value(inst: *mut Vt, param: *mut libc::c_void, index: libc::c_int) {
    let inst = unsafe { &mut *inst };

    match index {
        0 => unsafe {
            let p = param as *const *const libc::c_char;
            inst.path = CStr::from_ptr(*p).into();
        },
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn f0r_get_param_value(inst: *mut Vt, param: *mut libc::c_void, index: libc::c_int) {
    let inst = unsafe { &*inst };

    match index {
        0 => unsafe {
            let p = param as *mut *const libc::c_char;
            *p = inst.path.as_ptr();
        },
        _ => {}
    }
}

fn blend(fg: RGBA8, bg: RGBA8, ratio: u8) -> RGBA8 {
    let ratio = ratio as u16;

    RGBA8::new(
        ((bg.r as u16) * (255 - ratio) / 256) as u8 + ((fg.r as u16) * ratio / 256) as u8,
        ((bg.g as u16) * (255 - ratio) / 256) as u8 + ((fg.g as u16) * ratio / 256) as u8,
        ((bg.b as u16) * (255 - ratio) / 256) as u8 + ((fg.b as u16) * ratio / 256) as u8,
        255,
    )
}

#[no_mangle]
pub extern "C" fn f0r_update(
    inst: *mut Vt,
    _time: libc::c_double,
    _input: *mut u32,
    output: *mut u32,
) {
    let inst = unsafe { &*inst };

    let cols = 81;
    let rows = 20;
    let raw = "> # Welcome to asciinema!\r\n> # See how easy it is to record a terminal session\r\n> # First install the asciinema recorder\r\n> brew install asciinema\r\r\n\u{001b}[34m==>\u{001b}[0m \u{001b}[1mDownloading https://homebrew.bintray.com/bottles/asciinema-2.0.2_2.catalina.bottle.1.tar.gz\u{001b}[0m\r\n\u{001b}[34m==>\u{001b}[0m \u{001b}[1mDownloading from https://akamai.bintray.com/4a/4ac59de631594cea60621b45d85214e39a90a0ba8ddf4eeec5cba34bd6145711\u{001b}[0m\r\n\r##############                                                            19.7%\r######################################################################## 100.0%\r\n\u{001b}[34m==>\u{001b}[0m \u{001b}[1mPouring asciinema-2.0.2_2.catalina.bottle.1.tar.gz\u{001b}[0m\r\n🍺  /usr/local/Cellar/asciinema/2.0.2_2: 613 files, 6.4MB\r\n> # Now start recording\r\n> asciinema rec\r\n\u{001b}[0;32masciinema: recording asciicast to /tmp/u52erylk-ascii.cast\u{001b}[0m\r\n\u{001b}[0;32masciinema: press <ctrl-d> or type \"exit\" when you're done\u{001b}[0m\r\n\u{001b}[?1034hbash-3.2$ # I am in a new shell instance which is being recorded now\r\r\nbash-3.2$ sha1sum /etc/f* | tail -n 10 | lolcat -F 0.3\r\r\n\u{001b}[38;5;184md\u{001b}[0m\u{001b}[38;5;184ma\u{001b}[0m\u{001b}[38;5;184m3\u{001b}[0m\u{001b}[38;5;214m9\u{001b}[0m\u{001b}[38;5;214ma\u{001b}[0m\u{001b}[38;5;214m3\u{001b}[0m\u{001b}[38;5;208me\u{001b}[0m\u{001b}[38;5;208me\u{001b}[0m\u{001b}[38;5;208m5\u{001b}[0m\u{001b}[38;5;208me\u{001b}[0m\u{001b}[38;5;203m6\u{001b}[0m\u{001b}[38;5;203mb\u{001b}[0m\u{001b}[38;5;203m4\u{001b}[0m\u{001b}[38;5;203mb\u{001b}[0m\u{001b}[38;5;198m0\u{001b}[0m\u{001b}[38;5;198md\u{001b}[0m\u{001b}[38;5;198m3\u{001b}[0m\u{001b}[38;5;199m2\u{001b}[0m\u{001b}[38;5;199m5\u{001b}[0m\u{001b}[38;5;199m5\u{001b}[0m\u{001b}[38;5;164mb\u{001b}[0m\u{001b}[38;5;164mf\u{001b}[0m\u{001b}[38;5;164me\u{001b}[0m\u{001b}[38;5;164mf\u{001b}[0m\u{001b}[38;5;129m9\u{001b}[0m\u{001b}[38;5;129m5\u{001b}[0m\u{001b}[38;5;129m6\u{001b}[0m\u{001b}[38;5;93m0\u{001b}[0m\u{001b}[38;5;93m1\u{001b}[0m\u{001b}[38;5;93m8\u{001b}[0m\u{001b}[38;5;93m9\u{001b}[0m\u{001b}[38;5;63m0\u{001b}[0m\u{001b}[38;5;63ma\u{001b}[0m\u{001b}[38;5;63mf\u{001b}[0m\u{001b}[38;5;63md\u{001b}[0m\u{001b}[38;5;33m8\u{001b}[0m\u{001b}[38;5;33m0\u{001b}[0m\u{001b}[38;5;33m7\u{001b}[0m\u{001b}[38;5;39m0\u{001b}[0m\u{001b}[38;5;39m9\u{001b}[0m\u{001b}[38;5;39m \u{001b}[0m\u{001b}[38;5;44m \u{001b}[0m\u{001b}[38;5;44m/\u{001b}[0m\u{001b}[38;5;44me\u{001b}[0m\u{001b}[38;5;44mt\u{001b}[0m\u{001b}[38;5;49mc\u{001b}[0m\u{001b}[38;5;49m/\u{001b}[0m\u{001b}[38;5;49mf\u{001b}[0m\u{001b}[38;5;48mi\u{001b}[0m\u{001b}[38;5;48mn\u{001b}[0m\u{001b}[38;5;48md\u{001b}[0m\u{001b}[38;5;84m.\u{001b}[0m\u{001b}[38;5;83mc\u{001b}[0m\u{001b}[38;5;83mo\u{001b}[0m\u{001b}[38;5;83md\u{001b}[0m\u{001b}[38;5;119me\u{001b}[0m\u{001b}[38;5;118ms\u{001b}[0m\r\r\n\u{001b}[38;5;214m8\u{001b}[0m\u{001b}[38;5;214m8\u{001b}[0m\u{001b}[38;5;214md\u{001b}[0m\u{001b}[38;5;208md\u{001b}[0m\u{001b}[38;5;208m3\u{001b}[0m\u{001b}[38;5;208me\u{001b}[0m\u{001b}[38;5;208ma\u{001b}[0m\u{001b}[38;5;203m7\u{001b}[0m\u{001b}[38;5;203mf\u{001b}[0m\u{001b}[38;5;203mf\u{001b}[0m\u{001b}[38;5;203mc\u{001b}[0m\u{001b}[38;5;198mb\u{001b}[0m\u{001b}[38;5;198mb\u{001b}[0m\u{001b}[38;5;198m9\u{001b}[0m\u{001b}[38;5;199m1\u{001b}[0m\u{001b}[38;5;199m0\u{001b}[0m\u{001b}[38;5;199mf\u{001b}[0m\u{001b}[38;5;164mb\u{001b}[0m\u{001b}[38;5;164md\u{001b}[0m\u{001b}[38;5;164m1\u{001b}[0m\u{001b}[38;5;164md\u{001b}[0m\u{001b}[38;5;129m9\u{001b}[0m\u{001b}[38;5;129m2\u{001b}[0m\u{001b}[38;5;129m1\u{001b}[0m\u{001b}[38;5;93m8\u{001b}[0m\u{001b}[38;5;93m1\u{001b}[0m\u{001b}[38;5;93m1\u{001b}[0m\u{001b}[38;5;93m8\u{001b}[0m\u{001b}[38;5;63m1\u{001b}[0m\u{001b}[38;5;63m7\u{001b}[0m\u{001b}[38;5;63md\u{001b}[0m\u{001b}[38;5;63m9\u{001b}[0m\u{001b}[38;5;33m3\u{001b}[0m\u{001b}[38;5;33m5\u{001b}[0m\u{001b}[38;5;33m3\u{001b}[0m\u{001b}[38;5;39m1\u{001b}[0m\u{001b}[38;5;39m0\u{001b}[0m\u{001b}[38;5;39mb\u{001b}[0m\u{001b}[38;5;44m3\u{001b}[0m\u{001b}[38;5;44m4\u{001b}[0m\u{001b}[38;5;44m \u{001b}[0m\u{001b}[38;5;44m \u{001b}[0m\u{001b}[38;5;49m/\u{001b}[0m\u{001b}[38;5;49me\u{001b}[0m\u{001b}[38;5;49mt\u{001b}[0m\u{001b}[38;5;48mc\u{001b}[0m\u{001b}[38;5;48m/\u{001b}[0m\u{001b}[38;5;48mf\u{001b}[0m\u{001b}[38;5;84ms\u{001b}[0m\u{001b}[38;5;83mt\u{001b}[0m\u{001b}[38;5;83ma\u{001b}[0m\u{001b}[38;5;83mb\u{001b}[0m\u{001b}[38;5;119m.\u{001b}[0m\u{001b}[38;5;118mh\u{001b}[0m\u{001b}[38;5;118md\u{001b}[0m\r\r\n\u{001b}[38;5;208m4\u{001b}[0m\u{001b}[38;5;208m4\u{001b}[0m\u{001b}[38;5;208m2\u{001b}[0m\u{001b}[38;5;208ma\u{001b}[0m\u{001b}[38;5;203m5\u{001b}[0m\u{001b}[38;5;203mb\u{001b}[0m\u{001b}[38;5;203mc\u{001b}[0m\u{001b}[38;5;203m4\u{001b}[0m\u{001b}[38;5;198m1\u{001b}[0m\u{001b}[38;5;198m7\u{001b}[0m\u{001b}[38;5;198m4\u{001b}[0m\u{001b}[38;5;199ma\u{001b}[0m\u{001b}[38;5;199m8\u{001b}[0m\u{001b}[38;5;199mf\u{001b}[0m\u{001b}[38;5;164m4\u{001b}[0m\u{001b}[38;5;164md\u{001b}[0m\u{001b}[38;5;164m6\u{001b}[0m\u{001b}[38;5;164me\u{001b}[0m\u{001b}[38;5;129mf\u{001b}[0m\u{001b}[38;5;129m8\u{001b}[0m\u{001b}[38;5;129md\u{001b}[0m\u{001b}[38;5;93m5\u{001b}[0m\u{001b}[38;5;93ma\u{001b}[0m\u{001b}[38;5;93me\u{001b}[0m\u{001b}[38;5;93m5\u{001b}[0m\u{001b}[38;5;63md\u{001b}[0m\u{001b}[38;5;63ma\u{001b}[0m\u{001b}[38;5;63m9\u{001b}[0m\u{001b}[38;5;63m2\u{001b}[0m\u{001b}[38;5;33m5\u{001b}[0m\u{001b}[38;5;33m1\u{001b}[0m\u{001b}[38;5;33me\u{001b}[0m\u{001b}[38;5;39mb\u{001b}[0m\u{001b}[38;5;39mb\u{001b}[0m\u{001b}[38;5;39m6\u{001b}[0m\u{001b}[38;5;44ma\u{001b}[0m\u{001b}[38;5;44mb\u{001b}[0m\u{001b}[38;5;44m4\u{001b}[0m\u{001b}[38;5;44m5\u{001b}[0m\u{001b}[38;5;49m5\u{001b}[0m\u{001b}[38;5;49m \u{001b}[0m\u{001b}[38;5;49m \u{001b}[0m\u{001b}[38;5;48m/\u{001b}[0m\u{001b}[38;5;48me\u{001b}[0m\u{001b}[38;5;48mt\u{001b}[0m\u{001b}[38;5;84mc\u{001b}[0m\u{001b}[38;5;83m/\u{001b}[0m\u{001b}[38;5;83mf\u{001b}[0m\u{001b}[38;5;83mt\u{001b}[0m\u{001b}[38;5;119mp\u{001b}[0m\u{001b}[38;5;118md\u{001b}[0m\u{001b}[38;5;118m.\u{001b}[0m\u{001b}[38;5;118mc\u{001b}[0m\u{001b}[38;5;154mo\u{001b}[0m\u{001b}[38;5;154mn\u{001b}[0m\u{001b}[38;5;154mf\u{001b}[0m\r\r\n\u{001b}[38;5;208m4\u{001b}[0m\u{001b}[38;5;203m4\u{001b}[0m\u{001b}[38;5;203m2\u{001b}[0m\u{001b}[38;5;203ma\u{001b}[0m\u{001b}[38;5;203m5\u{001b}[0m\u{001b}[38;5;198mb\u{001b}[0m\u{001b}[38;5;198mc\u{001b}[0m\u{001b}[38;5;198m4\u{001b}[0m\u{001b}[38;5;199m1\u{001b}[0m\u{001b}[38;5;199m7\u{001b}[0m\u{001b}[38;5;199m4\u{001b}[0m\u{001b}[38;5;164ma\u{001b}[0m\u{001b}[38;5;164m8\u{001b}[0m\u{001b}[38;5;164mf\u{001b}[0m\u{001b}[38;5;164m4\u{001b}[0m\u{001b}[38;5;129md\u{001b}[0m\u{001b}[38;5;129m6\u{001b}[0m\u{001b}[38;5;129me\u{001b}[0m\u{001b}[38;5;93mf\u{001b}[0m\u{001b}[38;5;93m8\u{001b}[0m\u{001b}[38;5;93md\u{001b}[0m\u{001b}[38;5;93m5\u{001b}[0m\u{001b}[38;5;63ma\u{001b}[0m\u{001b}[38;5;63me\u{001b}[0m\u{001b}[38;5;63m5\u{001b}[0m\u{001b}[38;5;63md\u{001b}[0m\u{001b}[38;5;33ma\u{001b}[0m\u{001b}[38;5;33m9\u{001b}[0m\u{001b}[38;5;33m2\u{001b}[0m\u{001b}[38;5;39m5\u{001b}[0m\u{001b}[38;5;39m1\u{001b}[0m\u{001b}[38;5;39me\u{001b}[0m\u{001b}[38;5;44mb\u{001b}[0m\u{001b}[38;5;44mb\u{001b}[0m\u{001b}[38;5;44m6\u{001b}[0m\u{001b}[38;5;44ma\u{001b}[0m\u{001b}[38;5;49mb\u{001b}[0m\u{001b}[38;5;49m4\u{001b}[0m\u{001b}[38;5;49m5\u{001b}[0m\u{001b}[38;5;48m5\u{001b}[0m\u{001b}[38;5;48m \u{001b}[0m\u{001b}[38;5;48m \u{001b}[0m\u{001b}[38;5;84m/\u{001b}[0m\u{001b}[38;5;83me\u{001b}[0m\u{001b}[38;5;83mt\u{001b}[0m\u{001b}[38;5;83mc\u{001b}[0m\u{001b}[38;5;119m/\u{001b}[0m\u{001b}[38;5;118mf\u{001b}[0m\u{001b}[38;5;118mt\u{001b}[0m\u{001b}[38;5;118mp\u{001b}[0m\u{001b}[38;5;154md\u{001b}[0m\u{001b}[38;5;154m.\u{001b}[0m\u{001b}[38;5;154mc\u{001b}[0m\u{001b}[38;5;184mo\u{001b}[0m\u{001b}[38;5;184mn\u{001b}[0m\u{001b}[38;5;184mf\u{001b}[0m\u{001b}[38;5;184m.\u{001b}[0m\u{001b}[38;5;214md\u{001b}[0m\u{001b}[38;5;214me\u{001b}[0m\u{001b}[38;5;214mf\u{001b}[0m\u{001b}[38;5;208ma\u{001b}[0m\u{001b}[38;5;208mu\u{001b}[0m\u{001b}[38;5;208ml\u{001b}[0m\u{001b}[38;5;209mt\u{001b}[0m\r\r\n\u{001b}[38;5;203md\u{001b}[0m\u{001b}[38;5;203m3\u{001b}[0m\u{001b}[38;5;198me\u{001b}[0m\u{001b}[38;5;198m5\u{001b}[0m\u{001b}[38;5;198mf\u{001b}[0m\u{001b}[38;5;199mb\u{001b}[0m\u{001b}[38;5;199m0\u{001b}[0m\u{001b}[38;5;199mc\u{001b}[0m\u{001b}[38;5;164m5\u{001b}[0m\u{001b}[38;5;164m8\u{001b}[0m\u{001b}[38;5;164m2\u{001b}[0m\u{001b}[38;5;164m6\u{001b}[0m\u{001b}[38;5;129m4\u{001b}[0m\u{001b}[38;5;129m5\u{001b}[0m\u{001b}[38;5;129me\u{001b}[0m\u{001b}[38;5;93m6\u{001b}[0m\u{001b}[38;5;93m0\u{001b}[0m\u{001b}[38;5;93mf\u{001b}[0m\u{001b}[38;5;93m8\u{001b}[0m\u{001b}[38;5;63ma\u{001b}[0m\u{001b}[38;5;63m1\u{001b}[0m\u{001b}[38;5;63m3\u{001b}[0m\u{001b}[38;5;63m8\u{001b}[0m\u{001b}[38;5;33m0\u{001b}[0m\u{001b}[38;5;33m2\u{001b}[0m\u{001b}[38;5;33mb\u{001b}[0m\u{001b}[38;5;39me\u{001b}[0m\u{001b}[38;5;39m0\u{001b}[0m\u{001b}[38;5;39mc\u{001b}[0m\u{001b}[38;5;44m9\u{001b}[0m\u{001b}[38;5;44m0\u{001b}[0m\u{001b}[38;5;44m9\u{001b}[0m\u{001b}[38;5;44ma\u{001b}[0m\u{001b}[38;5;49m3\u{001b}[0m\u{001b}[38;5;49mf\u{001b}[0m\u{001b}[38;5;49m9\u{001b}[0m\u{001b}[38;5;48me\u{001b}[0m\u{001b}[38;5;48m4\u{001b}[0m\u{001b}[38;5;48md\u{001b}[0m\u{001b}[38;5;84m7\u{001b}[0m\u{001b}[38;5;83m \u{001b}[0m\u{001b}[38;5;83m \u{001b}[0m\u{001b}[38;5;83m/\u{001b}[0m\u{001b}[38;5;119me\u{001b}[0m\u{001b}[38;5;118mt\u{001b}[0m\u{001b}[38;5;118mc\u{001b}[0m\u{001b}[38;5;118m/\u{001b}[0m\u{001b}[38;5;154mf\u{001b}[0m\u{001b}[38;5;154mt\u{001b}[0m\u{001b}[38;5;154mp\u{001b}[0m\u{001b}[38;5;184mu\u{001b}[0m\u{001b}[38;5;184ms\u{001b}[0m\u{001b}[38;5;184me\u{001b}[0m\u{001b}[38;5;184mr\u{001b}[0m\u{001b}[38;5;214ms\u{001b}[0m\r\r\nbash-3.2$ # To finish recording just exit the shell\r\r\nbash-3.2$ exit\r\r\nexit\r\r\n\u{001b}[0;32masciinema: recording finished\u{001b}[0m\r\n\u{001b}[0;32masciinema: press <enter> to upload to asciinema.org, <ctrl-c> to save locally\u{001b}[0m\r\n\r\nhttps://asciinema.org/a/17648\r\n> # Open the above URL to view the recording\r\n> # Now install asciinema and start recording your own sessions\r\n> # Oh, and you can copy-paste from here\r\n> # Bye!\r\n";
    let mut vt = avt::Vt::builder()
        .size(cols, rows)
        .scrollback_limit(0)
        .build();

    vt.feed_str(raw);

    // TODO: Look up font in database...
    let font = include_bytes!("../font/ibm-plex-mono-latin-400-normal.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let font_size = 14.0;

    let size = (inst.width * inst.height) as usize;
    let advanced_width = font.metrics('0', font_size).width;
    let row_height = font_size * 1.0;

    let theme = Theme {
        background: RGB8::new(0x12, 0x13, 0x14),
        foreground: RGB8::new(0xcc, 0xcc, 0xcc),
        palette: [
            RGB8::new(0, 0, 0),
            RGB8::new(0xdd, 0x3c, 0x69),
            RGB8::new(0x4e, 0xbf, 0x22),
            RGB8::new(0xdd, 0xaf, 0x3c),
            RGB8::new(0x26, 0xb0, 0xd7),
            RGB8::new(0xb9, 0x54, 0xe1),
            RGB8::new(0x54, 0xe1, 0xb9),
            RGB8::new(0xd9, 0xd9, 0xd9),
            RGB8::new(0x4d, 0x4d, 0x4d),
            RGB8::new(0xdd, 0x3c, 0x69),
            RGB8::new(0x4e, 0xbf, 0x22),
            RGB8::new(0xdd, 0xaf, 0x3c),
            RGB8::new(0x26, 0xb0, 0xd7),
            RGB8::new(0xb9, 0x54, 0xe1),
            RGB8::new(0x54, 0xe1, 0xb9),
            RGB8::new(0xff, 0xff, 0xff),
        ],
    };

    let mut v: Vec<u32> = std::iter::repeat(rgb_to_u32(theme.background.into()))
        .take(size)
        .collect();

    for (row, line) in vt.view().iter().enumerate() {
        for (col, (char, pen)) in line.cells().enumerate() {
            let (metrics, bitmap) = font.rasterize(char, font_size);

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let pixel = bitmap[gy * metrics.width + gx];

                    let y = (row as f32 * row_height).round() as i32 + gy as i32 + font_size as i32
                        - metrics.height as i32
                        - metrics.ymin;

                    let x = (col * advanced_width) as i32 + gx as i32 + metrics.xmin;
                    let stride = inst.width as usize;

                    let bg = pen
                        .background()
                        .map(|x| color_to_rgb(x, &theme))
                        .unwrap_or(theme.background)
                        .alpha(255);

                    let fg = pen
                        .foreground()
                        .map(|x| color_to_rgb(x, &theme))
                        .unwrap_or(theme.foreground)
                        .alpha(255);

                    v[y as usize * stride + x as usize] = rgb_to_u32(blend(fg, bg, pixel));
                }
            }
        }
    }

    unsafe { std::ptr::copy_nonoverlapping(v.as_ptr(), output, size) }
}

fn rgb_to_u32(color: RGBA8) -> u32 {
    ((color.r as u32) << 0)
        | ((color.g as u32) << 8)
        | ((color.b as u32) << 16)
        | ((color.a as u32) << 24)
}

fn color_to_rgb(color: avt::Color, theme: &Theme) -> RGB8 {
    match color {
        avt::Color::Indexed(x) => theme.color(x),
        avt::Color::RGB(c) => c,
    }
}

impl Theme {
    pub fn color(&self, color: u8) -> RGB8 {
        match color {
            0..=15 => self.palette[color as usize],

            16..=231 => {
                let n = color - 16;
                let mut r = ((n / 36) % 6) * 40;
                let mut g = ((n / 6) % 6) * 40;
                let mut b = (n % 6) * 40;

                if r > 0 {
                    r += 55;
                }

                if g > 0 {
                    g += 55;
                }

                if b > 0 {
                    b += 55;
                }

                RGB8::new(r, g, b)
            }

            232.. => {
                let v = 8 + 10 * (color - 232);

                RGB8::new(v, v, v)
            }
        }
    }
}

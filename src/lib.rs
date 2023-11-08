extern crate libc;

use rgb::{RGB8, RGBA8};
use serde::Deserialize;
use std::ffi::{CStr, CString};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Event {
    Output { time: f64, data: String },
    Unknown { time: f64, tag: String },
}

pub struct Vt {
    width: u32,
    height: u32,
    path: CString,
    frames: Vec<Frame>,
}

struct Theme {
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
        frames: vec![],
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

            // TODO: Handle error
            let file = parse_file(inst.path.to_str().unwrap()).unwrap();

            let mut vt = avt::Vt::builder()
                .size(file.header.width, file.header.height)
                .scrollback_limit(0)
                .build();

            inst.frames = file
                .events
                .iter()
                .filter_map(|event| match event {
                    Event::Output { time, data } => {
                        let (changed, _) = vt.feed_str(&data);

                        if changed.is_empty() {
                            None
                        } else {
                            let data = vt
                                .lines()
                                .iter()
                                .map(|line| line.cells().collect())
                                .collect();

                            Some(Frame { time: *time, data })
                        }
                    }
                    _ => None,
                })
                .collect();
        },
        _ => {}
    }
}

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidEventTime,
    InvalidEvent,
    EmptyFile,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[derive(Deserialize)]
struct Header {
    width: usize,
    height: usize,
}

struct Screencast {
    header: Header,
    events: Vec<Event>,
}

fn parse_file(path: &str) -> Result<Screencast, Error> {
    let file = std::fs::read_to_string(path)?;

    let mut lines = file.lines();
    let first_line = lines.next().ok_or(Error::EmptyFile)?;

    let header: Header = serde_json::from_str(first_line)?;
    let events: Result<Vec<Event>, _> = lines.map(|line| parse_event(line)).collect();

    Ok(Screencast {
        header,
        events: events?,
    })
}

fn parse_event(line: &str) -> Result<Event, Error> {
    let value: serde_json::Value = serde_json::from_str(line)?;
    let time = value[0].as_f64().ok_or(Error::InvalidEventTime)?;

    match value[1].as_str() {
        Some("o") => {
            let data = value[2].as_str().ok_or(Error::InvalidEvent)?;
            Ok(Event::Output {
                time,
                data: data.to_owned(),
            })
        }
        Some(tag) => Ok(Event::Unknown {
            time,
            tag: tag.to_owned(),
        }),
        None => Err(Error::InvalidEvent),
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

struct Frame {
    time: f64,
    data: Vec<Vec<(char, avt::Pen)>>,
}

struct Dimensions {
    row_height: f32,
    col_width: usize,
}

impl Dimensions {
    fn x(&self, col: usize) -> usize {
        col * self.col_width
    }

    fn y(&self, row: usize) -> usize {
        (row as f32 * self.row_height).round() as usize
    }
}

#[no_mangle]
pub extern "C" fn f0r_update(
    inst: *mut Vt,
    time: libc::c_double,
    _input: *mut u32,
    output: *mut u32,
) {
    let inst = unsafe { &*inst };

    // TODO: Glyph caching
    // TODO: Better font (extra chars)
    let font = include_bytes!("../font/ibm-plex-mono-latin-400-normal.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let font_size = 16.0; // TODO: Param

    let size = (inst.width * inst.height) as usize;
    let col_width = font.metrics('0', font_size).width;
    let row_height = font_size * 1.3;

    let dim = Dimensions {
        row_height,
        col_width,
    };

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

    let first_frame = Frame {
        time: 0.0,
        data: vec![],
    };

    let frame = inst
        .frames
        .iter()
        .rfind(|frame| frame.time <= time)
        .unwrap_or(&first_frame);

    for (row, line) in frame.data.iter().enumerate() {
        for (col, (char, pen)) in line.iter().enumerate() {
            let (metrics, bitmap) = font.rasterize(*char, font_size);

            // Draw the background over the whole cell
            if let Some(c) = pen.background() {
                for y in dim.y(row)..dim.y(row + 1) {
                    for x in dim.x(col)..dim.x(col + 1) {
                        let stride = inst.width as usize;

                        if (x as u32) < inst.width && (y as u32) < inst.height {
                            v[y as usize * stride + x as usize] =
                                rgb_to_u32(color_to_rgb(c, &theme).into());
                        }
                    }
                }
            }

            if char == &' ' {
                continue;
            }

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let pixel = bitmap[gy * metrics.width + gx];

                    let y = (row as f32 * row_height).round() as i32 + gy as i32 + font_size as i32
                        - metrics.height as i32
                        - metrics.ymin;

                    let x = (col * col_width) as i32 + gx as i32 + metrics.xmin;
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

                    if (x as u32) < inst.width && (y as u32) < inst.height {
                        v[y as usize * stride + x as usize] = rgb_to_u32(blend(fg, bg, pixel));
                    }
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

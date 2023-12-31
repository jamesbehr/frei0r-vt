extern crate libc;

use rgb::{RGB8, RGBA8};
use serde::Deserialize;
use std::ffi::{CStr, CString};

const F0R_PLUGIN_TYPE_SOURCE: i32 = 1;
const F0R_COLOR_MODEL_RGBA8888: i32 = 1;
const FREI0R_MAJOR_VERSION: i32 = 1;
const F0R_PARAM_BOOL: i32 = 0;
const F0R_PARAM_COLOR: i32 = 2;
const F0R_PARAM_STRING: i32 = 4;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
enum Event {
    Output { time: f64, data: String },
    Marker { time: f64, data: String },
    Unknown { time: f64, tag: String },
}

impl Event {
    fn is_marker(&self) -> bool {
        match self {
            Self::Marker { .. } => true,
            _ => false,
        }
    }
}

#[derive(Deserialize, Debug)]
struct Cut {
    // Marker to start the cut from. Play from the beginning of the file if omitted.
    first_marker: Option<usize>,

    // Marker to stop the cut at. Play to end of file if ommitted.
    last_marker: Option<usize>,

    // If true, the timing of all events after the specified marker is shifted such that there is
    // no delay between starting playback and the first event.
    start_immediately: bool,
}

pub struct Vt {
    width: u32,
    height: u32,
    path: CString,
    cut: Option<Cut>,
    groups: Vec<FrameGroup>,
    font: fontdue::Font,
}

#[derive(Debug)]
struct FrameGroup {
    time: f64,
    frames: Vec<Frame>,
}

impl Vt {
    fn load(&mut self, file: Screencast) {
        let mut vt = avt::Vt::builder()
            .size(file.header.width, file.header.height)
            .scrollback_limit(0)
            .build();

        self.groups = vec![];

        let mut fg = FrameGroup {
            time: 0.0,
            frames: vec![],
        };

        for event in file.events {
            match event {
                Event::Output { time, data } => {
                    let (changed, _) = vt.feed_str(&data);

                    if !changed.is_empty() {
                        let data = vt
                            .lines()
                            .iter()
                            .map(|line| line.cells().collect())
                            .collect();

                        fg.frames.push(Frame { time, data })
                    }
                }
                Event::Marker { time, .. } => {
                    self.groups.push(fg);
                    fg = FrameGroup {
                        time,
                        frames: vec![],
                    }
                }
                _ => (),
            }
        }

        if !fg.frames.is_empty() {
            self.groups.push(fg);
        }
    }

    fn cut(&self) -> (Option<usize>, Option<usize>) {
        match &self.cut {
            // TODO: Check first_marker < last_marker
            Some(cut) => (cut.first_marker, cut.last_marker),
            None => (None, None),
        }
    }

    fn frame(&self, time: f64) -> Frame {
        // if self.groups.len() == 0 {
        //     return Frame::new();
        // }

        let groups = match self.cut() {
            (Some(start), Some(end)) => &self.groups[start + 1..end + 1],
            (Some(start), None) => &self.groups[start + 1..],
            (None, Some(end)) => &self.groups[..end + 1],
            (None, None) => &self.groups[..],
        };

        let subtract = if let Some(Cut {
            start_immediately, ..
        }) = self.cut
        {
            if start_immediately {
                groups
                    .first()
                    .and_then(|group| group.frames.first())
                    .map(|frame| frame.time)
                    .unwrap_or(0.0)
            } else {
                groups.first().map(|group| group.time).unwrap_or(0.0)
            }
        } else {
            0.0
        };

        groups
            .iter()
            .rfind(|group| group.time - subtract < time)
            .and_then(|group| {
                group
                    .frames
                    .iter()
                    .rfind(|frame| frame.time - subtract <= time)
            })
            .unwrap_or(&Frame::new())
            .clone()
    }
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
    (*info).plugin_type = F0R_PLUGIN_TYPE_SOURCE;
    (*info).color_model = F0R_COLOR_MODEL_RGBA8888;
    (*info).frei0r_version = FREI0R_MAJOR_VERSION;
    (*info).major_version = 0;
    (*info).minor_version = 1;
    (*info).num_params = 2;
    (*info).explanation = b"Generates a VT terminal screencast\0".as_ptr() as *const libc::c_char;
}

#[no_mangle]
pub extern "C" fn f0r_get_param_info(info: *mut F0rParamInfo, index: libc::c_int) {
    match index {
        0 => unsafe {
            (*info).name = b"resource\0".as_ptr() as *const libc::c_char;
            (*info).param_type = F0R_PARAM_STRING;
            (*info).explanation = b"Path to asciicast file\0".as_ptr() as *const libc::c_char;
        },
        1 => unsafe {
            (*info).name = b"cut\0".as_ptr() as *const libc::c_char;
            (*info).param_type = F0R_PARAM_STRING;
            (*info).explanation =
                b"JSON formatted string describing cutting\0".as_ptr() as *const libc::c_char;
        },
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn f0r_construct(width: u32, height: u32) -> *mut Vt {
    let families = vec![
        "Cascadia Code",
        "JetBrains Mono",
        "Fira Code",
        "SF Mono",
        "Menlo",
        "Consolas",
        "DejaVu Sans Mono",
        "Liberation Mono",
        "DejaVu Sans",
        "Noto Emoji",
    ];

    let mut fonts = fontdb::Database::new();
    fonts.load_system_fonts();

    let families: Vec<fontdb::Family> = families
        .iter()
        .map(|name| fontdb::Family::Name(name.as_ref()))
        .collect();

    let query = fontdb::Query {
        families: &families,
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    };

    // TODO: Handle error
    let font_id = fonts.query(&query).unwrap();
    let font = fonts
        .with_face_data(font_id, |font_data, face_index| {
            let settings = fontdue::FontSettings {
                collection_index: face_index,
                ..Default::default()
            };

            fontdue::Font::from_bytes(font_data, settings).unwrap()
        })
        .unwrap();

    let path = CString::new("").unwrap();
    let inst = Box::new(Vt {
        width,
        height,
        path,
        cut: None,
        groups: vec![],
        font,
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
        0 => {
            let path = unsafe {
                let p = param as *const *const libc::c_char;
                CStr::from_ptr(*p).to_str().unwrap()
            };

            // TODO: Only load if path changed
            println!("{}", path);
            if path != "<producer>" {
                let file = parse_file(path).unwrap();
                inst.load(file);
            }
        }
        1 => unsafe {
            let p = param as *const *const libc::c_char;
            let cut: Cut = serde_json::from_str(CStr::from_ptr(*p).to_str().unwrap()).unwrap();
            inst.cut = Some(cut);
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
        Some("m") => {
            let data = value[2].as_str().ok_or(Error::InvalidEvent)?;
            Ok(Event::Marker {
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

#[derive(Clone, Debug)]
struct Frame {
    time: f64,
    data: Vec<Vec<(char, avt::Pen)>>,
}

impl Frame {
    fn new() -> Frame {
        Frame {
            time: 0.0,
            data: vec![],
        }
    }
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

    let font_size = 16.0; // TODO: Param

    let size = (inst.width * inst.height) as usize;
    let col_width = inst.font.metrics('0', font_size).width;
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

    for (row, line) in inst.frame(time).data.iter().enumerate() {
        for (col, (char, pen)) in line.iter().enumerate() {
            let (metrics, bitmap) = inst.font.rasterize(*char, font_size);

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

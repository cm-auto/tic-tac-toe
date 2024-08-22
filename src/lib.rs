#![cfg_attr(not(test), no_std)]

macro_rules! static_variable {
    ($name:ident, $static_name:ident: $ty:ty = $value:expr) => {
        static mut $static_name: $ty = $value;
        fn $name() -> &'static mut $ty {
            let stringifed_static_name = stringify!($static_name);
            unsafe {
                match core::ptr::addr_of_mut!($static_name).as_mut() {
                    Some(x) => x,
                    None => unreachable!("Address of {stringifed_static_name} is null"),
                }
            }
        }
    };
}

#[cfg(debug_assertions)]
fn print_panic_location(location: &core::panic::Location) {
    drawing::print_panic_location(
        location.file(),
        location.line() as f64,
        location.column() as f64,
    );
}

#[cfg(not(debug_assertions))]
#[inline(always)]
fn print_panic_location(_location: &core::panic::Location) {}

#[cfg_attr(not(test), panic_handler)]
// this is not really dead code, since it is used as the panic handler
#[allow(dead_code)]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    // using this code makes the binary about 1300 bytes bigger
    // which is why print_panic_location is only implemented in debug builds
    // and a noop in release builds (the user won't see the console anyway)
    let location_option = _info.location();
    if let Some(location) = location_option {
        // unsafe {
        //     // this works but needs an additional 1800 bytes!
        //     static_variable!(panic_string_buffer, PANIC_STRING_BUFFER: [u8; 1024] = [0; 1024]);
        //     let mut panic_string_buffer = PANIC_STRING_BUFFER;
        //     let mut length = 0;
        //     let ptr = panic_string_buffer.as_mut_ptr();
        //     let message = b"Got panic in file:\n";
        //     ptr.copy_from(message.as_ptr(), message.len());
        //     let ptr = ptr.add(message.len());
        //     length += message.len();
        //     let file_path = location.file();
        //     ptr.copy_from(file_path.as_ptr(), file_path.len());
        //     length += file_path.len();
        //     // let ptr = ptr.add(file_path.len());
        //     let slice = core::slice::from_raw_parts(panic_string_buffer.as_ptr(), length);
        //     let panic_text = str::from_utf8(slice).unwrap_unchecked();
        //     drawing::print(panic_text);
        // }

        print_panic_location(location);
    }
    draw_message_dialog("Error");
    drawing::handle_panic();
}

#[repr(C)]
struct Vector2 {
    x: f64,
    y: f64,
}

macro_rules! safe_extern_function {
        ($module:expr, $name:ident, $unsafe_name:ident, ($($arg:ident: $ty:ty),*) $(-> $ret:ty)?) => {
            #[link(wasm_import_module = $module)]
            extern "C" {
                #[link_name = stringify!($name)]
                fn $unsafe_name($($arg: $ty),*)$( -> $ret)?;
            }
            pub fn $name($($arg: $ty),*)$( -> $ret)? {
                unsafe { $unsafe_name($($arg),*) }
            }
        };
    }

mod drawing {

    macro_rules! safe_extern_drawing_function {
        ($name:ident, $unsafe_name:ident, ($($arg:ident: $ty:ty),*) $(-> $ret:ty)?) => {
            safe_extern_function!("drawing", $name, $unsafe_name, ($($arg: $ty),*) $(-> $ret)?);
        };
    }

    #[allow(dead_code)]
    #[repr(C)]
    pub enum LineJoin {
        Miter,
        Round,
        Bevel,
    }

    use crate::*;

    safe_extern_drawing_function!(draw_line, _draw_line, (x1: f64, y1: f64, x2: f64, y2: f64));

    // this is not relative to canvas size
    // you need to multiply it with the canvas size
    safe_extern_drawing_function!(set_stroke_thickness, _set_stroke_thickness, (thickness: f64));

    safe_extern_drawing_function!(fill_rect, _fill_rect, (x: f64, y: f64, w: f64, h: f64));
    safe_extern_drawing_function!(stroke_rect, _stroke_rect, (x: f64, y: f64, w: f64, h: f64));

    safe_extern_drawing_function!(set_line_join, _set_line_join, (join: LineJoin));

    safe_extern_drawing_function!(handle_panic, _handle_panic, () -> !);

    #[link(wasm_import_module = "drawing")]
    extern "C" {
        #[link_name = "draw_ellipse"]
        fn _draw_ellipse(
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            rotation: f64,
            start_angle: f64,
            end_angle: f64,
            counterclockwise: bool,
        );
        #[link_name = "canvas_width"]
        fn _canvas_width() -> f64;
        #[link_name = "canvas_height"]
        fn _canvas_height() -> f64;
        #[link_name = "set_stroke_color"]
        fn _set_stroke_color(r: u8, g: u8, b: u8, a: u8);

        #[link_name = "set_fill_color"]
        fn _set_fill_color(r: u8, g: u8, b: u8, a: u8);

        #[link_name = "set_font"]
        fn _set_font(pixel_size: f64, text: *const u8, len: usize);
        #[link_name = "fill_text"]
        fn _fill_text(text: *const u8, len: usize, x: f64, y: f64, max_width: f64);

        #[link_name = "print"]
        fn _print(s: *const u8, len: usize);
        #[link_name = "print_number"]
        fn _print_number(num: f64);
        #[link_name = "print_panic_location"]
        fn _print_panic_location(s: *const u8, len: usize, line: f64, column: f64);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_ellipse(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        counterclockwise: bool,
    ) {
        unsafe {
            _draw_ellipse(
                x,
                y,
                w,
                h,
                rotation,
                start_angle,
                end_angle,
                counterclockwise,
            )
        };
    }
    pub fn draw_full_ellipse(x: f64, y: f64, w: f64, h: f64) {
        draw_ellipse(x, y, w, h, 0.0, 0.0, 2.0 * core::f64::consts::PI, false)
    }
    pub fn draw_circle(x: f64, y: f64, r: f64) {
        draw_full_ellipse(x, y, r, r)
    }
    pub fn canvas_size() -> Vector2 {
        unsafe {
            Vector2 {
                x: _canvas_width(),
                y: _canvas_height(),
            }
        }
    }
    pub fn set_stroke_color(color: impl Into<Color>) {
        let (r, g, b, a) = color.into().to_bytes();
        unsafe { _set_stroke_color(r, g, b, a) };
    }

    pub fn set_fill_color(color: impl Into<Color>) {
        let (r, g, b, a) = color.into().to_bytes();
        unsafe { _set_fill_color(r, g, b, a) };
    }

    pub fn set_font(pixel_size: f64, font: &str) {
        unsafe { _set_font(pixel_size, font.as_ptr(), font.len()) }
    }
    #[allow(dead_code)]
    pub fn fill_text(text: &str, x: f64, y: f64) {
        const NO_LIMIT: f64 = -1.0;
        fill_text_with_max_width(text, x, y, NO_LIMIT);
    }
    pub fn fill_text_with_max_width(text: &str, x: f64, y: f64, max_width: f64) {
        unsafe {
            _fill_text(text.as_ptr(), text.len(), x, y, max_width);
        }
    }

    #[allow(dead_code)]
    pub fn print(text: &str) {
        unsafe { _print(text.as_ptr(), text.len()) };
    }
    #[allow(dead_code)]
    pub fn print_number(num: f64) {
        unsafe { _print_number(num) };
    }
    // in release builds, this function will not be called
    #[allow(dead_code)]
    pub fn print_panic_location(text: &str, line: f64, column: f64) {
        unsafe { _print_panic_location(text.as_ptr(), text.len(), line, column) };
    }
}

struct Color(u64);

impl Color {
    fn to_bytes(&self) -> (u8, u8, u8, u8) {
        let r = ((self.0 >> 24) & 0xFF) as u8;
        let g = ((self.0 >> 16) & 0xFF) as u8;
        let b = ((self.0 >> 8) & 0xFF) as u8;
        let a = (self.0 & 0xFF) as u8;
        (r, g, b, a)
    }
}

// Interesting fact: I thought I could save some bytes in the wasm file
// by removing this function and just explicitly passing the alpha value,
// but this resulted in even more bytes in the wasm file.
// I guess that's because every time the following function is called, the alpha value
// has to be saved in the wasm file.
// However, this function always has 255 as alpha and thus we need it just once.
impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color((r as u64) << 24 | (g as u64) << 16 | (b as u64) << 8 | 255_u64)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Color((r as u64) << 24 | (g as u64) << 16 | (b as u64) << 8 | (a as u64))
    }
}

impl From<u64> for Color {
    fn from(value: u64) -> Self {
        Color(value)
    }
}

#[derive(Copy, Clone, PartialEq)]
enum CellState {
    X,
    O,
}

impl CellState {
    fn switch(&mut self) {
        *self = match self {
            CellState::X => CellState::O,
            CellState::O => CellState::X,
        };
    }

    fn get_color(&self) -> Color {
        match self {
            CellState::X => (0, 0, 0).into(),
            CellState::O => (255, 0, 0).into(),
        }
    }
}

struct GameState {
    cells: [Option<CellState>; 9],
    current_player: CellState,
}

impl GameState {
    fn reset(&mut self) {
        self.cells = [None; 9];
        self.current_player = CellState::X;
    }

    fn get_winner(&self) -> Option<(CellState, usize, usize)> {
        for &(a, b, c) in &[
            (0, 1, 2),
            (3, 4, 5),
            (6, 7, 8),
            (0, 3, 6),
            (1, 4, 7),
            (2, 5, 8),
            (0, 4, 8),
            (2, 4, 6),
        ] {
            if let Some(state) = self.cells[a] {
                if self.cells[a] == self.cells[b] && self.cells[a] == self.cells[c] {
                    return Some((state, a, c));
                }
            }
        }
        None
    }

    fn is_draw(&self) -> bool {
        self.cells.iter().all(|&cell| cell.is_some())
    }
}

static_variable!(state, STATE: GameState = GameState {
    cells: [None; 9],
    current_player: CellState::X
});

fn draw_grid() {
    use drawing::*;
    let size = canvas_size();

    // canvas width is the smaller of the sides
    let playing_field_size = size.x;
    let cell_size = playing_field_size / 3.0;

    for x in 1..3 {
        let x_start = x as f64 * cell_size;
        let y_start = size.y - playing_field_size;

        // vertical line
        draw_line(x_start, y_start, x_start, size.y);

        // horizontal line
        draw_line(0.0, x_start + y_start, size.x, x_start + y_start);
    }
}

static_variable!(last_click, LAST_CLICK: Option<Vector2> = None);

#[no_mangle]
pub extern "C" fn handle_click(x: f64, y: f64) {
    last_click().replace(Vector2 { x, y });
    if state().get_winner().is_some() || state().is_draw() {
        state().reset();
        return;
    }
    if y < playing_field_start_y() {
        return;
    }
    let x = (x / playing_field_size() * 3.0) as usize;
    let y = ((y - playing_field_start_y()) / playing_field_size() * 3.0) as usize;
    let i = y * 3 + x;
    let cell = &mut state().cells[i];
    let current_player = &mut state().current_player;
    if cell.is_none() {
        *cell = Some(*current_player);
        current_player.switch();
    }
}

static_variable!(size, SIZE: Vector2 = Vector2 { x: 0.0, y: 0.0 });
fn playing_field_size() -> f64 {
    // since the canvas is always vertical, the width is the smaller of the dimensions
    size().x
}
fn playing_field_start_y() -> f64 {
    size().y - playing_field_size()
}
#[no_mangle]
pub extern "C" fn set_size(width: f64, height: f64) {
    if let Some(Vector2 { x, y }) = last_click() {
        *x = scale(*x, 0.0, size().x, 0.0, width);
        *y = scale(*y, 0.0, size().y, 0.0, height);
    }
    size().x = width;
    size().y = height;
}

fn scale(num: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    (num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

fn relative_modifier() -> f64 {
    playing_field_size() / 600.0
}

fn cell_padding() -> f64 {
    playing_field_size() / 3.0 * 0.05
}

fn cell_stroke_thickness() -> f64 {
    10.0 * relative_modifier()
}

fn draw_current_player() {
    use drawing::*;
    let playing_field_size = playing_field_size();
    let current_player = state().current_player;
    let playing_field_third = playing_field_size / 3.0;
    let x_top_left = playing_field_third;
    let y_top_left = 0.0;
    let cell_padding = cell_padding();
    set_stroke_thickness(cell_stroke_thickness());
    match current_player {
        CellState::X => {
            set_stroke_color(0x000000ff);
            // from top left to bottom right
            draw_line(
                x_top_left + cell_padding,
                y_top_left + cell_padding,
                x_top_left + playing_field_third - cell_padding,
                playing_field_third - cell_padding,
            );
            // from top right to bottom left
            draw_line(
                x_top_left + playing_field_third - cell_padding,
                y_top_left + cell_padding,
                x_top_left + cell_padding,
                playing_field_third - cell_padding,
            );
        }
        CellState::O => {
            set_stroke_color(0xff0000ff);
            draw_circle(
                x_top_left + playing_field_third / 2.0,
                y_top_left + playing_field_third / 2.0,
                playing_field_third / 2.0 - cell_padding,
            );
        }
    };
}

fn draw_message_dialog(message: &str) {
    use drawing::*;

    let relative_modifier = relative_modifier();
    let playing_field_size = playing_field_size();
    let horizontal_offset = 50.0;
    let font_height = 80.0 * relative_modifier;

    let w = playing_field_size - horizontal_offset * 2.0 * relative_modifier;
    // has to be a little bigger than the font height,
    // because the bottom line of the text would otherwise
    // be exactly on the bottom of the rectangle
    let h = font_height * 1.5;
    let x = (0.0 + horizontal_offset) * relative_modifier;
    let y = size().y - playing_field_size + playing_field_size / 2.0 - h / 2.0;
    set_fill_color((255, 255, 255, 255 / 4 * 3));
    fill_rect(x, y, w, h);
    set_stroke_thickness(2.0 * relative_modifier);
    set_stroke_color((0, 0, 0));
    stroke_rect(x, y, w, h);

    let x = x + 10.0 * relative_modifier;
    let w = w - 20.0 * relative_modifier;
    set_font(80.0 * relative_modifier, "monospace");
    set_fill_color(0x000000ff);
    fill_text_with_max_width(message, x, y + font_height, w);
}

fn get_top_left_corner_of_cell(index: usize) -> (f64, f64) {
    let x = index % 3;
    let y = index / 3;
    let cell_size = playing_field_size() / 3.0;
    (
        x as f64 * cell_size,
        playing_field_start_y() + y as f64 * cell_size,
    )
}

fn get_center_of_cell(index: usize) -> (f64, f64) {
    let (x, y) = get_top_left_corner_of_cell(index);
    let cell_size = playing_field_size() / 3.0;
    (x + cell_size / 2.0, y + cell_size / 2.0)
}

fn draw_winner_line(winner: CellState, start: usize, end: usize) {
    use drawing::*;
    let color = winner.get_color();
    let (start_x, start_y) = get_center_of_cell(start);
    let (end_x, end_y) = get_center_of_cell(end);
    set_stroke_color(color);
    set_stroke_thickness(cell_stroke_thickness());
    draw_line(start_x, start_y, end_x, end_y);
}

#[no_mangle]
pub extern "C" fn init() {
    use drawing::*;
    let size = canvas_size();
    set_size(size.x, size.y);
    set_line_join(LineJoin::Round);
}

#[no_mangle]
pub extern "C" fn draw(_delta: f64) {
    use drawing::*;
    let size = size();

    let playing_field_size = size.x;
    let relative_modifier = playing_field_size / 600.0;

    draw_current_player();

    set_stroke_thickness(3.0 * relative_modifier);
    set_stroke_color(0x00000040);
    draw_grid();

    set_stroke_thickness(cell_stroke_thickness());

    let canvas_width_third = size.x / 3.0;
    let cell_padding = playing_field_size / 3.0 * 0.05;

    let state = state();
    for (i, cell) in state.cells.iter().enumerate() {
        let x = (i % 3) as f64;
        let y = (i / 3) as f64;
        let x_top_left = x * canvas_width_third + cell_padding;
        let y_top_left = y * canvas_width_third + size.y - playing_field_size + cell_padding;
        if matches!(*cell, Some(CellState::X)) {
            set_stroke_color((0, 0, 0));
            // from top left to top right
            draw_line(
                x_top_left,
                y_top_left,
                x_top_left + canvas_width_third - cell_padding * 2.0,
                y_top_left + canvas_width_third - cell_padding * 2.0,
            );
            // from top right to bottom left
            draw_line(
                x_top_left + canvas_width_third - cell_padding * 2.0,
                y_top_left,
                x_top_left,
                y_top_left + canvas_width_third - cell_padding * 2.0,
            );
        } else if matches!(*cell, Some(CellState::O)) {
            set_stroke_color((255, 0, 0));
            draw_circle(
                x_top_left + canvas_width_third / 2.0 - cell_padding,
                y_top_left + canvas_width_third / 2.0 - cell_padding,
                canvas_width_third / 2.0 - cell_padding,
            );
        }
    }

    if let Some((winner, start, end)) = state.get_winner() {
        draw_winner_line(winner, start, end);
        let message = match winner {
            CellState::X => "X has won",
            CellState::O => "O has won",
        };
        draw_message_dialog(message);
    } else if state.is_draw() {
        draw_message_dialog("It's a draw");
    }

    // if debug mode is on, show the last click
    if !cfg!(debug_assertions) {
        return;
    }
    if let Some(Vector2 { x, y }) = last_click() {
        set_stroke_color((0, 255, 0));
        draw_circle(*x, *y, 10.0 * relative_modifier);
    }
}

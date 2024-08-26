"use strict"

const canvas = document.querySelector("#game")
/** @type {CanvasRenderingContext2D} */
const ctx = canvas.getContext("2d")

function draw_line(x1, y1, x2, y2) {
    ctx.beginPath()
    ctx.moveTo(x1, y1)
    ctx.lineTo(x2, y2)
    ctx.closePath()
    ctx.stroke()
}

function draw_ellipse(x, y, w, h, rotation, startAngle, endAngle, counterclockwise) {
    ctx.beginPath()
    ctx.ellipse(x, y, w, h, rotation, startAngle, endAngle, counterclockwise)
    ctx.closePath()
    ctx.stroke()
}

// this function can't be used like that, because wasm only supports
// simple numeric parameters, so a simple struct can't be returned
// alternative: request_canvas_size() -> call rust function with multiple params
// other alternative: pass a pointer to an array, as argument
// function canvas_size() {
//     // const array = new Float64Array(2)
//     // array[0] = canvas.width
//     // array[1] = canvas.height
//     // return array
//     // return [canvas.width, canvas.height]
// }

function canvas_width() {
    return canvas.width
}

function canvas_height() {
    return canvas.height
}

function clear_canvas() {
    ctx.clearRect(0, 0, canvas.width, canvas.height)
}

function set_stroke_color(r, g, b, a) {
    ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${a / 255})`
}

function set_stroke_thickness(thickness) {
    ctx.lineWidth = thickness
}

function set_fill_color(r, g, b, a) {
    ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${a / 255})`
}

function fill_rect(x, y, w, h) {
    ctx.fillRect(x, y, w, h)
}

function stroke_rect(x, y, w, h) {
    ctx.strokeRect(x, y, w, h)
}

/**
 * 
 * @returns {ArrayBuffer}
 */
function get_buffer() {
    return source.instance.exports.memory.buffer
}
const decoder = new TextDecoder("utf-8")
function get_text(str_ptr, str_len) {
    const arrayBuffer = get_buffer().slice(str_ptr, str_ptr + str_len)
    // this copies the string, which is desired
    return decoder.decode(arrayBuffer)
}

function set_font(pixel_size, str_ptr, str_len) {
    ctx.font = `${pixel_size}px ${get_text(str_ptr, str_len)}`
}
function fill_text(str_ptr, str_len, x, y, max_width) {
    // TODO: could be useful for text wrapping
    // const metrics = ctx.measureText(get_text(str_ptr, str_len))
    const no_limit = -1
    if (max_width === no_limit) {
        max_width = undefined
    }
    ctx.fillText(get_text(str_ptr, str_len), x, y, max_width)
}

function set_line_join(line_join) {
    const map = ["miter", "round", "bevel"]
    ctx.lineJoin = map[line_join]
}

// localStorge only allows strings, so we need to convert the Uint8Array to a string
// chunks are required because calling String.fromCharCode
// on a large Uint8Array might cause exceeding the maximum stack size
function uint8_array_to_string(uint8_array) {
    const chunk_size = 1024
    const chunks = []
    for (let i = 0; i < uint8_array.length; i += chunk_size) {
        chunks.push(String.fromCharCode(...uint8_array.subarray(i, i + chunk_size)))
    }
    return chunks.join("")
}

function save_bytes(key_ptr, key_len, value_ptr, value_len) {
    try {
        const key = get_text(key_ptr, key_len)
        const value = get_buffer().slice(value_ptr, value_ptr + value_len)
        const uint8_array = new Uint8Array(value)
        const byte_string = uint8_array_to_string(uint8_array)
        localStorage.setItem(key, byte_string)
        return true
    } catch {
        return false
    }
}

function load_bytes(key_ptr, key_len, value_ptr, value_buffer_len, has_value_ptr) {
    const key = get_text(key_ptr, key_len)
    const byte_string = localStorage.getItem(key)
    const view = new DataView(get_buffer())
    if (!byte_string) {
        view.setUint8(has_value_ptr, 0)
        return
    }
    const uint8_array = new Uint8Array(byte_string.length)
    for (let i = 0; i < byte_string.length; i++) {
        uint8_array[i] = byte_string.charCodeAt(i)
    }
    for (let i = 0; i < value_buffer_len; i++) {
        view.setUint8(value_ptr + i, uint8_array[i])
    }
    view.setUint8(has_value_ptr, 1)
}

// useful for debugging
function print(str_ptr, str_len) {
    const array_buffer = get_buffer().slice(str_ptr, str_ptr + str_len)
    const str = decoder.decode(array_buffer)
    console.log(str)
}

function print_number(num) {
    console.log(num)
}

function print_panic_location(file_path_ptr, file_path_len, line, column) {
    console.error(`Panic in file:\n${get_text(file_path_ptr, file_path_len)}\nLine:${line}\nColumn:${column}`)
}

let has_panicked = false
function handle_panic() {
    has_panicked = true
    console.error("There was a panic in the wasm code. Stopping game loop.")
}

const source = await WebAssembly.instantiateStreaming(fetch("game.wasm"), {
    drawing: {
        draw_line,
        draw_ellipse,
        // canvas_size,
        canvas_width,
        canvas_height,
        set_stroke_color,
        set_stroke_thickness,
        set_fill_color,
        fill_rect,
        stroke_rect,
        set_font,
        fill_text,
        set_line_join,
        save_bytes,
        load_bytes,
        print,
        print_number,
        print_panic_location,
        handle_panic,
    }
})

function scale(num, in_min, in_max, out_min, out_max) {
    return (num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

canvas.addEventListener("click", e => {
    const { x, y, target } = e
    const relative_x = x - target.offsetLeft
    const relative_y = y - target.offsetTop
    const ranged_x = scale(relative_x, 0, target.offsetWidth, 0, canvas_width())
    const ranged_y = scale(relative_y, 0, target.offsetHeight, 0, canvas_height())
    source.instance.exports.handle_click(ranged_x, ranged_y)
})

window.addEventListener("resize", _ => {
    const { offsetWidth, offsetHeight } = canvas
    canvas.width = offsetWidth
    canvas.height = offsetHeight
    const { width, height } = canvas
    source.instance.exports.set_size(width, height)
})

let last
function loop(timestamp) {
    if (has_panicked) {
        return
    }
    requestAnimationFrame(loop)
    if (!last) {
        last = timestamp
        return
    }

    const delta = timestamp - last
    const delta_per_second = delta / 1000
    last = timestamp
    clear_canvas()
    source.instance.exports.draw(delta_per_second)
}
source.instance.exports.init()
requestAnimationFrame(loop)

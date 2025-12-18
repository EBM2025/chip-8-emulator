import init, * as wasm from "./wasm.js"

const WIDTH = 64
const HEIGHT = 32
const SCALER = 15
const TICKS_PER_FRAME = 10
let anim_fram = 0

const canvas = document.getElementById("canvas")
canvas.width = WIDTH * SCALER
canvas.height = HEIGHT * SCALER

const ctx = canvas.getContext("2d")
ctx.fillStyle = "black"
ctx.fillRect(0, 0, WIDTH * SCALER, HEIGHT * SCALER)

const input = document.getElementById("fileinput")

async function run() {
    await init()
    let chip8 = new wasm.EmuWasm()

    document.addEventListener("keydown", function(e) {
        chip8.keypress(e, true)
    })

    document.addEventListener("keyup", function(e) {
        chip8.keypress(e, false)
    })

    input.addEventListener("change", function(e) {

        if (anim_fram != 0) {
            window.cancelAnimationFrame(anim_frame)
        }

        let file = e.target.files[0]
        if (!file) {
            alert("Failed to read file")
            return
        }

        let reader = new FileReader()
        reader.onload() = function(e) {
            let buff = reader.result
            const rom = new Uint8Array(buff)
            chip8.reset()
            chip8.load_game(rom)
            mainloop(chip8)
        }
        reader.readAsArrayBuffer(file)
    }, false)

    function mainloop(chip8) {

        for (let i = 0; i < TICKS_PER_FRAME; i++) {
            chip8.tick()
        }
        chip8.tick_timers()

        ctx.fillStyle = "black"
        ctx.fillRect(0, 0, WIDTH*SCALER, HEIGHT*SCALER)

        ctx.fillStyle = "white"
        chip8.draw_screen(SCALER)

        anim_frame = window.requestAnimationFrame(() => {
            mainloop(chip8)
        })
    }
}

run.catch(console.error)
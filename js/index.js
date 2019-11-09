const rust = import("../pkg/index.js");
const bg = import("../pkg/index_bg");

let imports = {};

Promise
    .all([rust, bg])
    .then(function (values) {
        imports.rust = values[0];
        imports.bg = values[1];

        init(values[0]);
    })
    .catch(console.error);


const SCALE = 2;
window.KeyboardState = class {
    constructor () {
        this.keyboard_state = Object.create(null);
        this.version = 0;
        window.addEventListener("keyup", e => this.keyboard_state[e.code] = undefined);
        window.addEventListener("keydown", evt => {
            if (this.is_held(evt.code)) {
                return;
            }
            this.keyboard_state[evt.code] = this.version;
        });
    }

    is_pressed(code) {
        return this.keyboard_state[code] === this.version;
    }

    is_held(code) {
        return this.keyboard_state[code] !== undefined;
    }
    tick() {
        this.version += 1;
    }
}

window.draw_layer = function (context, image, sprite_width, sprite_height, data, width, height) {
    const cells = new Uint8Array(imports.bg.memory.buffer, data, width * height * 2);
    for (var row = 0; row < height; row++) {
        let dest_y = row * sprite_height;

        for (var col = 0; col < width; col++) {
            let index = (row * width + col) * 2;
            let source_x = cells[index] * sprite_width;
            let source_y = cells[index + 1] * sprite_height;

            let dest_x = col * sprite_width;

            context.drawImage(
                image, 
                source_x,
                source_y,
                sprite_width, 
                sprite_height, 
                dest_x,
                dest_y,
                sprite_width, 
                sprite_height);
        }
    }
}

function init (rust) {
    let app = rust.LeapsAndBounds.new();

    let canvas = document.getElementById("canvas");
    let context = canvas.getContext('2d');
    context.imageSmoothingEnabled = false;

    canvas.addEventListener("click", evt => {
        let rect = canvas.getBoundingClientRect();
        let x = evt.clientX - rect.left;
        let y = evt.clientY - rect.top;
        app.left_click(x / SCALE, y / SCALE);
    });

    context.scale(SCALE, SCALE);

    let blocks = document.getElementById("blocks");
    let sprites = document.getElementById("sprites");

    let assets = rust.Assets.new(blocks, sprites);

    let old_time = 0;
    let loop = new_time => {
        const dt = new_time - old_time;
        old_time = new_time;

        context.clearRect(0, 0, canvas.width, canvas.height);

        app.step(dt);
        app.draw(context, assets);

        requestAnimationFrame(loop);
    }
    loop(0);
};
"use strict";

window.addEventListener("DOMContentLoaded", onLoad);
window.addEventListener("gamepadconnected", e => {
    console.log("Gamepad connected %d %s %d buttons %d axes",
            e.gamepad.index, e.gamepad.id,
            e.gamepad.buttons.length, e.gamepad.axes.length);
});

var loadFired = false;

function onLoad() {
    if (loadFired) return;
    loadFired = true;

    let screen = document.getElementById('screen');
    let canvasContext = screen.getContext('2d');
    let imageData = canvasContext.getImageData(0, 0, screen.width,
            screen.height);

    document.getElementById("rom")
            .addEventListener("change",
                ev => handleNewRom(
                    ev.target.files[0],
                    canvasContext,
                    imageData
                ));

    console.log("Loaded!");
}

var KEYBOARD_MAPPING = {
    "ArrowLeft": "left",
    "ArrowRight": "right",
    "ArrowUp": "up",
    "ArrowDown": "down",
    "a": "a",
    "s": "b",
    "d": "start",
    "f": "select"
};

var FPS = 60;
var FREQUENCY = 44100;
var AUDIO_FRAMES_PER_SEC = FREQUENCY / FPS;
var INTERNAL_AUDIO_FRAMES_PER_SEC = 512;
var SCREEN_X = 160;
var SCREEN_Y = 144;

function pollGamepad() {
    if (!navigator.getGamepads().length) {
        return null;
    }

    let gamepad = navigator.getGamepads()[0];
    if (!gamepad) {
        return null;
    }

    // This is for a PS4 gamepad
    let buttons = {
        a: false, // Button 1
        b: false, // Button 2
        start: false, // Button 8
        select: false, // Button 9
        up: false, // Axis 7 == -1
        down: false, // Axis 7 == 1
        left: false, // Axis 6 == -1
        right: false, // Axis 6 == 1
    };

    buttons.a = gamepad.buttons[2].pressed;
    buttons.b = gamepad.buttons[1].pressed;
    buttons.select = gamepad.buttons[8].pressed;
    buttons.start = gamepad.buttons[9].pressed;
    buttons.up = gamepad.axes[7] == -1;
    buttons.down = gamepad.axes[7] == 1;
    buttons.left = gamepad.axes[6] == -1;
    buttons.right = gamepad.axes[6] == 1;

    return buttons;
}

function convertSoundToStereoF16(sound) {
    let channels = [
      new Float32Array(AUDIO_FRAMES_PER_SEC),
      new Float32Array(AUDIO_FRAMES_PER_SEC)
    ];

    for (let channel = 0; channel < 2; channel++) {
        for (let i = 0; i < AUDIO_FRAMES_PER_SEC; i++) {
            channels[channel][i] = sound[i * 2 + channel] / 32768;
        }
    }

    return channels;
}

function refreshScreen(screen, img) {
    for (let i = 0; i < SCREEN_X; i++) {
        for (let j = 0; j < SCREEN_Y; j++) {
            let color = 255 - screen[i * SCREEN_Y + j] * 64;
            img[j * SCREEN_X * 4 + i * 4    ] = color;
            img[j * SCREEN_X * 4 + i * 4 + 1] = color;
            img[j * SCREEN_X * 4 + i * 4 + 2] = color;
            img[j * SCREEN_X * 4 + i * 4 + 3] = 255;
        }
    }
}

function refreshGamepad(gamepad, keyboard) {
    let poll = pollGamepad();
    if (poll === null) {
        poll = keyboard;
    }

    gamepad[0] = poll.a + 0;
    gamepad[1] = poll.b + 0;
    gamepad[2] = poll.start + 0;
    gamepad[3] = poll.select + 0;
    gamepad[4] = poll.up + 0;
    gamepad[5] = poll.down + 0;
    gamepad[6] = poll.left + 0;
    gamepad[7] = poll.right + 0;
}

/* ScriptProcessor only supports buffers of power-of-2 length:
 * 256, 512, 1024, 2048, 4096. The emulator runs off of buffers of length
 * 735 (one frame worth of sound data, 44100 Hz / 60 fps). This class
 * allows consumers to play music buffered for arbitrary-sized chunk of data.
 */
class ArbitraryAudioProcessor {
    constructor(bufferSize, internalBufferSize, channels) {
        if (bufferSize < internalBufferSize) {
            // bufferSize cannot be smaller than our update value in the
            // script processor, otherwise we may not have enough data
            // when the audio stack asks us for audio frames.
            console.error("bufferSize cannot be smaller than " +
                    "internalBufferSize.");
        }

        this.bufferSize = bufferSize;
        this.audioContext = new AudioContext();
        this.channels = channels;
        this.buffers = [];
        for (let i = 0; i < this.channels; i++) {
            this.buffers.push(new Float32Array(this.bufferSize * 4));
        }
        this.index = 0;
        this.startIndex = 0;
        this.remaining = 0;

        let scriptProcessor = this.audioContext.createScriptProcessor(
            internalBufferSize, this.channels, this.channels);
        scriptProcessor.onaudioprocess = e =>
            this._refreshAudio(e.outputBuffer);
        scriptProcessor.connect(this.audioContext.destination);
    }

    /* This method will provide audio data when requested by the
     * ScriptProcessor. The buffer is cyclic so it will wrap around it
     * when it reaches the end. */
    _refreshAudio(buffer) {
        if (this.remaining < buffer.length) {
            // We don't have enough data :( a pop will be heard
            return;
        }
        this.remaining -= buffer.length;
        if (this.remaining > buffer.length * 6) {
            // Audio is running a little too much ahead, let's discard the
            // next buffer to try catching up.
            this._refreshAudio(buffer);
        }
        let next = this.index + buffer.length;
        let dataLength = this.buffers[0].length;
        let end = next < dataLength ? next : dataLength;

        for (let i = 0; i < this.channels; i++) {
            buffer.getChannelData(i)
                .set(this.buffers[i].subarray(this.index, end));
        }

        let remaining = next - end;
        if (remaining == 0) {
            this.index = next;
            return;
        }

        this.index = 0;

        for (let i = 0; i < this.channels; i++) {
            let channelData = buffer.getChannelData(i);
            channelData.set(this.buffers[i].subarray(this.index, remaining),
                    channelData.length - remaining);
        }

        this.index = remaining;
    }

    /* Enqueue data to be played.
     *
     * data = [
     *      Float32Array(bufferSize), // left channel data
     *      Float32Array(bufferSize)  // right channel data
     * ]
     */
    pushData(data) {
        for (let i = 0; i < this.channels; i++) {
            if (data[i].length != this.bufferSize) {
                console.error("Data buffer size error.");
                return;
            }
            this.buffers[i].set(data[i], this.startIndex);
        }
        this.startIndex =
            (this.startIndex + data[0].length) % this.buffers[0].length;
        this.remaining += data[0].length;
    }
}

function romLoaded(rom, exports, canvasContext, imageData) {
    let Emu = {
        _memory: exports.memory,
        _alloc: exports.alloc,
        init: exports.init,
        copy_save: exports.copy_save,
        main_loop: exports.main_loop,
        audio_processor: new ArbitraryAudioProcessor(
                AUDIO_FRAMES_PER_SEC,
                INTERNAL_AUDIO_FRAMES_PER_SEC,
                2 /* channels */),
        // Allocates and copies obj to the heap, obj is a Typed array
        // e.g. Uint8Array, Int16Array, etc
        alloc: function(obj /* TypedArray */) {
            let size = obj.length * obj.BYTES_PER_ELEMENT;
            let ptr = this._alloc(size);
            let heap = new Uint8Array(this._memory.buffer, ptr, size);
            heap.set(new Uint8Array(obj.buffer));
            return {
                size: size,
                ptr: ptr
            };
        },
        // Gives a view of the heap object, it will return a Uint8Array
        // view which should be only used locally and not stored as any
        // allocation may invalidate it.
        view_u8: function(heap_ref) {
            return new Uint8Array(
                this._memory.buffer, heap_ref.ptr,
                heap_ref.size / Uint8Array.BYTES_PER_ELEMENT);
        },
        view_i16: function(heap_ref) {
            return new Int16Array(
                this._memory.buffer, heap_ref.ptr,
                heap_ref.size / Int16Array.BYTES_PER_ELEMENT);
        },
    };

    let romHeap = Emu.alloc(rom);

    let save = Uint8Array.from(
            (window.localStorage.getItem('save') || '').split(','));
    if (save.length != 32768) {
        console.error('invalid sized save.');
        save = new Uint8Array(32768);
    }
    let saveHeap = Emu.alloc(save);

    let screenHeap = Emu.alloc(new Uint8Array(SCREEN_X * SCREEN_Y));
    // Sound data is interleaved in the emulator
    //    sound = [left, right, left, right, ...]
    // for a frame of execution
    let soundHeap = Emu.alloc(new Int16Array(AUDIO_FRAMES_PER_SEC * 2));
    let gamepadHeap = Emu.alloc(new Uint8Array(8));

    Emu.init(romHeap.ptr, romHeap.size, saveHeap.ptr, screenHeap.ptr,
             soundHeap.ptr, gamepadHeap.ptr);

    let keyboard = {
        a: false,
        b: false,
        start: false,
        select: false,
        up: false,
        down: false,
        left: false,
        right: false,
    };

    window.addEventListener("keyup", e => {
        let key = KEYBOARD_MAPPING[e.key];
        if (key) {
            keyboard[key] = false;
            e.preventDefault();
        }
    });

    window.addEventListener("keydown", e => {
        let key = KEYBOARD_MAPPING[e.key];
        if (key) {
            keyboard[key] = true;
            e.preventDefault();
        }
    });

    function mainLoop() {
        Emu.main_loop();

        let screen = Emu.view_u8(screenHeap);
        let img = imageData.data;
        refreshScreen(screen, img);

        let gamepad = Emu.view_u8(gamepadHeap);
        refreshGamepad(gamepad, keyboard);

        let sound = Emu.view_i16(soundHeap);
        Emu.audio_processor
            .pushData(convertSoundToStereoF16(sound));

        canvasContext.putImageData(imageData, 0, 0);
        window.requestAnimationFrame(mainLoop);
    };

    function saveState() {
        Emu.copy_save();
        let save = Emu.view_u8(saveHeap);
        window.localStorage.setItem('save', save.toString());
        window.localStorage.setItem('saveTimestamp', new Date());
        window.setTimeout(saveState, 1000);
    }

    mainLoop();
    saveState();
}

function handleNewRom(rom, canvasContext, imageData) {
    let reader = new FileReader();

    reader.onload = () => {
        let rom = new Uint8Array(reader.result);
        fetch('emu.wasm')
            .then(r => r.arrayBuffer())
            .then(b => WebAssembly.instantiate(b, {env: {}}))
            .then(r => {
                romLoaded(rom, r.instance.exports, canvasContext,
                          imageData);
            });
    };

    reader.readAsArrayBuffer(rom);
}


// index.js
import init, { initThreadPool, mandelbrot } from './pkg/mandelbrot.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const imageData = ctx.createImageData(canvas.width, canvas.height);

init().then(async () => {
    await initThreadPool(navigator.hardwareConcurrency);
    const pixels = mandelbrot(canvas.width, canvas.height, 1000);
    imageData.data.set(pixels);
    ctx.putImageData(imageData, 0, 0);
});

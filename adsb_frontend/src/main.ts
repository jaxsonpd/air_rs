import { Aircraft } from "./aircraft";
import { Center, Position, PostionXY } from "./position";

const pos_canvas = document.getElementById("display") as HTMLCanvasElement | null;

if (!pos_canvas) {
    throw new Error("Canvas element not found");
}

const canvas: HTMLCanvasElement = pos_canvas

function resizeCanvas() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

resizeCanvas();

let ctx: CanvasRenderingContext2D;

const context = canvas.getContext("2d");

if (!context) {
    throw new Error("Could not get 2D context");
}

ctx = context;

let aircraft: Aircraft[] = [
    new Aircraft("8723", "ANZ100", 35000, new Position(33.9425, 33.9426))
]

ctx.fillStyle = "red";
ctx.fillRect(10, 10, 100, 100);

let center: Center = new Center(new Position(33.9425, 33.9425), new PostionXY(400, 400), 1);

function animate() {
    aircraft.forEach(plane => {
        plane.draw(ctx, center);
    });
    console.log(canvas?.width, canvas?.height);
    requestAnimationFrame(animate);
}

window.addEventListener("resize", () => {
    resizeCanvas();
    center.recenter(canvas.width, canvas.height);
});


animate();
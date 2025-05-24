import { Aircraft } from "./aircraft";
import { Center, Position, PostionXY } from "./position";
import { create_demo_aircraft, update_aircraft_demo, create_demo_center } from "./demo";

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

const demo = true;
let aircraft: Aircraft[] = [];

let center: Center = new Center(new Position(33.9425, 33.9425), new PostionXY(400, 400), 1);

if (demo) {
    aircraft = create_demo_aircraft();
    center = create_demo_center();
    center.recenter(canvas.width, canvas.height);
}

let lastUpdate = performance.now();
const UPDATE_RATE = 1000;

function animate(timestamp: DOMHighResTimeStamp) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    aircraft.forEach(plane => {
        plane.update_pos_xy(center);
        plane.draw(ctx);
    });
    
    if (demo && (timestamp - lastUpdate) >= UPDATE_RATE) {
        update_aircraft_demo(aircraft);
        lastUpdate = timestamp;
    }

    requestAnimationFrame(animate);
}

window.addEventListener("resize", () => {
    resizeCanvas();
    center.recenter(canvas.width, canvas.height);
});


requestAnimationFrame(animate);
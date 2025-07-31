import { Aircraft } from "./aircraft";
import { Center, Position, PostionXY } from "./position";
import { create_demo_aircraft, update_aircraft_demo, create_demo_center } from "./demo";
import { AircraftSummary } from "../../bindings/AircraftSummary";

const pos_canvas = document.getElementById("display") as HTMLCanvasElement | null;

if (!pos_canvas) {
    throw new Error("Canvas element not found");
}

const canvas: HTMLCanvasElement = pos_canvas;

let ctx: CanvasRenderingContext2D;

const context = canvas.getContext("2d");

if (!context) {
    throw new Error("Could not get 2D context");
}

ctx = context;

ctx.font = 'bold 12.5px Courier New'

function resizeCanvas() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    ctx.font = 'bold 12.5px Courier New'
}

resizeCanvas();

let mouse_x: number = 0;
let mouse_y: number = 0;

canvas.addEventListener('mousemove', e => {
    mouse_x = e.clientX;
    mouse_y = e.clientY;
});

canvas.addEventListener('click', e => {
    const mx = e.clientX;
    const my = e.clientY;
    for (const ac of aircraft) {
        if (ac.check_hover(mx, my)) {
            ac.toggle_expanded();
            return;
        }
    }
});

const demo = false;
const socket = new WebSocket("ws://localhost:8080/ws");
let aircraft: Aircraft[] = [];

let center: Center = new Center(new Position(33.9425, 33.9425), new PostionXY(400, 400), 1);

if (demo) {
    aircraft = create_demo_aircraft();
    center = create_demo_center();
    center.recenter(canvas.width, canvas.height);
}

function draw_statistics(ctx: CanvasRenderingContext2D, aircraft: Aircraft[]) {
    const num_planes = aircraft.length;
    const max_alt = Math.max(...aircraft.map(plane => plane.altitude));
    const min_alt = Math.min(...aircraft.map(plane => plane.altitude));

    const lines = [
        `Tracking Stats:`,
        `# Aircraft: ${num_planes}`,
        `Max Altitude: ${max_alt}`,
        `Min Altitude: ${min_alt}`
    ]

    let box_x = 10;
    let box_y = 10;
    let padding = 10;
    let text_width = 0;
        
    lines.forEach(line => {
        const length = ctx.measureText(line).width;
        if (length > text_width) {
            text_width = length
        }
    });

    const width = text_width + padding * 2;
    const height = padding + 15 * lines.length;

    ctx.fillStyle = 'black';
    ctx.fillRect(box_x, box_y, width, height);
    ctx.strokeStyle = 'white';
    ctx.strokeRect(box_x, box_y, width, height);

    ctx.fillStyle = 'white';

    for (let i = 0; i < lines.length; i++) {
        ctx.fillText(lines[i], box_x + padding, box_y + (12.5 * (i+1)));
        
    }

}

/// Handle new aircraft data received from the WebSocket
function handle_new_aircraft(aircraftData: any) {
    const exsitingAircraft = aircraft.find(ac => ac.icao === aircraftData.icao);

    let geoPosition = center.pos;
    if (aircraftData.geoPosition) {
        geoPosition = new Position(aircraftData.geoPosition.latitude, aircraftData.geoPosition.longitude);
    }

    if (exsitingAircraft) {
        // Update existing aircraft
        exsitingAircraft.callsign = aircraftData.callsign;
        exsitingAircraft.altitude = aircraftData.altitude;
        exsitingAircraft.pos = geoPosition;
        exsitingAircraft.last_contact = Date.now();
    } else {
        // Create new aircraft
        const newAircraft = new Aircraft(
            aircraftData.icao,
            aircraftData.callsign,
            aircraftData.altitude,
            geoPosition
        );
        aircraft.push(newAircraft);
    }
}

let lastUpdate = performance.now();
const UPDATE_RATE = 1000;

function animate(timestamp: DOMHighResTimeStamp) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    if (aircraft.length === 0) {
        center.scale_p_p_m = canvas.width / 60000;
    } else {
        const bounds = aircraft.reduce((acc, plane) => {
            if (plane.pos) {
                acc.mindist = Math.min(acc.mindist, plane.pos.get_distance(center.pos));
                acc.maxdist = Math.max(acc.maxdist, plane.pos.get_distance(center.pos));
            }
            return acc;
        }, {
            mindist: Infinity,
            maxdist: -Infinity,
        });

        center.scale_p_p_m = canvas.width / (bounds.maxdist*2);
    }
    ctx.beginPath();
    ctx.moveTo(25, canvas.height - 25);
    ctx.lineTo(25 + center.scale_p_p_m * 1000, canvas.height - 25);
    ctx.stroke();

    if (canvas.width > 200 && canvas.height > 200) {
        draw_statistics(ctx, aircraft);
    }

    aircraft.forEach(plane => {
        plane.update_pos_xy(center);
        plane.draw(ctx);
        if (plane.check_hover(mouse_x, mouse_y)) {
            plane.draw_expanded(ctx);
        }
    });

    if ((timestamp - lastUpdate) >= UPDATE_RATE) {
        if (demo) {
            update_aircraft_demo(aircraft);
            lastUpdate = timestamp;
        } else {
            socket.onmessage = (event) => {
            const aircraft = JSON.parse(event.data, );
            console.log("Aircraft:", aircraft);
            handle_new_aircraft(aircraft);
        };

        }
    }

    requestAnimationFrame(animate);
}

window.addEventListener("resize", () => {
    resizeCanvas();
    center.recenter(canvas.width, canvas.height);
});


requestAnimationFrame(animate);

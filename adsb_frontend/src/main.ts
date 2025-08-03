import { Aircraft } from "./aircraft";
import { Center, Position, PostionXY } from "./position";
import { create_demo_aircraft, update_aircraft_demo, create_demo_center } from "./demo";
import { Airfield, loadAirfieldsFromCSV} from "./airfield/airfield"
import { AircraftSummary } from "../../bindings/AircraftSummary";


const CONFIG = {
    UPDATE_RATE: 1000,
    DEMO_MODE: false,
    DEFAULT_CENTER_POS: new Position(-41.296466, 174.785409),
    DEFAULT_CENTER_PPM: 60000,
    DEFAULT_CENTER_XY: new PostionXY(400, 400),
    FONT: 'bold 12.5px Courier New',
    AIRFIELDS_CSV_LOCATION: "/airfields.csv"
};

/**
 * Draw a statistics window in the top left corner of the screeen
 * 
 * @param ctx the canvas element to draw on
 * @param aircraft the aircraft to pull data from
 */
function draw_statistics(ctx: CanvasRenderingContext2D, aircraft: Aircraft[]) {
    const num_planes = aircraft.length;
    const max_alt = Math.max(...aircraft.map(plane => plane.altitude));
    const min_alt = Math.min(...aircraft.map(plane => plane.altitude));

    const lines = [
        `Tracking Stats:`,
        `# Aircraft: ${num_planes}`,
        `Max Altitude: ${max_alt}`,
        `Min Altitude: ${min_alt}`
    ];

    const padding = 10;
    let text_width = Math.max(...lines.map(line => ctx.measureText(line).width));
    const width = text_width + padding * 2;
    const height = padding + 15 * lines.length;

    const box_x = 10;
    const box_y = 10;

    ctx.fillStyle = 'black';
    ctx.fillRect(box_x, box_y, width, height);
    ctx.strokeStyle = 'white';
    ctx.strokeRect(box_x, box_y, width, height);
    ctx.fillStyle = 'white';

    for (let i = 0; i < lines.length; i++) {
        ctx.fillText(lines[i], box_x + padding, box_y + (12.5 * (i + 1)));
    }
}

/**
 * Handle in coming aircraft data.
 * 
 * @param aircraftData the incoming aircraft data
 * @param aircraft the current aircraft available
 * @param center the center point of the screen for placement
 */
function handle_new_aircraft(aircraftData: any, aircraft: Aircraft[], center: Center) {
    const existingAircraft = aircraft.find(ac => ac.icao === aircraftData.icao);

    let geoPosition = center.pos;
    if (aircraftData.geoPosition) {
        geoPosition = new Position(
            aircraftData.geoPosition.latitude,
            aircraftData.geoPosition.longitude
        );
    }

    if (existingAircraft) {
        existingAircraft.callsign = aircraftData.callsign;
        existingAircraft.altitude = aircraftData.altitude;
        existingAircraft.pos = geoPosition;
        existingAircraft.last_contact = Date.now();
    } else {
        const newAircraft = new Aircraft(
            aircraftData.icao,
            aircraftData.callsign,
            aircraftData.altitude,
            geoPosition
        );
        aircraft.push(newAircraft);
    }
}

class AircraftDisplayApp {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private aircraft: Aircraft[] = [];
    private airfields: Airfield[] = [];
    private center: Center;
    private mouse = { x: 0, y: 0 };
    private lastUpdate = performance.now();
    private socket: WebSocket | null;

    constructor(canvasId: string) {
        const canvas = document.getElementById(canvasId) as HTMLCanvasElement | null;
        if (!canvas) throw new Error("Canvas element not found");

        const ctx = canvas.getContext("2d");
        if (!ctx) throw new Error("Could not get 2D context");

        this.canvas = canvas;
        this.ctx = ctx;
        this.ctx.font = CONFIG.FONT;

        this.center = new Center(
            CONFIG.DEFAULT_CENTER_POS,
            CONFIG.DEFAULT_CENTER_XY,
            CONFIG.DEFAULT_CENTER_PPM
        );

        if (!CONFIG.DEMO_MODE) {
            this.socket = new WebSocket("ws://localhost:8080/ws");
        } else {
            this.socket = null;
        }

        if (CONFIG.DEMO_MODE) {
            this.aircraft = create_demo_aircraft();
            this.center = create_demo_center();
            this.center.recenter(this.canvas.width, this.canvas.height);
        }

        loadAirfieldsFromCSV(CONFIG.AIRFIELDS_CSV_LOCATION).then((loaded) => {
            this.airfields = loaded;
        })

        this.initEventListeners();
        this.resizeCanvas();
        this.center.recenter(this.canvas.width, this.canvas.height);
        requestAnimationFrame(this.animate.bind(this));
    }

    private initEventListeners() {
        window.addEventListener("resize", () => {
            this.resizeCanvas();
            this.center.recenter(this.canvas.width, this.canvas.height);
        });

        this.canvas.addEventListener("mousemove", e => {
            this.mouse.x = e.clientX;
            this.mouse.y = e.clientY;
        });

        this.canvas.addEventListener("click", e => {
            const mx = e.clientX;
            const my = e.clientY;
            for (const ac of this.aircraft) {
                if (ac.check_hover(mx, my)) {
                    ac.toggle_expanded();
                    return;
                }
            }
        });

        if (this.socket != null) {
            this.socket.onmessage = (event) => {
                const data = JSON.parse(event.data);
                handle_new_aircraft(data, this.aircraft, this.center);
            };
        }
    }

    private resizeCanvas() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
        this.ctx.font = CONFIG.FONT;
    }

    private update_scale() {
        if (this.aircraft.length === 0) {
            this.center.scale_p_p_m = this.canvas.width / CONFIG.DEFAULT_CENTER_PPM;
        } else {
            const bounds = this.aircraft.reduce((acc, plane) => {
                if (plane.pos) {
                    acc.maxdist = Math.max(acc.maxdist, plane.pos.get_distance(this.center.pos));
                }
                return acc;
            }, { maxdist: CONFIG.DEFAULT_CENTER_PPM });

            this.center.scale_p_p_m = this.canvas.width / (bounds.maxdist * 2.3);
        }
    }

    private draw_scale() {
        this.ctx.beginPath();
        this.ctx.moveTo(25, this.canvas.height - 25);
        this.ctx.lineTo(25 + this.center.scale_p_p_m * 1000, this.canvas.height - 25);
        this.ctx.stroke();
    }

    private animate(timestamp: DOMHighResTimeStamp) {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        this.update_scale()
        this.draw_scale()

        if (this.canvas.width > 200 && this.canvas.height > 200) {
            draw_statistics(this.ctx, this.aircraft);
        }

        this.aircraft.forEach(plane => {
            plane.update_pos_xy(this.center);
            plane.draw(this.ctx);
            if (plane.check_hover(this.mouse.x, this.mouse.y)) {
                plane.draw_expanded(this.ctx);
            }
        });

        if ((timestamp - this.lastUpdate) >= CONFIG.UPDATE_RATE) {
            if (CONFIG.DEMO_MODE) {
                update_aircraft_demo(this.aircraft);
            }
            this.lastUpdate = timestamp;
        }

        this.airfields.forEach(airfield => {
            if (this.center.check_visible(airfield.position)) {
                airfield.draw(this.ctx, this.center);
            }
        });

        requestAnimationFrame(this.animate.bind(this));
    }
}

new AircraftDisplayApp("display");

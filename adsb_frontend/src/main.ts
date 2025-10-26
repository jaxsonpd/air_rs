import { Aircraft } from "./aircraft";
import { Center, Position, PositionXY } from "./position";
import { create_demo_aircraft, update_aircraft_demo, create_demo_center } from "./demo";
import { Airfield, loadAirfieldsFromCSV } from "./airfield/airfield"
import { get_text_height } from "./utils";
import { ConfigBar } from "./config_bar/config_bar";
import { CONFIG } from "./config";
import { center_store, aircraft_store } from "./store/store";

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
    const box_x = 20;
    const box_y = 20;

    const text_width = Math.max(...lines.map(line => ctx.measureText(line).width));
    // const width = text_width + padding * 2;
    const text_height = get_text_height(ctx, lines[0]);
    // const height = padding + text_height * lines.length;

    // ctx.strokeStyle = 'white';
    // ctx.lineWidth = text_height/10;
    // ctx.strokeRect(box_x, box_y, width, height);
    ctx.fillStyle = 'white';

    for (let i = 0; i < lines.length; i++) {
        ctx.fillText(lines[i], box_x, box_y + (i) * (padding + text_height));
    }
}

/**
 * Draw a table of aircraft without positions currently
 *
 * @param ctx the canvas to draw on
 * @param aircraft list of aircraft to draw
 * @param position the locating position for the table (anchor point)
 * @param position_corner the position corner to use for locating
 */
function draw_aircraft_table(
    ctx: CanvasRenderingContext2D,
    aircraft: Aircraft[],
    position: PositionXY,
    position_corner: "top-left" | "top-right" | "bottom-left" | "bottom-right"
) {
    const filtered = aircraft.filter(a => a.pos === null);
    if (filtered.length === 0) return;

    const headers = ["ICAO", "Callsign", "Altitude", "Last Contact (s)"];
    const rows = filtered.map(a => [
        a.icao.toString(16).toUpperCase(),
        a.callsign || "",
        a.altitude.toString(),
        ((Date.now() - a.last_contact) / 1000).toFixed(1)
    ]);
    const tableData = [headers, ...rows];

    const cellWidth = Math.max(100,
                        Math.max(...tableData.map(line =>
                            Math.max(...line.map(cell => ctx.measureText(cell).width))
                        )
                    ));
    const cellHeight = get_text_height(ctx, tableData[0][0]);
    const padding = 10;


    const cols = headers.length;
    const totalWidth = cols * cellWidth + padding;
    const totalHeight = tableData.length * cellHeight;

    // Get base x/y for top-left of table
    const canvasWidth = ctx.canvas.width;
    const canvasHeight = ctx.canvas.height;

    let x = 0;
    let y = 0;

    switch (position_corner) {
        case "top-left":
            x = position.x;
            y = position.y;
            break;
        case "top-right":
            x = canvasWidth - position.x - totalWidth;
            y = position.y;
            break;
        case "bottom-left":
            x = position.x;
            y = canvasHeight - position.y - totalHeight;
            break;
        case "bottom-right":
            x = canvasWidth - position.x - totalWidth;
            y = canvasHeight - position.y - totalHeight;
            break;
    }

    ctx.save(); // protect other drawings

    ctx.textBaseline = "middle";
    ctx.textAlign = "center";
    ctx.fillStyle = "#FFFFFF";

    for (let r = 0; r < tableData.length; r++) {
        for (let c = 0; c < cols; c++) {
            const cellX = x + c * cellWidth;
            const cellY = y + (r) * (cellHeight + padding);

            ctx.fillText(String(tableData[r][c]), cellX + cellWidth / 2, cellY);
        }
    }

    ctx.restore(); // restore previous state so no offsets leak
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

    let geoPosition = null;
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
    private airfields: Airfield[] = [];
    private config_bar: ConfigBar;
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

        if (!CONFIG.DEMO_MODE) {
            this.socket = new WebSocket("ws://localhost:8080/ws");
        } else {
            this.socket = null;
        }

        if (CONFIG.DEMO_MODE) {
            aircraft_store.set(create_demo_aircraft());
            center_store.set(create_demo_center());
        }

        loadAirfieldsFromCSV(CONFIG.AIRFIELDS_CSV_LOCATION).then((loaded) => {
            this.airfields = loaded;
        })

        this.initEventListeners();
        this.resizeCanvas();
        center_store.get().recenter(this.canvas.width, this.canvas.height);
        requestAnimationFrame(this.animate.bind(this));

        this.config_bar = new ConfigBar(new PositionXY(this.canvas.width / 2, 25))
    }

    private initEventListeners() {
        window.addEventListener("resize", () => {
            this.resizeCanvas();
            center_store.get().recenter(this.canvas.width, this.canvas.height);
            this.config_bar.center = new PositionXY(this.canvas.width / 2, 25);
        });

        this.canvas.addEventListener("mousemove", e => {
            this.mouse.x = e.offsetX;
            this.mouse.y = e.offsetY;
        });

        this.canvas.addEventListener("click", e => {
            const mx = e.offsetX;
            const my = e.offsetY;
            for (const ac of aircraft_store.get()) {
                if (ac.check_hover(mx, my)) {
                    ac.toggle_expanded();
                    return;
                }
            }
        });

        if (this.socket != null) {
            this.socket.onmessage = (event) => {
                const data = JSON.parse(event.data);
                handle_new_aircraft(data, aircraft_store.get(), center_store.get());
            };
        }

        document.fonts.ready.then(() => {
            this.ctx.font = CONFIG.FONT;
            this.animate(performance.now());
        });
    }

    private resizeCanvas() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
        this.ctx.font = CONFIG.FONT;
    }

    private update_scale() {
        if (aircraft_store.get().length === 0) {
            center_store.get().scale_p_p_m = this.canvas.width / CONFIG.DEFAULT_CENTER_PPM;
        } else {
            const bounds = aircraft_store.get().reduce((acc, plane) => {
                if (plane.pos) {
                    acc.maxdist = Math.max(acc.maxdist, plane.pos.get_distance(center_store.get().pos));
                }
                return acc;
            }, { maxdist: CONFIG.DEFAULT_CENTER_PPM });

            center_store.get().scale_p_p_m = this.canvas.width / (bounds.maxdist * 2.3);
        }
    }

    private draw_scale() {
        this.ctx.beginPath();
        this.ctx.moveTo(25, this.canvas.height - 25);
        this.ctx.lineTo(25 + center_store.get().scale_p_p_m * 1000, this.canvas.height - 25);
        this.ctx.stroke();
    }

    private animate(timestamp: DOMHighResTimeStamp) {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.font = CONFIG.FONT;

        this.update_scale();
        this.draw_scale();

        if ((timestamp - this.lastUpdate) >= CONFIG.UPDATE_RATE) {
            if (CONFIG.DEMO_MODE) {
                update_aircraft_demo(aircraft_store.get());
            }
            this.lastUpdate = timestamp;
        }

        let no_pos_aircraft: Aircraft[] = [];
        aircraft_store.get().forEach(plane => {
            if (plane.pos != null) {
                plane.update_pos_xy(center_store.get());
                plane.draw(this.ctx);
                plane.check_hover(this.mouse.x, this.mouse.y);
            } else {
                no_pos_aircraft.push(plane);
            }
        });

        if (this.canvas.width > 700 && this.canvas.height > 500) {
            draw_statistics(this.ctx, aircraft_store.get());
            draw_aircraft_table(this.ctx, no_pos_aircraft, new PositionXY(20, 20), "top-right");
            this.config_bar.draw(this.ctx);
        }

        this.airfields.forEach(airfield => {
            if (center_store.get().check_visible(airfield.position)) {
                airfield.draw(this.ctx, center_store.get());
            }
        });

        requestAnimationFrame(this.animate.bind(this));
    }
}

new AircraftDisplayApp("display");

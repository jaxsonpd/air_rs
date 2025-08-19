/// Object to handle aircraft data and plotting

import { Position, Center, PositionXY } from "./position";
import { get_text_height } from "./utils";

export class Aircraft {
    private pos_xy: PositionXY = new PositionXY(0, 0);
    private extended_pane: boolean = false;
    private hover: boolean = false;
    private suppress_details: boolean = false;
    public last_contact: number = Date.now();
    constructor(
        public icao: number,
        public callsign: string,
        public altitude: number,
        public pos: Position | null,
    ) { }

    public update_pos_xy(center: Center) {
        let position: PositionXY;
        if (this.pos !== null) {
            position = center.get_xy(this.pos);
            this.pos_xy.x = position.x;
            this.pos_xy.y = position.y;
        }
    }

    /**
     * Draw a aeroplane on the canvas just position icao and altitude
     */
    public draw(ctx: CanvasRenderingContext2D) {
        const line_end = new PositionXY(this.pos_xy.x + 10, this.pos_xy.y - 17.5); 

        /// Draw dot
        ctx.fillStyle = 'white';
        ctx.beginPath();
        ctx.arc(this.pos_xy.x, this.pos_xy.y, 3, 0, 2 * Math.PI);
        ctx.fill();

        // Draw indicator line
        ctx.strokeStyle = 'white';
        ctx.beginPath();
        ctx.moveTo(this.pos_xy.x + 2, this.pos_xy.y - 2);
        ctx.lineTo(line_end.x, line_end.y);
        ctx.stroke();
        
        if (!this.extended_pane && !this.hover || this.suppress_details) { 
            const icao_line = `${this.icao.toString(16)}`;
            const altitude_line = `${this.altitude} ft`;

            const text_width = Math.max(ctx.measureText(icao_line).width, ctx.measureText(altitude_line).width)
            const text_height = get_text_height(ctx, icao_line);
            
            const padding = new PositionXY(7, 5);
            const box_height = padding.y * 3 + text_height * 2;
            const box_width = padding.x * 2 + text_width;

            const box_pos = new PositionXY(line_end.x, this.pos_xy.y - box_height);

            ctx.strokeStyle = 'white';
            ctx.strokeRect(box_pos.x, box_pos.y, box_width, box_height);

            ctx.fillStyle = 'white';
            ctx.fillText(icao_line, box_pos.x + padding.x, box_pos.y + padding.y + text_height);
            ctx.fillText(altitude_line, box_pos.x + padding.x, box_pos.y + padding.y * 2 + text_height * 2);
        } else {
            this.draw_expanded(ctx);
        }
    }

    /**
     * Draw the expanded text window
     * @param ctx the canvas to draw on
     */
    public draw_expanded(ctx: CanvasRenderingContext2D) {
        const latLonLine = this.pos
            ? `Lat/Lon: ${this.pos.latitude.toFixed(3)}, ${this.pos.longitude.toFixed(3)}`
            : "Lat/Lon: N/A";

        const lines = [
            `ICAO: ${this.icao.toString(16)}`,
            `Callsign: ${this.callsign}`,
            `Altitude: ${this.altitude} ft`,
            `Last Contact: ${new Date(this.last_contact).toLocaleTimeString()}`,
            latLonLine
        ]
        const line_end = new PositionXY(this.pos_xy.x + 10, this.pos_xy.y - 17.5); 

        const text_width = Math.max(...lines.map(line => ctx.measureText(line).width));
        const text_height = get_text_height(ctx, lines[0]);
        
        const padding = new PositionXY(7, 5);
        const box_height = padding.y * 2 + (padding.y + text_height) * lines.length;
        const box_width = padding.x * 2 + text_width;

        const box_pos = new PositionXY(line_end.x, this.pos_xy.y - box_height);

        ctx.strokeStyle = 'white';
        ctx.strokeRect(box_pos.x, box_pos.y, box_width, box_height);

        ctx.fillStyle = 'white';

        ctx.fillStyle = 'white';

        for (let i = 0; i < lines.length; i++) {
            ctx.fillText(lines[i], box_pos.x + padding.x, box_pos.y + (padding.y + text_height) * (i + 1));
        }
    }

    /**
     * Toggle if to show the expanded pane
     */
    public toggle_expanded() {
        this.extended_pane = !this.extended_pane;

        if (!this.extended_pane) {
            this.suppress_details = true;
        } else {
            this.suppress_details = false;
        }
    }

    /**
     * Check if the mouse is over the window updating the 
     * internal state if so
     * 
     * @param x the mouse x position
     * @param y the mouse y position
     * @returns true if hovering
     */
    public update_hover(x: number, y: number): boolean {
        const dx = x - this.pos_xy.x;
        const dy = y - this.pos_xy.y;

        const dist = Math.sqrt(dx * dx + dy * dy);
        const hover =  dist < 8;

        this.hover = hover;

        if (!hover) {
            this.suppress_details = false;
        }

        return hover;
    }
}
/// Object to handle aircraft data and plotting

import { Position, Center, PostionXY } from "./position";

export class Aircraft {
    private pos_xy: PostionXY;
    public last_contact: number;
    constructor(
        public icao: number,
        public callsign: string,
        public altitude: number,
        public pos: Position,
    ) {
        this.pos_xy = new PostionXY(0, 0);
        this.last_contact = Date.now()
     }

    public update_pos_xy(center: Center) {
        let position: PostionXY = center.get_xy(this.pos);
        
        this.pos_xy.x = position.x;
        this.pos_xy.y = position.y;
    }   

    /**
     * Draw a aeroplane on the canvas just position icao and altitude
     */
    public draw(ctx: CanvasRenderingContext2D) {
        ctx.fillStyle = 'white';
        ctx.beginPath();
        ctx.arc(this.pos_xy.x, this.pos_xy.y, 3, 0, 2 * Math.PI);
        ctx.fill();

        const icao_line = `${this.icao}`;
        const altitude_line = `${this.altitude} ft`;
        const padding = 4
        const text_width = Math.max(ctx.measureText(icao_line).width, ctx.measureText(altitude_line).width)
        const box_Height = 30;
        const boxX = this.pos_xy.x + 10
        const boxY = this.pos_xy.y - 35;

        ctx.strokeStyle = 'white';
        ctx.beginPath();
        ctx.moveTo(this.pos_xy.x + 2, this.pos_xy.y - 2);
        ctx.lineTo(boxX, boxY + box_Height / 2);
        ctx.stroke();

        ctx.fillStyle = 'black';
        ctx.fillRect(boxX, boxY, text_width + padding * 2, box_Height);
        ctx.strokeStyle = 'white';
        ctx.strokeRect(boxX, boxY, text_width + padding * 2, box_Height);

        ctx.fillStyle = 'white';
        ctx.fillText(icao_line, boxX + padding, boxY + 12);
        ctx.fillText(altitude_line, boxX + padding, boxY + 25);
    }

    /**
     * Draw the expanded text window
     * @param ctx the canvas to draw on
     */
    public draw_expanded(ctx: CanvasRenderingContext2D) {
        const lines = [
            `ICAO: ${this.icao}`,
            `Callsign: ${this.callsign}`,
            `Altitude: ${this.altitude} ft`,
            `Last Contact: ${new Date(this.last_contact).toLocaleTimeString()}`,
            `Lat/Lon: ${this.pos.latitude.toFixed(3)}, ${this.pos.longitude.toFixed(3)}`
        ]
        const boxX = this.pos_xy.x + 10
        const boxY = this.pos_xy.y - (5+15*lines.length);

        const padding = 4
        let text_width = 0;
        
        lines.forEach(line => {
            const length = ctx.measureText(line).width;
            if (length > text_width) {
                text_width = length
            }
        });

        const box_Height = 15*lines.length;

        ctx.fillStyle = 'black';
        ctx.fillRect(boxX, boxY, text_width + padding * 2, box_Height);
        ctx.strokeStyle = 'white';
        ctx.strokeRect(boxX, boxY, text_width + padding * 10, box_Height);

        ctx.fillStyle = 'white';

        for (let i = 0; i < lines.length; i++) {
            ctx.fillText(lines[i], boxX + padding, boxY + (12.5 * (i+1)));
            
        }
    }

    public check_hover(x: number, y: number) {
        const dx = x - this.pos_xy.x;
        const dy = y - this.pos_xy.y;

        const dist = Math.sqrt(dx * dx + dy * dy);
        return dist < 8;
    }
}
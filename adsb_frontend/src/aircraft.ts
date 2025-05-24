/// Object to handle aircraft data and plotting

import { Position, Center, PostionXY } from "./position";

export class Aircraft {
    private pos_xy: PostionXY;

    constructor(
        public icao: number,
        public callsign: string,
        public altitude: number,
        public pos: Position,
    ) {
        this.pos_xy = new PostionXY(0, 0);
     }

    public update_pos_xy(center: Center) {
        let position: PostionXY = center.get_xy(this.pos);
        
        this.pos_xy.x = position.x;
        this.pos_xy.y = position.y;
    }   

    /**
     * Draw a aeroplane on the canvas
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

    
}
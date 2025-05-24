/// Object to handle aircraft data and plotting

import { Position, Center, PostionXY } from "./position";

export class Aircraft {
    constructor(
        public icao: string,
        public callsign: string,
        public altitude: number,
        public pos: Position
    ) { }

    /**
     * Draw a aeroplane on the canvas
     */
    public draw(ctx: CanvasRenderingContext2D, center: Center) {
        let position: PostionXY = center.get_xy(this.pos);

        ctx.fillStyle = 'white';
        ctx.beginPath();
        ctx.arc(position.x, position.y, 3, 0, 2 * Math.PI);
        ctx.fill();
    }
}
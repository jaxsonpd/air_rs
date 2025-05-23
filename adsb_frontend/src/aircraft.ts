/// Object to handle aircraft data and plotting

export class Aircraft {
    constructor (
        public icao: string,
        public callsign: string,
        public altitude: number,
        public latitude: number,
        public longitude: number
    ) {}

    draw (ctx: CanvasRenderingContext2D) {
        ctx.fillStyle = 'white';
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, 2 * Math.PI);
        ctx.fill();
    }
}
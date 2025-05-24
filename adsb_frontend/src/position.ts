/// Hold position information

export class Position {
    constructor(
        public latitude: number,
        public longitude: number,
    ) { }
    /**
     * get_distance
     * 
     * Get the distance in meters from one position to the next
     * using the haversine formula: https://en.wikipedia.org/wiki/Haversine_formula
     */
    public get_distance(point2: Position): number {
        const R = 6371000;
        const toRad = (deg: number) => deg * Math.PI / 180;

        const dLat = toRad(point2.latitude - this.latitude);
        const dLon = toRad(point2.longitude - this.longitude);
        const lat1 = toRad(this.latitude);
        const lat2 = toRad(point2.latitude);

        const a = Math.sin(dLat / 2) ** 2 +
              Math.cos(lat1) * Math.cos(lat2) *
              Math.sin(dLon / 2) ** 2;

        const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));

        return R * c;
    }   

    /**
     * Get the bearing to another point
     * 
     * @param to the point to get the bearing of
     * @returns the angle in radians to the point
     */
    public get_bearing(to: Position): number {
        const toRad = (deg: number) => deg * Math.PI / 180;
        const lat1 = toRad(this.latitude);
        const lat2 = toRad(to.latitude);
        const dLon = toRad(to.longitude - this.longitude);

        const y = Math.sin(dLon) * Math.cos(lat2);
        const x = Math.cos(lat1) * Math.sin(lat2) -
                Math.sin(lat1) * Math.cos(lat2) * Math.cos(dLon);

        return Math.atan2(y, x); // Radians
    }
}

export class PostionXY {
    constructor (
        public x: number,
        public y: number
    ) {}
}

export class Center {
    constructor(
        public pos: Position,
        public pos_xy: PostionXY,
        public scale: number,
    ) {}

    /**
     * get_xy
     * 
     * Convert a position to an xy position based off the center
     */
    public get_xy(pos: Position): PostionXY {
        const distance = this.pos.get_distance(pos);
        const bearing = this.pos.get_bearing(pos);

        const dx = distance * Math.sin(bearing); // East-West offset in meters
        const dy = -distance * Math.cos(bearing); // North-South offset in meters (negative so north is up)

        const x = this.pos_xy.x + dx * this.scale;
        const y = this.pos_xy.y + dy * this.scale;

        return new PostionXY(x, y);         
    }

    public recenter(width: number, height: number) {
        this.pos_xy.x = Math.floor(width / 2);
        this.pos_xy.y = Math.floor(height / 2);
    }
}
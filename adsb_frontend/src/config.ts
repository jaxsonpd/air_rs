import { Position, PositionXY } from "./position";

export const CONFIG = {
    UPDATE_RATE: 1000,
    DEMO_MODE: true,
    DEFAULT_CENTER_POS: new Position(-41.296466, 174.785409),
    DEFAULT_CENTER_PPM: 10000,
    DEFAULT_CENTER_XY: new PositionXY(400, 400),
    FONT: "16px 'Consolas', monospace",
    AIRFIELDS_CSV_LOCATION: "/airfields.csv"
};
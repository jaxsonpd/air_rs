/// Demo functionality

import { Aircraft } from "./aircraft";
import { Position, PositionXY, Center } from "./position";

export function create_demo_center(): Center {
    return new Center(new Position(-41.294260, 174.776858),
        new PositionXY(0, 0),
        0.05);
}

export function create_demo_aircraft(): Aircraft[] {
    let aircraft: Aircraft[] = [
        new Aircraft(0x8723c8, "ANZ100", 0, new Position(-41.326694, 174.806931)),
        new Aircraft(0x8723d8, "ANZ200", 1000, new Position(-41.287131, 174.723534)),
        new Aircraft(0x8723b8, "ANZ300", 9500, new Position(-41.261161, 174.929136)),
        new Aircraft(0xc83678, "ANZ400", 500, null)
    ];

    return aircraft;
}

export function update_aircraft_demo(aircraft: Aircraft[]) {
    aircraft[0].altitude += 1
    if (aircraft[0].pos != null) {
        aircraft[0].pos.latitude += 0.0005
    }
    
    aircraft[1].altitude -= 1
    if (aircraft[1].pos != null) {
        aircraft[1].pos.longitude += 0.0005
    }

    aircraft[2].altitude += 0
    if (aircraft[2].pos != null) {
        aircraft[2].pos.latitude -= 0.001
    }
}
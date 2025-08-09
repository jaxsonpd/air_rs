/**
 * Handle the drawing and placement of airfields
 * 
 * @author Jack Duignan
 */

import Papa from "papaparse";
import { Center, Position } from "../position";

/**
 * Loads and parses airfields from a CSV file.
 * CSV must have headers: icao,lat,lon,name
 *
 * @param url - URL or path to the CSV file
 * @returns Promise<Airfield[]> - Array of Airfield instances
 */
export async function loadAirfieldsFromCSV(url: string): Promise<Airfield[]> {
    const csvText = await fetch(url).then(res => res.text());

    return new Promise((resolve, reject) => {
        Papa.parse(csvText, {
            header: true,
            skipEmptyLines: true,
            complete: (results: any) => {
                console.log(results)
                try {
                    const airfields = results.data.map((row: any) =>
                        new Airfield(
                            row.icao,
                            parseFloat(row.lat),
                            parseFloat(row.lon),
                            row.name
                        )
                    );
                    resolve(airfields);
                } catch (err) {
                    reject(`Failed to parse airfields: ${err}`);
                }
            },
            error: reject
        });
    });
}

export class Airfield {
    readonly icao: string;
    readonly position: Position;
    readonly name: string;

    constructor(icao: string, lat: number, lon: number, name: string) {
        this.icao = icao;
        this.position = new Position(lat, lon);
        this.name = name;
    }

    /**
     * Draws this airfield on the canvas at its projected XY position.
     *
     * @param ctx - Canvas 2D context
     * @param center - Center projection used to convert geo to screen
     */
    draw(ctx: CanvasRenderingContext2D, center: Center): void {
        const xy = center.get_xy(this.position);

        ctx.beginPath();
        ctx.arc(xy.x, xy.y, 4, 0, Math.PI * 2);
        ctx.fillStyle = 'yellow';
        ctx.fill();
        ctx.strokeStyle = 'black';
        ctx.stroke();

        ctx.fillStyle = 'white';
        ctx.fillText(this.icao, xy.x + 6, xy.y - 6);
    }
}
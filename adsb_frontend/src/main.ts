import { Aircraft } from "./aircraft";

const canvas = document.getElementById("display") as HTMLCanvasElement | null;

if (!canvas) {
  throw new Error("Canvas element not found");
}

const ctx = canvas.getContext("2d");

if (!ctx) {
  throw new Error("Could not get 2D context");
}

let aircraft: Aircraft[] = [
    {
        icao: "8723",
        callsign: "ANZ100",
        altitude: 35000,
        latitude: 33.9425,
        longitude: 33.9425
    }
];

aircraft

ctx.fillStyle = "red";
ctx.fillRect(10, 10, 100, 100);


function animate() {
    aircraft.forEach(plane => {
        plane
    });
    }


    requestAnimationFrame(animate);
}

animate();
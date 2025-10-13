/// Implementation of a searchable config that can be interacted with using a search bar

import { PositionXY } from "../position";
import { roundRectTextBox } from "../utils";

export class ConfigBar {
    private bar_size: PositionXY = new PositionXY(0, 0);
    private focused: boolean = false;
    private text: string = "Search ...";

    constructor(
        public center: PositionXY,
        public width_char: number = 30,

    ) {
        window.addEventListener("mousedown", (e) => this.on_click(e));
        window.addEventListener("keydown", (e) => this.handle_key(e));

    }

    private handle_key(e: KeyboardEvent) {
        if (!this.focused) return;

        if (e.key === "Backspace") {
            this.text = this.text.slice(0, -1);
        } else if (e.key === "Enter") {
            this.on_search(this.text);
        } else if (e.key === "Escape") {
            this.focused = false;
        } else if (e.key.length === 1) {
            this.text += e.key;
        }
    }

    private on_search(text: string) {
        console.log(text);
    }

    public draw(ctx: CanvasRenderingContext2D) {
        this.bar_size = roundRectTextBox(ctx, this.center, 60, 10);
        ctx.fillStyle = "#cccccc";
        ctx.fillText(this.text, this.center.x - ctx.measureText(this.text).width / 2, this.center.y);
    }

    private on_click(ev: MouseEvent) {
        if (this.check_hover(ev.offsetX, ev.offsetY)) {
            this.focused = true;
        }
    }


    /**
     * Check if the mouse is over the window
     *
     * @param x the mouse x position
     * @param y the mouse y position
     * @returns true if hovering
     */
    public check_hover(x: number, y: number): boolean {
        return (Math.abs(y - this.center.y) < this.bar_size.y / 2 && Math.abs(x - this.center.x) < this.bar_size.x / 2);
    }
}
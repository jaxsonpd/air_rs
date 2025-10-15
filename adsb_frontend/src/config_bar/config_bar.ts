/// Implementation of a searchable config that can be interacted with using a search bar

import { PositionXY } from "../position";
import { roundRectTextBox, get_text_height } from "../utils";

export class ConfigBar {
    private bar_size: PositionXY = new PositionXY(0, 0);
    private focused: boolean = false;
    private placeholder: string = "Search ...";
    private text: string = "";

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
            this.lose_focus();
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
        if (!this.focused) {
            ctx.fillText(this.placeholder, this.center.x - (ctx.measureText(this.placeholder).width / 2), this.center.y + get_text_height(ctx, "h") /2);
        } else {
            ctx.fillText(this.text, this.center.x - (this.bar_size.x / 2 - ctx.measureText("h").width), this.center.y + get_text_height(ctx, "h") /2);
        }
    }

    private on_click(ev: MouseEvent) {
        if (this.check_hover(ev.offsetX, ev.offsetY)) {
            this.gain_focus();
        } else {
            this.lose_focus();
        }
    }

    private gain_focus() {
        this.focused = true;
    }

    private lose_focus() {
        this.focused = false;
        this.text = "";
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
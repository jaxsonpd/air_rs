/// Implementation of a searchable config that can be interacted with using a search bar

import { PositionXY } from "../position";
import { roundRectTextBox, get_text_height } from "../utils";
import { SearchManager, AutocompleteSuggestion } from "../store/store";

const BAR_SIZE = 60;

export class ConfigBar {
    private bar_size: PositionXY = new PositionXY(0, 0);
    private focused: boolean = false;
    private placeholder: string = "Search ...";
    private text: string = "";
    private search_manager: SearchManager = new SearchManager();

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
        this.search_manager.execute_search(text);
        this.text = "";
        this.lose_focus();
    }

    private draw_coloured_text(ctx: CanvasRenderingContext2D, text_pos: PositionXY, label: string, example_value: string = "") {
        const labelWords = label.split(" ");
        let x = text_pos.x;
        const y = text_pos.y;

        // First word of label — purple
        ctx.fillStyle = "#ff00ffff";
        const firstWord = labelWords[0] ?? "";
        ctx.fillText(firstWord, x, y);
        x += ctx.measureText(firstWord + " ").width;

        // Remaining label — green
        ctx.fillStyle = "#008000";
        const restLabel = labelWords.slice(1).join(" ");
        ctx.fillText(restLabel, x, y);
        x += ctx.measureText(restLabel + " ").width;

        // Example value — grey
        ctx.fillStyle = "#888888";
        ctx.fillText(example_value, x, y);
    }

    public draw(ctx: CanvasRenderingContext2D) {
        if (!this.focused) {
            // NOT FOCUSED — Draw placeholder
            ctx.fillStyle = "#cccccc";
            ctx.strokeStyle = "#cccccc";
            ctx.fillText(
                this.placeholder,
                this.center.x - (ctx.measureText(this.placeholder).width / 2),
                this.center.y + get_text_height(ctx, "h") / 2
            );
            this.bar_size = roundRectTextBox(ctx, this.center);
            return;
        } else {
            // FOCUSED — Draw input + autocomplete box
            let autocompletions: AutocompleteSuggestion[] = this.search_manager.autocomplete_search(this.text);
            autocompletions.sort((a, b) => {
                return b.similarity - a.similarity
            });

            ctx.fillStyle = "#ffffff";
            ctx.strokeStyle = "#ffffff";
            const lineWidth = ctx.lineWidth;
            ctx.lineWidth = 2;

            this.bar_size = roundRectTextBox(ctx, this.center);


            const text_pos = new PositionXY(this.center.x - this.bar_size.x / 2 + ctx.measureText(" ").width, this.center.y + get_text_height(ctx, "h") / 2)
            if (autocompletions.length == 1) {
                const number_words = autocompletions[0].command.split(" ").length;
                const command = this.text.split(" ").slice(0, number_words).join(" ");
                const value = this.text.split(" ").slice(number_words).join(" ")
                this.draw_coloured_text(ctx, text_pos, command, value);
            } else {
                this.draw_coloured_text(ctx, text_pos, this.text);
            }
            if (autocompletions.length > 0) {
                const autocomplete_box_pos: PositionXY = new PositionXY(this.center.x, this.center.y + get_text_height(ctx, "h") * (autocompletions.length / 2 + 2) )
                const autocomplete_size = roundRectTextBox(ctx, autocomplete_box_pos, 60, autocompletions.length);

                let autocomplete_text_pos = new PositionXY(text_pos.x, text_pos.y + get_text_height(ctx, "h") * 2.5)
                autocompletions.forEach((entry) => {
                    this.draw_coloured_text(ctx, autocomplete_text_pos, entry.command, entry.example_value);
                    autocomplete_text_pos.y += get_text_height(ctx, "h")
                });
            }
            ctx.lineWidth = lineWidth;
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
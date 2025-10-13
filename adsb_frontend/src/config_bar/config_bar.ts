/// Implementation of a searchable config that can be interacted with using a search bar

import { PositionXY } from "../position";
import { roundRectTextBox } from "../utils";

export class ConfigBar {
    constructor(
        public center: PositionXY,
        public width_char: number = 30,
    ) { }

    public draw(ctx: CanvasRenderingContext2D) {
        roundRectTextBox(ctx, this.center, 60, 10);
    }
}
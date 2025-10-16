import { PositionXY } from "./position";

/**
 * Get the height of some text on a canvas
 *
 * @param ctx the canvas
 * @param text the text
 *
 * @returns the number of pixels high the text is rendered.
 */
export function get_text_height(ctx: CanvasRenderingContext2D, text: string): number {
    return ctx.measureText(text).actualBoundingBoxAscent + ctx.measureText(text).actualBoundingBoxDescent;
}

/**
 * Create a single width rounded rectangle for text
 * @param ctx the canvas to draw on
 * @param center the center postion of the box
 * @param width_char the number of characters to hold
 * @param heigh_char the height of the box in characters
 * @param radius the radius in pixels of the edge of the box
 *
 * @returns the size of the rectangle including the radius
 */
export function roundRectTextBox(
    ctx: CanvasRenderingContext2D,
    center: PositionXY,
    width_char: number = 60,
    heigh_char: number = 1,
    radius: number = 10,
): PositionXY {
    const height = get_text_height(ctx, "h")*(heigh_char + 1);
    const width = width_char * ctx.measureText("A").width
    const position = new PositionXY(center.x - width / 2, center.y - height / 2);

    ctx.beginPath();
    ctx.moveTo(position.x + radius, position.y);
    ctx.lineTo(position.x + width - radius, position.y);
    ctx.quadraticCurveTo(position.x + width, position.y, position.x + width, position.y + radius);
    ctx.lineTo(position.x + width, position.y + height - radius);
    ctx.quadraticCurveTo(position.x + width, position.y + height, position.x + width - radius, position.y + height);
    ctx.lineTo(position.x + radius, position.y + height);
    ctx.quadraticCurveTo(position.x, position.y + height, position.x, position.y + height - radius);
    ctx.lineTo(position.x, position.y + radius);
    ctx.quadraticCurveTo(position.x, position.y, position.x + radius, position.y);
    ctx.closePath();
    ctx.stroke();

    return new PositionXY(width, height);
}
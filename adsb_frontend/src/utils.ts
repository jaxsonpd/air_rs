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
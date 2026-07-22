import type { ScreenState, ViewRenderer } from "./registry.ts";

// Full-screen alignment / test pattern for setting up physical panels and
// bezels: a pixel grid, edge frame, centre cross, corner L-markers, and a
// bottom colour-bar strip. Redraws on resize so it always fills the display.
export const testPatternView: ViewRenderer = (container: HTMLElement, _screen: ScreenState) => {
  const canvas = document.createElement("canvas");
  canvas.className = "block";
  container.append(canvas);
  const ctx = canvas.getContext("2d");

  const draw = () => {
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const w = container.clientWidth;
    const h = container.clientHeight;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, w, h);

    // Pixel grid every 100px.
    ctx.strokeStyle = "#1e2a38";
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let x = 0; x <= w; x += 100) {
      ctx.moveTo(x + 0.5, 0);
      ctx.lineTo(x + 0.5, h);
    }
    for (let y = 0; y <= h; y += 100) {
      ctx.moveTo(0, y + 0.5);
      ctx.lineTo(w, y + 0.5);
    }
    ctx.stroke();

    // 1px edge frame — must be fully visible if the panel isn't overscanned.
    ctx.strokeStyle = "#29c5ff";
    ctx.lineWidth = 2;
    ctx.strokeRect(1, 1, w - 2, h - 2);

    // Centre cross + diagonals.
    ctx.strokeStyle = "#3ddc84";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(w / 2, 0);
    ctx.lineTo(w / 2, h);
    ctx.moveTo(0, h / 2);
    ctx.lineTo(w, h / 2);
    ctx.moveTo(0, 0);
    ctx.lineTo(w, h);
    ctx.moveTo(w, 0);
    ctx.lineTo(0, h);
    ctx.stroke();

    // Corner L-markers.
    const m = 60;
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 3;
    const corner = (cx: number, cy: number, dx: number, dy: number) => {
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx + dx, cy);
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx, cy + dy);
      ctx.stroke();
    };
    corner(2, 2, m, m);
    corner(w - 2, 2, -m, m);
    corner(2, h - 2, m, -m);
    corner(w - 2, h - 2, -m, -m);

    // Colour bars along the bottom.
    const bars = ["#fff", "#ff0", "#0ff", "#0f0", "#f0f", "#f00", "#00f", "#000"];
    const bw = w / bars.length;
    const bh = Math.max(40, h * 0.08);
    bars.forEach((c, i) => {
      ctx.fillStyle = c;
      ctx.fillRect(i * bw, h - bh, bw, bh);
    });

    // Resolution readout.
    ctx.fillStyle = "#fff";
    ctx.font = "16px ui-monospace, Menlo, monospace";
    ctx.textAlign = "center";
    ctx.fillText(`${w} × ${h} @ ${dpr}x`, w / 2, h / 2 - 16);
  };

  draw();
  window.addEventListener("resize", draw);

  return () => {
    window.removeEventListener("resize", draw);
    canvas.remove();
  };
};

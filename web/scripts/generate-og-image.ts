/**
 * OpenGraph Image Generator
 *
 * Generates a 1200x630 OG image matching the hero section styling.
 *
 * Run with: pnpm run generate:og
 */
import { Resvg } from "@resvg/resvg-js";
import { writeFileSync } from "fs";
import { dirname, join } from "path";
import satori from "satori";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

const WIDTH = 1200;
const HEIGHT = 630;
const GRID_SIZE = 32;

/**
 * Fetches a font from Google Fonts as a TTF file.
 *
 * We use an older user-agent to get TTF instead of woff2.
 *
 * @param family - The font family to fetch.
 * @param weight - The font weight to fetch.
 * @returns A Promise that resolves to the font file as an ArrayBuffer.
 */
async function fetchFont(family: string, weight: number): Promise<ArrayBuffer> {
  const url = `https://fonts.googleapis.com/css2?family=${encodeURIComponent(family)}:wght@${weight}&display=swap`;

  const cssRes = await fetch(url, {
    headers: { "User-Agent": "Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.1; Trident/6.0)" },
  });
  const css = await cssRes.text();

  const fontUrlMatch = css.match(/src: url\(([^)]+)\)/);
  if (!fontUrlMatch) {
    throw new Error(`Could not find font URL for ${family}`);
  }

  const fontRes = await fetch(fontUrlMatch[1]);
  return fontRes.arrayBuffer();
}

/**
 * Generates a grid pattern as SVG.
 *
 * @returns An object representing the grid pattern.
 */
function GridPattern() {
  const lines: { x1: number; y1: number; x2: number; y2: number }[] = [];

  for (let x = 0; x <= WIDTH; x += GRID_SIZE) {
    lines.push({ x1: x, y1: 0, x2: x, y2: HEIGHT });
  }

  for (let y = 0; y <= HEIGHT; y += GRID_SIZE) {
    lines.push({ x1: 0, y1: y, x2: WIDTH, y2: y });
  }

  return {
    type: "div",
    props: {
      style: { position: "absolute", top: 0, left: 0, width: WIDTH, height: HEIGHT, display: "flex" },
      children: {
        type: "svg",
        props: {
          width: WIDTH,
          height: HEIGHT,
          children: lines.map((line, i) => ({
            type: "line",
            props: {
              key: i,
              x1: line.x1,
              y1: line.y1,
              x2: line.x2,
              y2: line.y2,
              stroke: "rgba(85, 85, 85, 0.3)",
              strokeWidth: 1,
            },
          })),
        },
      },
    },
  };
}

function NoteCard(
  config: { x: number; y: number; width: number; bgColor: string; borderColor: string; scribbleColor: string },
) {
  return {
    type: "div",
    props: {
      style: {
        position: "absolute",
        left: config.x,
        top: config.y,
        width: config.width,
        height: 80,
        backgroundColor: config.bgColor,
        border: `1px solid ${config.borderColor}`,
        borderRadius: 8,
        padding: 12,
        display: "flex",
        flexDirection: "column",
        gap: 8,
      },
      children: [{
        type: "div",
        props: { style: { width: "80%", height: 8, backgroundColor: config.scribbleColor, borderRadius: 4 } },
      }, {
        type: "div",
        props: { style: { width: "60%", height: 8, backgroundColor: config.scribbleColor, borderRadius: 4 } },
      }],
    },
  };
}

const cards = [
  NoteCard({
    x: 800,
    y: 60,
    width: 180,
    bgColor: "rgba(37, 99, 235, 0.2)",
    borderColor: "rgba(59, 130, 246, 0.3)",
    scribbleColor: "rgba(147, 197, 253, 0.5)",
  }),
  NoteCard({
    x: 950,
    y: 180,
    width: 160,
    bgColor: "rgba(147, 51, 234, 0.2)",
    borderColor: "rgba(168, 85, 247, 0.3)",
    scribbleColor: "rgba(216, 180, 254, 0.5)",
  }),
  NoteCard({
    x: 820,
    y: 300,
    width: 170,
    bgColor: "rgba(234, 88, 12, 0.2)",
    borderColor: "rgba(251, 146, 60, 0.3)",
    scribbleColor: "rgba(253, 186, 116, 0.5)",
  }),
  NoteCard({
    x: 920,
    y: 420,
    width: 200,
    bgColor: "rgba(14, 116, 144, 0.2)",
    borderColor: "rgba(34, 211, 238, 0.3)",
    scribbleColor: "rgba(103, 232, 249, 0.5)",
  }),
];

const ogImage = {
  type: "div",
  props: {
    style: {
      width: WIDTH,
      height: HEIGHT,
      backgroundColor: "#000000",
      display: "flex",
      flexDirection: "column",
      position: "relative",
    },
    children: [GridPattern(), ...cards, {
      type: "div",
      props: {
        style: { position: "absolute", left: 60, top: 180, display: "flex", flexDirection: "column" },
        children: [{
          type: "div",
          props: {
            style: { fontSize: 96, fontFamily: "Source Serif 4", fontWeight: 500, color: "#ffffff", lineHeight: 1.1 },
            children: "Learning on",
          },
        }, {
          type: "div",
          props: {
            style: { fontSize: 96, fontFamily: "Source Serif 4", fontWeight: 500, color: "#737373", lineHeight: 1.1 },
            children: "the AT Protocol.",
          },
        }],
      },
    }, {
      type: "div",
      props: {
        style: {
          position: "absolute",
          left: 60,
          bottom: 50,
          fontSize: 48,
          fontFamily: "Figtree",
          fontWeight: 600,
          color: "#ffffff",
        },
        children: "Malfestio",
      },
    }, {
      type: "div",
      props: {
        style: {
          position: "absolute",
          right: 72,
          bottom: 50,
          fontSize: 32,
          fontFamily: "Figtree",
          fontWeight: 500,
          textShadow: "0px 0px 10px rgba(0, 0, 0, 0.5)",
          color: "#737373",
        },
        children: "malfestio.stormlightlabs.org",
      },
    }],
  },
};

async function main() {
  console.log("Generating OpenGraph image...");
  console.log("Fetching fonts from Google Fonts...");

  const [sourceSerif, figtree] = await Promise.all([fetchFont("Source Serif 4", 500), fetchFont("Figtree", 600)]);

  console.log("Rendering SVG...");

  const svg = await satori(ogImage, {
    width: WIDTH,
    height: HEIGHT,
    fonts: [{ name: "Source Serif 4", data: sourceSerif, weight: 500, style: "normal" }, {
      name: "Figtree",
      data: figtree,
      weight: 600,
      style: "normal",
    }],
  });

  console.log("Converting to PNG...");

  const resvg = new Resvg(svg, { fitTo: { mode: "width", value: WIDTH } });
  const pngBuffer = resvg.render().asPng();
  const outputPath = join(__dirname, "..", "public", "og-image.png");
  writeFileSync(outputPath, pngBuffer);

  console.log(`✓ Generated: ${outputPath}`);
  console.log(`  Dimensions: ${WIDTH}x${HEIGHT}px`);
}

main().catch(console.error);

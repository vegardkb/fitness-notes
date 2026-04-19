import { chromium, devices } from "playwright";
import sharp from "sharp";
import { mkdir } from "node:fs/promises";

const BASE = "http://localhost:1420";
const OUT = "screenshots";

const routes = [
  { name: "home", path: "/" },
  { name: "calendar", path: "/calendar" },
  { name: "settings", path: "/settings" },
  { name: "body-history", path: "/body/history" },
  { name: "body-day", path: "/body/2026-04-19" },
  { name: "exercises-day", path: "/exercises/2026-04-19" },
  { name: "templates-day", path: "/templates/2026-04-19" },
  { name: "exercise-history", path: "/exercise/1/history" },
  { name: "exercise-prs", path: "/exercise/1/prs" },
  { name: "exercise-graph", path: "/exercise/1/graph" },
  { name: "exercise-entry", path: "/exercise/1/1" },
];

await mkdir(OUT, { recursive: true });
const browser = await chromium.launch();

const profiles = [
  { label: "pixel-7", options: { ...devices["Pixel 7"] } },
  {
    label: "iphone-x",
    options: { ...devices["iPhone X"] },
  },
];

for (const profile of profiles) {
  const ctx = await browser.newContext(profile.options);
  const page = await ctx.newPage();

  const shots = [];
  console.log(profile.label);
  for (const { name, path } of routes) {
    await page.goto(BASE + path, { waitUntil: "networkidle" });
    const file = `${OUT}/${profile.label}-${name}.png`;
    await page.screenshot({ path: file, fullPage: false });
    shots.push({ name, file });
    console.log("✓", name);
  }

  const loaded = await Promise.all(
    shots.map(async (s) => ({
      ...s,
      img: sharp(s.file),
      meta: await sharp(s.file).metadata(),
    })),
  );
  const width = Math.max(...loaded.map((s) => s.meta.width));
  const gap = 40;
  const labelH = 30;
  const totalH = loaded.reduce((h, s) => h + s.meta.height + gap + labelH, 0);

  const composites = [];
  let y = 0;
  for (const s of loaded) {
    const label = Buffer.from(
      `<svg width="${width}" height="${labelH}"><rect width="100%" height="100%" fill="#222"/>
       <text x="10" y="20" font-family="monospace" font-size="14" fill="#fff">${s.name}</text></svg>`,
    );
    composites.push({ input: label, top: y, left: 0 });
    y += labelH;
    composites.push({ input: s.file, top: y, left: 0 });
    y += s.meta.height + gap;
  }

  await sharp({
    create: { width, height: totalH, channels: 4, background: "#fff" },
  })
    .composite(composites)
    .png()
    .toFile(`${OUT}/${profile.label}-contact-sheet.png`);
  console.log("→ screenshots/", profile.label, "-contact-sheet.png");
}
await browser.close();

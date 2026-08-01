// Generates design/overview-bg.svg — navy + amber hero background for the Overview screen.
// Motif matches the app icon: an isometric "cube hub" with a connected node network.
// Seeded RNG so re-runs are stable. Render with sharp (preserves alpha / crisp gradients).
const fs = require('fs');

const W = 1920, H = 1080;

// --- seeded PRNG (mulberry32) ---
let seed = 0x9e3779b9;
const rand = () => {
  seed |= 0; seed = (seed + 0x6d2b79f5) | 0;
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};
const rng = (a, b) => a + (b - a) * rand();

// --- hero cube position ---
const HX = 590, HY = 560, CS = 92; // center + cube half-width

// --- scatter nodes across the canvas, avoiding the hero core ---
const nodes = [];
const tries = 260;
for (let i = 0; i < tries && nodes.length < 26; i++) {
  const x = rng(90, W - 90), y = rng(90, H - 90);
  if (Math.hypot(x - HX, y - HY) < 170) continue;          // keep clear of hero cube
  let ok = true;
  for (const n of nodes) if (Math.hypot(x - n.x, y - n.y) < 150) { ok = false; break; }
  if (ok) nodes.push({ x, y, r: rng(3.5, 7) });
}
// hero acts as a network node too
const hero = { x: HX, y: HY, r: 0, hero: true };
const all = [hero, ...nodes];

// amber-accent a few nodes near the hero for a warm focal cluster
const byDist = [...nodes].sort((a, b) => Math.hypot(a.x - HX, a.y - HY) - Math.hypot(b.x - HX, b.y - HY));
const amber = new Set(byDist.slice(0, 3));

// --- connections: link nearby nodes ---
const D = 340;
const links = [];
for (let i = 0; i < all.length; i++)
  for (let j = i + 1; j < all.length; j++) {
    const d = Math.hypot(all[i].x - all[j].x, all[i].y - all[j].y);
    if (d < D) links.push({ a: all[i], b: all[j], op: (1 - d / D) * 0.5 });
  }

// --- faint large background cubes for depth ---
function ghostCube(cx, cy, s, op) {
  const t = `${cx},${cy - s * 0.86}`, r = `${cx + s},${cy - s * 0.28}`,
        b = `${cx},${cy + s * 0.3}`, l = `${cx - s},${cy - s * 0.28}`,
        bl = `${cx - s},${cy + s * 0.58}`, bc = `${cx},${cy + s * 1.16}`, br = `${cx + s},${cy + s * 0.58}`;
  return `<g opacity="${op}" fill="none" stroke="#4A6C9E" stroke-width="2">
    <path d="M${t} L${r} L${b} L${l} Z"/>
    <path d="M${l} L${b} L${bc} L${bl} Z"/>
    <path d="M${r} L${b} L${bc} L${br} Z"/></g>`;
}

// --- hero cube (isometric, amber top) ---
function heroCube(cx, cy, s) {
  const t = `${cx},${cy - s * 0.86}`, r = `${cx + s},${cy - s * 0.28}`,
        b = `${cx},${cy + s * 0.3}`, l = `${cx - s},${cy - s * 0.28}`,
        bl = `${cx - s},${cy + s * 0.58}`, bc = `${cx},${cy + s * 1.16}`, br = `${cx + s},${cy + s * 0.58}`;
  return `<g filter="url(#cubeShadow)">
    <path d="M${r} L${b} L${bc} L${br} Z" fill="url(#gRight)"/>
    <path d="M${l} L${b} L${bc} L${bl} Z" fill="url(#gLeft)"/>
    <path d="M${t} L${r} L${b} L${l} Z" fill="url(#gTop)"/>
    <path d="M${t} L${b}" stroke="#fff" stroke-opacity="0.16" stroke-width="2.5"/>
    <path d="M${l} L${r}" stroke="#fff" stroke-opacity="0.10" stroke-width="2"/>
  </g>`;
}

const linkSvg = links.map(l =>
  `<line x1="${l.a.x.toFixed(1)}" y1="${l.a.y.toFixed(1)}" x2="${l.b.x.toFixed(1)}" y2="${l.b.y.toFixed(1)}" stroke="#5A7CAE" stroke-opacity="${l.op.toFixed(3)}" stroke-width="1.4"/>`
).join('\n    ');

const nodeSvg = nodes.map(n => {
  const isA = amber.has(n);
  const fill = isA ? '#F59E0B' : 'url(#gNode)';
  const glow = isA ? `<circle cx="${n.x.toFixed(1)}" cy="${n.y.toFixed(1)}" r="${(n.r*4).toFixed(1)}" fill="#F59E0B" opacity="0.12"/>` : '';
  return `${glow}<circle cx="${n.x.toFixed(1)}" cy="${n.y.toFixed(1)}" r="${n.r.toFixed(1)}" fill="${fill}"/>`;
}).join('\n    ');

const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="${W}" y2="${H}" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#0C1A2E"/>
      <stop offset="0.5" stop-color="#0A1526"/>
      <stop offset="1" stop-color="#060D18"/>
    </linearGradient>
    <radialGradient id="amberGlow" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#F59E0B" stop-opacity="0.30"/>
      <stop offset="0.6" stop-color="#F59E0B" stop-opacity="0.06"/>
      <stop offset="1" stop-color="#F59E0B" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="blueGlow" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#3B82F6" stop-opacity="0.18"/>
      <stop offset="1" stop-color="#3B82F6" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="vignette" cx="0.5" cy="0.5" r="0.75">
      <stop offset="0.55" stop-color="#000000" stop-opacity="0"/>
      <stop offset="1" stop-color="#000000" stop-opacity="0.45"/>
    </radialGradient>
    <linearGradient id="gTop" x1="${HX-CS}" y1="${HY-CS}" x2="${HX+CS}" y2="${HY}" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#FCD34D"/><stop offset="0.5" stop-color="#F59E0B"/><stop offset="1" stop-color="#D97706"/>
    </linearGradient>
    <linearGradient id="gLeft" x1="${HX-CS}" y1="${HY}" x2="${HX}" y2="${HY+CS*1.2}" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#3B5578"/><stop offset="1" stop-color="#243B5A"/>
    </linearGradient>
    <linearGradient id="gRight" x1="${HX+CS}" y1="${HY}" x2="${HX}" y2="${HY+CS*1.2}" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#2C466A"/><stop offset="1" stop-color="#182C46"/>
    </linearGradient>
    <radialGradient id="gNode" cx="0.35" cy="0.3" r="0.8">
      <stop offset="0" stop-color="#A9C4E8"/><stop offset="1" stop-color="#5476A2"/>
    </radialGradient>
    <pattern id="dots" width="46" height="46" patternUnits="userSpaceOnUse">
      <circle cx="2" cy="2" r="1.4" fill="#4A6C9E" fill-opacity="0.16"/>
    </pattern>
    <filter id="cubeShadow" x="-60%" y="-60%" width="220%" height="220%">
      <feDropShadow dx="0" dy="18" stdDeviation="26" flood-color="#000" flood-opacity="0.45"/>
    </filter>
    <filter id="soft" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="1.2"/>
    </filter>
  </defs>

  <rect width="${W}" height="${H}" fill="url(#bg)"/>
  <rect width="${W}" height="${H}" fill="url(#dots)"/>

  <!-- ambient glows -->
  <ellipse cx="${HX+40}" cy="${HY-20}" rx="620" ry="520" fill="url(#amberGlow)"/>
  <ellipse cx="${W-380}" cy="300" rx="560" ry="480" fill="url(#blueGlow)"/>

  <!-- depth: ghost cubes -->
  ${ghostCube(W - 300, H - 160, 210, 0.10)}
  ${ghostCube(1500, 210, 130, 0.08)}

  <!-- network -->
  <g filter="url(#soft)">
    ${linkSvg}
  </g>
  <g>
    ${nodeSvg}
  </g>

  <!-- hero cube hub -->
  ${heroCube(HX, HY, CS)}
  <circle cx="${HX}" cy="${HY-CS*1.6}" r="9" fill="#FCD34D"/>

  <!-- vignette for focus -->
  <rect width="${W}" height="${H}" fill="url(#vignette)"/>
</svg>`;

fs.writeFileSync('design/overview-bg.svg', svg);
console.log('wrote design/overview-bg.svg with', nodes.length, 'nodes,', links.length, 'links');

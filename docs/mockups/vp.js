// Draws a small fake scene into each element with data-vp="persp|top|front|left".
// Mockup only: a box (selected), a cylinder and a sphere on a grid, Z up.
(function () {
  const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
  const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const norm = a => { const l = Math.hypot(...a); return [a[0] / l, a[1] / l, a[2] / l]; };

  function box(x, y, w, d, h) {
    const v = [];
    for (const z of [0, h]) for (const [a, b] of [[-1, -1], [1, -1], [1, 1], [-1, 1]]) v.push([x + a * w / 2, y + b * d / 2, z]);
    return { v, f: [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]] };
  }
  function cyl(x, y, r, h, n) {
    const v = [], f = [], bot = [], top = [];
    for (const z of [0, h]) for (let i = 0; i < n; i++) { const t = i / n * Math.PI * 2; v.push([x + r * Math.cos(t), y + r * Math.sin(t), z]); }
    for (let i = 0; i < n; i++) { const j = (i + 1) % n; f.push([i, j, n + j, n + i]); bot.unshift(i); top.push(n + i); }
    f.push(bot, top);
    return { v, f };
  }
  function sphere(x, y, z, r, la, lo) {
    const v = [], f = [];
    for (let i = 0; i <= la; i++) for (let j = 0; j < lo; j++) {
      const p = i / la * Math.PI, t = j / lo * Math.PI * 2;
      v.push([x + r * Math.sin(p) * Math.cos(t), y + r * Math.sin(p) * Math.sin(t), z + r * Math.cos(p)]);
    }
    for (let i = 0; i < la; i++) for (let j = 0; j < lo; j++) {
      const k = (j + 1) % lo;
      f.push([i * lo + j, (i + 1) * lo + j, (i + 1) * lo + k, i * lo + k]);
    }
    return { v, f };
  }

  const scene = [
    { m: box(-28, 4, 40, 40, 30), sel: true, tone: 0 },
    { m: cyl(38, 26, 14, 46, 20), tone: 1 },
    { m: sphere(16, -42, 16, 16, 10, 16), tone: 2 },
  ];

  const views = {
    persp: { dir: norm([1.1, -1.5, 0.95]), up: [0, 0, 1], persp: true },
    top: { dir: [0, 0, 1], up: [0, 1, 0] },
    front: { dir: [0, -1, 0], up: [0, 0, 1] },
    left: { dir: [-1, 0, 0], up: [0, 0, 1] },
  };

  function draw(el, t) {
    const kind = el.dataset.vp, V = views[kind];
    const W = el.clientWidth, H = el.clientHeight, cx = W / 2, cy = H / 2;
    const eye = V.dir.map(c => c * 300);
    const f = norm(sub([0, 0, 0], eye)), r = norm(cross(f, V.up)), u = cross(r, f);
    const scale = V.persp ? Math.min(W, H) * 1.7 : Math.min(W, H) / 150;
    const P = p => {
      const d = sub(p, eye), x = dot(d, r), y = dot(d, u), z = dot(d, f);
      return V.persp ? [cx + x * scale / z, cy - y * scale / z, z] : [cx + x * scale, cy - y * scale, z];
    };
    const pts = a => a.map(p => P(p).slice(0, 2).map(n => n.toFixed(1)).join(',')).join(' ');
    let s = '';
    const G = V.persp ? 240 : 300;
    for (let i = -G; i <= G; i += 10) {
      const major = i === 0 ? t.gridMajor : t.grid;
      const ga = V.dir[2] ? [[i, -G, 0], [i, G, 0], [-G, i, 0], [G, i, 0]]
        : V.dir[1] ? [[i, 0, -G], [i, 0, G], [-G, 0, i], [G, 0, i]]
        : [[0, i, -G], [0, i, G], [0, -G, i], [0, G, i]];
      s += `<polyline points="${pts(ga.slice(0, 2))}" stroke="${major}" fill="none"/><polyline points="${pts(ga.slice(2))}" stroke="${major}" fill="none"/>`;
    }
    const light = norm([V.dir[0] + 0.25, V.dir[1] - 0.2, V.dir[2] + 0.6]);
    if (el.dataset.shade !== undefined) {
      const faces = [];
      for (const o of scene) for (const fc of o.m.f) {
        const a = o.m.v[fc[0]], b = o.m.v[fc[1]], c = o.m.v[fc[2]];
        const n = norm(cross(sub(b, a), sub(c, a)));
        const mid = fc.reduce((m, i) => m.map((q, k) => q + o.m.v[i][k] / fc.length), [0, 0, 0]);
        if (dot(n, sub(eye, mid)) <= 0) continue;
        faces.push({ o, poly: fc.map(i => o.m.v[i]), z: P(mid)[2], lit: 0.45 + 0.55 * Math.max(0, dot(n, light)) });
      }
      faces.sort((a, b) => b.z - a.z);
      if (t.halo) for (const fc of faces) if (fc.o.sel) s += `<polygon points="${pts(fc.poly)}" fill="${t.halo}" stroke="${t.halo}" stroke-width="4" stroke-linejoin="round"/>`;
      for (const fc of faces) {
        const base = fc.o.sel && t.selTint ? t.selTint : t.shade[fc.o.tone];
        const col = `rgb(${base.map(c => Math.round(c * fc.lit)).join(',')})`;
        s += `<polygon points="${pts(fc.poly)}" fill="${col}" stroke="${fc.o.sel && !t.halo ? t.sel : col}" stroke-width="${fc.o.sel && !t.halo ? 1.2 : 0.6}"/>`;
      }
      if (V.persp && t.plane) {
        const q = [[0, 0], [9, 0], [9, 9], [0, 9]].map(([a, b]) => [-28 + a, 4 + b, 30]);
        s += `<polygon points="${pts(q)}" fill="${t.plane}" fill-opacity=".35" stroke="${t.plane}"/>`;
      }
      if (V.persp && t.brackets) {
        const c = [], k = 4;
        for (const x of [-48, -8]) for (const y of [-16, 24]) for (const z of [0, 30]) c.push([x, y, z]);
        for (const p of c) for (const [i, d] of [[0, p[0] < 0 && p[0] < -20 ? k : -k], [1, p[1] < 4 ? k : -k], [2, p[2] ? -k : k]]) {
          const e = p.slice(); e[i] += d;
          const [x1, y1] = P(p), [x2, y2] = P(e);
          s += `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="#fff" stroke-width="1.2"/>`;
        }
      }
      if (V.persp && t.axis) {
      const o = [-28, 4, 30], L = 26, ax = [[[L, 0, 0], t.axis[0]], [[0, L, 0], t.axis[1]], [[0, 0, L], t.axis[2]]];
      for (const [d, c] of ax) {
        const [x1, y1] = P(o), [x2, y2] = P([o[0] + d[0], o[1] + d[1], o[2] + d[2]]);
        s += `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${c}" stroke-width="2.4"/>${t.plane ? '' : `<circle cx="${x2}" cy="${y2}" r="3.6" fill="${c}"/>`}`;
      }
      }
    } else {
      for (const o of scene) for (const fc of o.m.f)
        s += `<polygon points="${pts(fc.map(i => o.m.v[i]))}" fill="none" stroke="${o.sel ? t.sel : t.wire[o.tone]}" stroke-width="1"/>`;
    }
    if (t.tripod) {
      const ox = 22, oy = H - 22;
      [[[1, 0, 0], '#d83c3c', 'x'], [[0, 1, 0], '#3cb83c', 'y'], [[0, 0, 1], '#4a6cf0', 'z']].forEach(([a, c, n]) => {
        const dx = dot(a, r), dy = dot(a, u);
        if (Math.hypot(dx, dy) < 0.05) return;
        const x2 = ox + dx * 16, y2 = oy - dy * 16;
        s += `<line x1="${ox}" y1="${oy}" x2="${x2}" y2="${y2}" stroke="${c}" stroke-width="1.5"/><text x="${x2 + dx * 6}" y="${y2 - dy * 6 + 3}" font-size="9" fill="${c}" text-anchor="middle">${n}</text>`;
      });
    }
    el.innerHTML = `<svg width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" style="display:block">${s}</svg>` + el.innerHTML;
  }

  window.drawViewports = function (theme) {
    document.querySelectorAll('[data-vp]').forEach(el => draw(el, theme));
  };
})();

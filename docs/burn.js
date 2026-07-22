/* Phosphor burn: continuously animating oscilloscope trace behind the hero
   meter. Canvas 2D persistence sim — each frame decays the previous image
   toward the well color, then adds three mirrored envelope traces in the
   site's amber ramp. Pauses when the tab is hidden or the hero is scrolled
   away; prefers-reduced-motion gets a single static burn frame. */
(function () {
  "use strict";

  var canvas = document.getElementById("burn");
  if (!canvas) return;
  var ctx = canvas.getContext("2d", { alpha: true });
  if (!ctx) return;

  var DPR_CAP = 1.5;
  var FRAME_MS = 33; // ~30fps; persistence hides the lower rate
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  var w = 0, h = 0, dpr = 1;

  function resize() {
    var rect = canvas.getBoundingClientRect();
    dpr = Math.min(DPR_CAP, window.devicePixelRatio || 1);
    w = Math.max(1, Math.floor(rect.width * dpr));
    h = Math.max(1, Math.floor(rect.height * dpr));
    canvas.width = w;
    canvas.height = h;
    ctx.clearRect(0, 0, w, h);
  }

  // Envelope of one band at normalized x in [0,1]: lobes that swell and
  // pinch, mirrored about the band centerline (the reference-pattern look).
  function envelope(x, t, band) {
    var lobes = Math.abs(Math.sin(x * Math.PI * (3 + band.lobeShift) + t * band.drift));
    var swell = 0.55 + 0.45 * Math.sin(x * Math.PI * 2 - t * band.swellRate + band.phase);
    var taper = Math.sin(x * Math.PI); // fade to points at both edges
    return (0.12 + 0.88 * lobes * swell) * taper;
  }

  var bands = [
    { cy: 0.16, amp: 0.09, drift: 0.51, swellRate: 0.33, lobeShift: 0.0, phase: 0.0 },
    { cy: 0.50, amp: 0.12, drift: 0.43, swellRate: 0.27, lobeShift: 0.5, phase: 2.1 },
    { cy: 0.84, amp: 0.09, drift: 0.58, swellRate: 0.38, lobeShift: 0.0, phase: 4.2 }
  ];

  function tracePath(band, t, scale) {
    var steps = 96;
    var cy = band.cy * h;
    var ampPx = band.amp * h * scale;
    ctx.beginPath();
    for (var i = 0; i <= steps; i++) {
      var x = i / steps;
      var a = envelope(x, t, band) * ampPx;
      var px = x * w;
      if (i === 0) ctx.moveTo(px, cy - a); else ctx.lineTo(px, cy - a);
    }
    for (var j = steps; j >= 0; j--) {
      var x2 = j / steps;
      ctx.lineTo(x2 * w, band.cy * h + envelope(x2, t, band) * ampPx);
    }
    ctx.closePath();
  }

  function drawFrame(t) {
    // decay: pull the accumulated image toward the well color
    ctx.globalCompositeOperation = "source-over";
    ctx.fillStyle = "rgba(6, 7, 8, 0.11)";
    ctx.fillRect(0, 0, w, h);

    ctx.globalCompositeOperation = "lighter";
    for (var i = 0; i < bands.length; i++) {
      var band = bands[i];
      // glow pass: wide amber halo
      ctx.shadowColor = "rgba(255, 180, 84, 0.5)";
      ctx.shadowBlur = 14 * dpr;
      ctx.fillStyle = "rgba(255, 180, 84, 0.045)";
      tracePath(band, t, 1);
      ctx.fill();
      // core pass: narrower, hotter
      ctx.shadowBlur = 6 * dpr;
      ctx.fillStyle = "rgba(255, 226, 178, 0.04)";
      tracePath(band, t, 0.55);
      ctx.fill();
    }
    ctx.shadowBlur = 0;
  }

  var raf = 0;
  var running = false;
  var visible = true;
  var onScreen = true;
  var last = 0;
  var t = 0;

  function loop(now) {
    raf = requestAnimationFrame(loop);
    if (now - last < FRAME_MS) return;
    t += (now - last) / 1000;
    last = now;
    drawFrame(t);
  }

  function start() {
    if (running || reduced || !visible || !onScreen) return;
    running = true;
    last = performance.now();
    raf = requestAnimationFrame(loop);
  }

  function stop() {
    running = false;
    cancelAnimationFrame(raf);
  }

  function staticBurn() {
    // settle the persistence buffer once, no animation afterwards
    for (var i = 0; i < 70; i++) drawFrame(i * 0.033);
  }

  resize();
  window.addEventListener("resize", function () {
    resize();
    if (!running) staticBurn();
  });
  if (reduced) {
    staticBurn();
  } else {
    document.addEventListener("visibilitychange", function () {
      visible = !document.hidden;
      if (visible) start(); else stop();
    });
    if ("IntersectionObserver" in window) {
      new IntersectionObserver(function (entries) {
        onScreen = entries[0].isIntersecting;
        if (onScreen) start(); else stop();
      }).observe(canvas);
    }
    start();
  }
})();

/* Renders docs/data/gain.json (a committed snapshot of `stk gain --json`,
   plus RTK numbers and a timestamp) into the meter. Static fallback in the
   HTML is the cold-start state, so a failed fetch degrades gracefully. */
(function () {
  "use strict";

  function fmt(n) {
    return Number(n).toLocaleString("en-US");
  }

  function fmtBytes(n) {
    if (n >= 1048576) return (n / 1048576).toFixed(1) + " MB";
    if (n >= 1024) return (n / 1024).toFixed(1) + " KB";
    return fmt(n) + " B";
  }

  function setText(id, text) {
    var el = document.getElementById(id);
    if (el) el.textContent = text;
  }

  function render(data) {
    var stk = data.stk || {};
    var live = (stk.clamps || 0) + (stk.dup_hits || 0) > 0;

    setText("r-clamps", fmt(stk.clamps || 0));
    setText("r-dups", fmt(stk.dup_hits || 0));
    setText("r-bytes", fmtBytes(stk.bytes_avoided || 0));
    setText("r-tokens", fmt(stk.est_tokens || 0));
    if (data.generated_at) {
      setText("r-updated", data.generated_at.slice(0, 10));
    }

    if (live) {
      setText("meter-number", fmt(stk.est_tokens || 0));
      setText("meter-state", "LIVE");
      setText("meter-kicker", "MEASURED ON THE AUTHOR'S MACHINE · UPDATED DAILY");
      var sub = document.getElementById("meter-sub");
      if (sub) sub.hidden = true;
      var led = document.getElementById("meter-led");
      if (led) led.classList.add("is-live");
      var state = document.getElementById("meter-state");
      if (state) state.classList.add("is-live");
      var kicker = document.getElementById("meter-kicker");
      if (kicker) kicker.classList.add("is-live");
    }

    // per-day sparkline: bytes_avoided per day, last 60 days present in data
    var days = (stk.days || []).slice(-60);
    var spark = document.getElementById("spark");
    if (spark && days.length > 1) {
      var max = Math.max.apply(null, days.map(function (d) { return d.bytes_avoided || 0; }));
      if (max > 0) {
        days.forEach(function (d) {
          var bar = document.createElement("span");
          var h = Math.max(2, Math.round(((d.bytes_avoided || 0) / max) * 44));
          bar.style.height = h + "px";
          bar.title = d.date + " · " + fmtBytes(d.bytes_avoided || 0) + " avoided";
          spark.appendChild(bar);
        });
        spark.setAttribute("aria-label", "Bytes avoided per day, last " + days.length + " days");
        spark.removeAttribute("aria-hidden");
        spark.setAttribute("role", "img");
      }
    }

    var rtk = data.rtk;
    if (rtk) {
      if (rtk.commands != null) setText("rtk-cmds", fmt(rtk.commands));
      if (rtk.tokens_saved != null) setText("rtk-saved", rtk.tokens_saved);
      if (rtk.reduction != null) setText("rtk-pct", rtk.reduction);
    }
  }

  fetch("data/gain.json", { cache: "no-cache" })
    .then(function (r) { if (!r.ok) throw new Error(r.status); return r.json(); })
    .then(render)
    .catch(function () { /* cold-start HTML already says the honest thing */ });
})();

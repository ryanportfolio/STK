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

  // "2026-07-23" -> "7/23". Split, not Date, so the label never shifts a day
  // in timezones behind UTC.
  function monthDay(iso) {
    var p = String(iso).split("-");
    if (p.length < 3) return String(iso);
    return Number(p[1]) + "/" + Number(p[2]);
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
      setText("meter-number", String(stk.est_tokens || 0));
      setText("meter-state", "LIVE");
      var sub = document.getElementById("meter-sub");
      if (sub) sub.hidden = true;
      var led = document.getElementById("meter-led");
      if (led) led.classList.add("is-live");
      var state = document.getElementById("meter-state");
      if (state) state.classList.add("is-live");
    }

    // per-day sparkline: bytes_avoided per day, last 60 days present in data
    var days = (stk.days || []).slice(-60);
    var spark = document.getElementById("spark");
    if (spark && days.length > 1) {
      var max = Math.max.apply(null, days.map(function (d) { return d.bytes_avoided || 0; }));
      if (max > 0) {
        var ticks = [];
        days.forEach(function (d) {
          var col = document.createElement("span");
          col.className = "spark-col";
          col.title = d.date + " · " + fmtBytes(d.bytes_avoided || 0) + " avoided";

          var bar = document.createElement("span");
          bar.className = "spark-bar";
          var h = Math.max(2, Math.round(((d.bytes_avoided || 0) / max) * 44));
          bar.style.height = h + "px";
          col.appendChild(bar);

          var day = document.createElement("span");
          day.className = "spark-day";
          col.appendChild(day);
          ticks.push(day);

          spark.appendChild(col);
        });

        // Only as many month/day ticks as the rendered width fits, so they
        // never collide. Newest day always gets one; the step walks back from
        // it. Re-run on resize since the fit depends on measured width.
        function labelTicks() {
          var fits = Math.max(2, Math.floor(spark.clientWidth / 44));
          var step = Math.ceil(days.length / fits);
          ticks.forEach(function (tick, i) {
            var show = (days.length - 1 - i) % step === 0;
            tick.textContent = show ? monthDay(days[i].date) : "";
          });
        }
        labelTicks();
        var relabel;
        window.addEventListener("resize", function () {
          clearTimeout(relabel);
          relabel = setTimeout(labelTicks, 150);
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

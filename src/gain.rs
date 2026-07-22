//! `stk gain` — aggregate stats: clamps, dup hits, bytes avoided, est. tokens.

use crate::store::Store;
use serde_json::json;

/// Days-from-unix-epoch -> (y, m, d). Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn iso_date(ts_secs: u64) -> String {
    let (y, m, d) = civil_from_days((ts_secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Machine-readable aggregate for dashboards: totals + per-day series.
pub fn report_json(store: &Store) -> String {
    let stats = store.read_stats();
    let clamps = stats.iter().filter(|s| s.kind == "clamp").count();
    let dups = stats.iter().filter(|s| s.kind == "dup").count();
    let bytes_avoided: u64 = stats
        .iter()
        .map(|s| s.file_bytes.saturating_sub(s.sent_bytes))
        .sum();

    let mut days: Vec<(String, u64, u64, u64)> = Vec::new(); // date, clamps, dups, bytes
    for s in &stats {
        let date = iso_date(s.ts);
        let avoided = s.file_bytes.saturating_sub(s.sent_bytes);
        match days.iter_mut().find(|d| d.0 == date) {
            Some(d) => {
                if s.kind == "clamp" { d.1 += 1 } else { d.2 += 1 }
                d.3 += avoided;
            }
            None => days.push((
                date,
                (s.kind == "clamp") as u64,
                (s.kind == "dup") as u64,
                avoided,
            )),
        }
    }
    days.sort_by(|a, b| a.0.cmp(&b.0));

    json!({
        "clamps": clamps,
        "dup_hits": dups,
        "bytes_avoided": bytes_avoided,
        "est_tokens": bytes_avoided / 4,
        "caveat": "re-fetch follow-ups after a clamp are not measurable here; real savings are somewhat lower",
        "days": days.iter().map(|(date, c, du, b)| json!({
            "date": date, "clamps": c, "dup_hits": du, "bytes_avoided": b
        })).collect::<Vec<_>>(),
    })
    .to_string()
}

pub fn report(store: &Store) -> String {
    let stats = store.read_stats();
    let clamps = stats.iter().filter(|s| s.kind == "clamp").count();
    let dups = stats.iter().filter(|s| s.kind == "dup").count();
    let bytes_avoided: u64 = stats
        .iter()
        .map(|s| s.file_bytes.saturating_sub(s.sent_bytes))
        .sum();
    let est_tokens = bytes_avoided / 4;

    format!(
        "stk gain\n--------\nclamps:        {clamps}\ndup hits:      {dups}\nbytes avoided: {bytes_avoided}\nest. tokens:   {est_tokens} (bytes/4)\n\nCaveat: re-fetch follow-ups (extra scoped Reads after a clamp) are not\nmeasurable from here; real savings are somewhat lower than the raw number."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::StatRecord;
    use crate::testutil::tempdir;

    #[test]
    fn aggregates_clamps_and_dups() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        store
            .record_stat(&StatRecord { ts: 1, file: "a".into(), file_bytes: 10000, sent_bytes: 2000, kind: "clamp".into() })
            .unwrap();
        store
            .record_stat(&StatRecord { ts: 2, file: "a".into(), file_bytes: 10000, sent_bytes: 150, kind: "dup".into() })
            .unwrap();
        let out = report(&store);
        assert!(out.contains("clamps:        1"));
        assert!(out.contains("dup hits:      1"));
        assert!(out.contains("bytes avoided: 17850"));
        assert!(out.contains("est. tokens:   4462"));
        assert!(out.contains("not\nmeasurable from here"));
    }

    #[test]
    fn json_report_totals_and_days() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        // 2026-07-21 00:00:00 UTC = 1784592000
        store
            .record_stat(&StatRecord { ts: 1_784_592_000, file: "a".into(), file_bytes: 10_000, sent_bytes: 2_000, kind: "clamp".into() })
            .unwrap();
        store
            .record_stat(&StatRecord { ts: 1_784_592_100, file: "a".into(), file_bytes: 10_000, sent_bytes: 150, kind: "dup".into() })
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&report_json(&store)).unwrap();
        assert_eq!(v["clamps"], 1);
        assert_eq!(v["dup_hits"], 1);
        assert_eq!(v["bytes_avoided"], 17_850);
        assert_eq!(v["est_tokens"], 4_462);
        assert_eq!(v["days"][0]["date"], "2026-07-21");
        assert_eq!(v["days"][0]["bytes_avoided"], 17_850);
    }

    #[test]
    fn empty_store_reports_zero() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        let out = report(&store);
        assert!(out.contains("clamps:        0"));
        assert!(out.contains("bytes avoided: 0"));
    }
}

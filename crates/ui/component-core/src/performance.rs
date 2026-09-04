use std::time::Duration;

pub const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceRow {
    pub title: String,
    pub subtitle: String,
}

#[derive(Default)]
pub struct PerformanceRows {
    rows: Vec<PerformanceRow>,
}

impl PerformanceRows {
    pub fn refresh(&mut self) -> bool {
        let rows = rows();
        if rows == self.rows {
            return false;
        }
        self.rows = rows;
        true
    }

    pub fn rows(&self) -> &[PerformanceRow] {
        &self.rows
    }
}

pub fn rows() -> Vec<PerformanceRow> {
    let mut snapshot = shrimply_benchmarking::snapshot();
    snapshot
        .timings
        .sort_by_key(|timing| std::cmp::Reverse(timing.average));
    snapshot.counters.sort_by_key(|counter| counter.name);
    let frame = snapshot
        .timings
        .iter()
        .find(|timing| timing.name == "Video / Render request")
        .map(|timing| timing.average);
    let mut rows = Vec::with_capacity(snapshot.timings.len() + snapshot.counters.len());
    for timing in snapshot.timings {
        let percentage = frame
            .filter(|frame| !frame.is_zero())
            .map(|frame| {
                let tenths = timing
                    .average
                    .as_nanos()
                    .saturating_mul(1_000)
                    .checked_div(frame.as_nanos())
                    .unwrap_or_default();
                format!(" · {}.{}% frame", tenths / 10, tenths % 10)
            })
            .unwrap_or_default();
        rows.push(PerformanceRow {
            title: timing.name.to_string(),
            subtitle: shrimply_i18n_core::text_args(
                "Last %{last} · Avg %{average} · Min %{minimum} · Max %{maximum} · %{samples} samples%{percentage}",
                &[
                    ("last", duration_label(timing.last)),
                    ("average", duration_label(timing.average)),
                    ("minimum", duration_label(timing.minimum)),
                    ("maximum", duration_label(timing.maximum)),
                    ("samples", timing.samples.to_string()),
                    ("percentage", percentage),
                ],
            ),
        });
    }
    rows.extend(snapshot.counters.into_iter().map(|counter| PerformanceRow {
        title: counter.name.to_string(),
        subtitle: counter.value.to_string(),
    }));
    rows
}

pub fn clear() {
    shrimply_benchmarking::clear();
}

pub fn report_json() -> String {
    shrimply_benchmarking::report_json()
}

fn duration_label(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros >= 1_000_000 {
        format!(
            "{}.{:02} s",
            micros / 1_000_000,
            micros % 1_000_000 / 10_000
        )
    } else if micros >= 1_000 {
        format!("{}.{:02} ms", micros / 1_000, micros % 1_000 / 10)
    } else {
        format!("{micros} µs")
    }
}

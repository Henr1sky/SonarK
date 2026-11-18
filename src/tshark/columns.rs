use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use crate::actions::FieldTransform;

pub fn transform(output: &str, transform: FieldTransform) -> String {
    let pairs = output.lines().filter_map(|line| {
        let (key, value) = line.split_once('\t')?;
        (!key.is_empty() && !value.is_empty()).then_some((key, value))
    });
    let mut result = String::new();
    match transform {
        FieldTransform::EndpointAddress => {
            for row in output.lines() {
                if let Some(address) = row.split('\t').find(|address| !address.is_empty()) {
                    let _ = writeln!(result, "{address}");
                }
            }
        }
        FieldTransform::MeanByKey | FieldTransform::SumByKey => {
            let mut totals = BTreeMap::new();
            for (key, value) in pairs {
                if let Ok(value) = value.parse::<u64>() {
                    let (sum, count) = totals.entry(key).or_insert((0u64, 0u64));
                    *sum = sum.saturating_add(value);
                    *count += 1;
                }
            }
            if matches!(transform, FieldTransform::SumByKey) {
                let mut totals: Vec<_> = totals.into_iter().collect();
                totals.sort_by(|(a, (a_sum, _)), (b, (b_sum, _))| b_sum.cmp(a_sum).then_with(|| a.cmp(b)));
                for (key, (sum, _)) in totals {
                    let _ = writeln!(result, "{key}\t{sum}");
                }
            } else {
                for (key, (sum, count)) in totals {
                    let _ = writeln!(result, "{key}\t{:.2}", sum as f64 / count as f64);
                }
            }
        }
        FieldTransform::ConflictingValues => {
            let mut values = BTreeMap::<_, BTreeSet<_>>::new();
            for (key, value) in pairs {
                values.entry(key).or_default().insert(value);
            }
            for (key, values) in values.into_iter().filter(|(_, values)| values.len() > 1) {
                let values = values.into_iter().collect::<Vec<_>>().join(", ");
                let _ = writeln!(result, "{key}\t{values}");
            }
        }
        FieldTransform::MacKind => {
            let mut counts = [0; 3];
            for address in output.lines() {
                if address.eq_ignore_ascii_case("ff:ff:ff:ff:ff:ff") {
                    counts[2] += 1;
                } else if let Some(first) = address.split(':').next().and_then(|byte| u8::from_str_radix(byte, 16).ok()) {
                    counts[usize::from(first & 1)] += 1;
                }
            }
            for (label, count) in ["Unicast", "Multicast", "Broadcast"].into_iter().zip(counts) {
                let _ = writeln!(result, "{label}\t{count}");
            }
        }
        FieldTransform::TimeBetween => {
            let mut last = BTreeMap::new();
            for (key, timestamp) in pairs {
                if let Ok(timestamp) = timestamp.parse::<f64>() {
                    if let Some(previous) = last.insert(key, timestamp) {
                        let _ = writeln!(result, "{key}\t{:.6}", timestamp - previous);
                    }
                }
            }
        }
        FieldTransform::UniquePerWindow => {
            let mut addresses = BTreeMap::<i64, BTreeSet<_>>::new();
            for (time, address) in pairs {
                if let Ok(time) = time.parse::<f64>() {
                    let start = ((time / 10.0).floor() as i64).saturating_mul(10);
                    addresses.entry(start).or_default().insert(address);
                }
            }
            for (start, addresses) in addresses {
                let _ = writeln!(result, "{start}\t{}", addresses.len());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflicting_values_require_two_distinct_observations() {
        let output = "a\t01\na\t01\nb\t02\nb\t01\n\t0\n";
        assert_eq!(transform(output, FieldTransform::ConflictingValues), "b\t01, 02\n");
    }
}

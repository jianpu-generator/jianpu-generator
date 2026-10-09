use cloud_outbox::backoff::backoff_ms;

type TestResult = Result<(), String>;

fn expect(actual: u64, expected: u64, label: &str) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected}, got {actual}"))
    }
}

#[test]
fn exact_values_for_failures_one_to_eight() -> TestResult {
    // (failures, exp) -> half = exp / 2; jitter 0 => half, 0.5 => half * 1.5, 1 => exp
    let expectations = [
        (1, 1_000, 1_500, 2_000),
        (2, 2_000, 3_000, 4_000),
        (3, 4_000, 6_000, 8_000),
        (4, 8_000, 12_000, 16_000),
        (5, 16_000, 24_000, 32_000),
        (6, 30_000, 45_000, 60_000),
        (7, 30_000, 45_000, 60_000),
        (8, 30_000, 45_000, 60_000),
    ];
    for (failures, at_zero, at_half, at_one) in expectations {
        expect(
            backoff_ms(failures, 0.0),
            at_zero,
            &format!("n={failures} j=0"),
        )?;
        expect(
            backoff_ms(failures, 0.5),
            at_half,
            &format!("n={failures} j=0.5"),
        )?;
        expect(
            backoff_ms(failures, 1.0),
            at_one,
            &format!("n={failures} j=1"),
        )?;
    }
    Ok(())
}

#[test]
fn cap_holds_for_huge_counts_without_overflow() -> TestResult {
    expect(backoff_ms(u32::MAX, 0.0), 30_000, "max j=0")?;
    expect(backoff_ms(u32::MAX, 1.0), 60_000, "max j=1")?;
    expect(backoff_ms(100, 1.0), 60_000, "100 j=1")
}

#[test]
fn jitter_is_clamped() -> TestResult {
    expect(backoff_ms(1, -3.0), 1_000, "negative")?;
    expect(backoff_ms(1, 7.0), 2_000, "above one")?;
    expect(backoff_ms(1, f64::NAN), 1_000, "nan")
}

#[test]
fn jitter_is_floored() -> TestResult {
    expect(backoff_ms(1, 0.0015), 1_001, "floor")
}

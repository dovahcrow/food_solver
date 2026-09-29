//! The report formatting helpers.

#[path = "../src/format.rs"]
#[allow(dead_code)]
mod format;

#[test]
fn g_matches_c_percent_g() {
    // Expected strings come from C's `%g`.
    let cases = [
        (1796.0900634150437, "1796.09"),
        (27.27579378874549, "27.2758"),
        (13.3804, "13.3804"),
        (618.465, "618.465"),
        (0.0392, "0.0392"),
        (300.0, "300"),
        (1.6750e-9, "1.675e-09"),
        (1.12704, "1.12704"),
        (0.0, "0"),
        (0.980872, "0.980872"),
        (47.2165, "47.2165"),
        (173.095, "173.095"),
        (0.02205, "0.02205"),
    ];
    for (value, expected) in cases {
        assert_eq!(format::g(value), expected, "value {value}");
    }
}

#[test]
fn weight_decimals_follow_magnitude() {
    let cases = [
        // >= 100: whole grams.
        (1796.0900634150437, "1796"),
        (564.1651680898664, "564"),
        (100.0, "100"),
        // 99.96 is below 100, so it takes one decimal and rounds up to 100.0.
        (99.96, "100.0"),
        // 10..100: one decimal.
        (99.4, "99.4"),
        (15.386216432485288, "15.4"),
        (10.0, "10.0"),
        // 1..10: two decimals.
        (9.87, "9.87"),
        (1.0, "1.00"),
        // 0.01..1: two decimals.
        (0.98, "0.98"),
        (0.013, "0.01"),
        // < 0.01: shown as zero.
        (0.009, "0"),
        (0.0, "0"),
    ];
    for (value, expected) in cases {
        assert_eq!(format::weight(value), expected, "value {value}");
    }
}

#[test]
fn weight_is_total_on_negatives_and_infinities() {
    // The solver never returns these, but the helper must not panic.
    assert_eq!(format::weight(-150.0), "-150");
    assert_eq!(format::weight(-3.456), "-3.46");
    assert_eq!(format::weight(f64::INFINITY), "inf");
}

//! The report formatting must match Python's `%g`-family output.

#[path = "../src/format.rs"]
#[allow(dead_code)]
mod format;

#[test]
fn g_matches_python_percent_g() {
    // Expected strings come from Python's f"{value:g}".
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
fn two_significant_matches_python_two_g() {
    // Python printed weights as f"{float(f'{x:.2g}'):.1f}".
    // These are the solver's actual optima, so the two-significant-digit
    // rounding is exercised on the same values the CLI prints.
    let cases = [
        (564.1651680898664, "560.0"),
        (244.99999999999994, "240.0"),
        (961.6387897502321, "960.0"),
        (15.386216432485288, "15.0"),
        (500.00000000000006, "500.0"),
        (0.0, "0.0"),
    ];
    for (value, expected) in cases {
        let rounded = format::two_significant(value);
        assert_eq!(format!("{rounded:.1}"), expected, "value {value}");
    }
}

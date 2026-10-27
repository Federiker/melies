#![allow(clippy::float_cmp, reason = "the pauses and lengths are exact values")]

use super::*;

#[test]
fn the_defaults_are_manuzios() {
    let options = NarrationOptions::default();
    assert_eq!(options.profile, Profile::Standard);
    assert_eq!(options.heading_pause, 0.5);
    assert_eq!(options.closing_pause, 1.5);
    assert_eq!(options.sample_rate, 44_100);
    assert_eq!(options.label, None);
}

#[test]
fn the_standard_profile_keeps_every_setting() {
    let options = NarrationOptions {
        closing_pause: 9.0,
        sample_rate: 22_050,
        ..NarrationOptions::default()
    };
    assert_eq!(options.effective(), options);
}

#[test]
fn the_retail_profile_fixes_the_rate_and_keeps_the_closing_pause_within_the_limits() {
    for (pause, kept) in [(0.2, 1.0), (2.5, 2.5), (7.0, 5.0)] {
        let options = NarrationOptions {
            profile: Profile::Retail,
            closing_pause: pause,
            sample_rate: 22_050,
            ..NarrationOptions::default()
        }
        .effective();
        assert_eq!(options.sample_rate, 44_100);
        assert_eq!(options.closing_pause, kept);
        assert_eq!(options.heading_pause, 0.5);
    }
}

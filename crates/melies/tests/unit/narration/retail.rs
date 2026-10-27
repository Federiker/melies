use super::*;

fn good() -> AudioAnalysis {
    AudioAnalysis {
        codec: "mp3".into(),
        bit_rate: Some(192_000),
        sample_rate: 44_100,
        channels: 1,
        duration_secs: 600.0,
        rms_dbfs: -20.5,
        peak_dbfs: -4.2,
        noise_floor_dbfs: -68.0,
        lead_secs: 1.6,
        tail_secs: 1.7,
    }
}

#[test]
fn a_file_inside_every_limit_has_no_problems() {
    assert_eq!(problems(&good()), Vec::<RetailProblem>::new());
}

#[test]
fn each_limit_is_reported_with_what_was_measured() {
    let mut analysis = good();
    analysis.codec = "aac".into();
    analysis.bit_rate = Some(128_000);
    analysis.sample_rate = 24_000;
    analysis.rms_dbfs = -27.0;
    analysis.peak_dbfs = -1.0;
    analysis.noise_floor_dbfs = -50.0;
    analysis.lead_secs = 0.0;
    analysis.tail_secs = 8.0;
    analysis.duration_secs = 8_000.0;
    assert_eq!(
        problems(&analysis),
        [
            RetailProblem::Format {
                codec: "aac".into()
            },
            RetailProblem::BitRate {
                bits_per_second: 128_000
            },
            RetailProblem::SampleRate { hertz: 24_000 },
            RetailProblem::Level {
                rms_dbfs: -27.0,
                short: false
            },
            RetailProblem::Peak { peak_dbfs: -1.0 },
            RetailProblem::NoiseFloor {
                noise_floor_dbfs: -50.0
            },
            RetailProblem::RoomTone {
                edge: Edge::Start,
                secs: 0.0
            },
            RetailProblem::RoomTone {
                edge: Edge::End,
                secs: 8.0
            },
            RetailProblem::Duration { secs: 8_000.0 },
        ]
    );
    analysis.bit_rate = None;
    assert!(problems(&analysis).contains(&RetailProblem::UnknownBitRate));
}

#[test]
fn the_limits_have_a_little_tolerance() {
    let mut analysis = good();
    analysis.bit_rate = Some(191_200);
    analysis.lead_secs = 0.95;
    analysis.tail_secs = 5.05;
    assert_eq!(problems(&analysis), Vec::<RetailProblem>::new());
}

#[test]
fn a_very_short_chapter_is_told_apart_by_the_room_tone() {
    let mut analysis = good();
    analysis.duration_secs = 6.0;
    analysis.rms_dbfs = -26.0;
    assert_eq!(
        problems(&analysis),
        [RetailProblem::Level {
            rms_dbfs: -26.0,
            short: true
        }]
    );
    // Too loud is never the room tone's fault.
    analysis.rms_dbfs = -15.0;
    assert_eq!(
        problems(&analysis),
        [RetailProblem::Level {
            rms_dbfs: -15.0,
            short: false
        }]
    );
}

#[test]
fn file_names_become_plain_ascii() {
    assert_eq!(ascii_stem("Chapter 1"), Some("Chapter_1".into()));
    assert_eq!(
        ascii_stem("Café: l'été à Zürich!"),
        Some("Cafe_l_ete_a_Zurich".into())
    );
    assert_eq!(ascii_stem("Ærø Straße"), Some("AEro_Strasse".into()));
    assert_eq!(
        ascii_stem("Łódź, Þór, Œuvre"),
        Some("Lodz_Thor_OEuvre".into())
    );
    assert_eq!(ascii_stem("第一章"), None);
    assert_eq!(ascii_stem("Глава 1"), Some("1".into()));
    assert_eq!(ascii_stem("  ...  "), None);
}

#[test]
fn the_profile_asks_for_constant_192_and_room_tone() {
    let options = combine_options();
    assert_eq!(options.encoding, Mp3Encoding::Cbr(192));
    let tone = options.room_tone.unwrap();
    assert!((MIN_PAD_SECS..=MAX_PAD_SECS).contains(&tone.lead_secs));
    assert!(tone.level_dbfs < MAX_NOISE_FLOOR_DBFS);
    assert!(options.tags.is_empty());
}

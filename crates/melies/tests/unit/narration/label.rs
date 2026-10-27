use super::*;

#[test]
fn the_tags_say_the_voice_is_synthetic_and_name_who_made_it() {
    let label = Label {
        producer: "manuzio".into(),
        content_id: "abc123".into(),
    };
    let tags = label.tags();
    let value = |name: &str| {
        tags.iter()
            .find(|(key, _)| key == name)
            .map_or_else(|| panic!("no {name} tag"), |(_, value)| value.as_str())
    };
    assert_eq!(
        value("comment"),
        "AI-generated narration: synthetic voice (Kokoro-82M), made with manuzio"
    );
    assert_eq!(
        value("AIGC"),
        r#"{"Label":"1","ContentProducer":"manuzio","ProduceID":"abc123"}"#
    );
    assert_eq!(tags.len(), 2);
}

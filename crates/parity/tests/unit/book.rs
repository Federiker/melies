use super::*;
use std::fs;

/// A sample book shaped like manuzio's: the Introduction with its matter markers, two chapters
/// with headings, and an entry outside `OEBPS/Text`.
fn sample(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("sample.epub");
    let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    let entries = [
        ("mimetype", "application/epub+zip".to_owned()),
        (
            INTRODUCTION,
            "<body><div class=\"latex-container\"><span class=\"mainmatter\"></span></div>\
<div class=\"latex-chapter no-number\"><h2>Introduction</h2></div><p>First.</p><p>Second.</p></body>"
                .to_owned(),
        ),
        (
            CHAPTER_1,
            "<body><div class=\"latex-chapter\"><h2>Chapter I</h2><h3>The Cyc\u{ad}lone</h3></div>\
<p class=\"first-par\">One.</p><p>Two.</p></body>"
                .to_owned(),
        ),
        (
            CHAPTER_2,
            "<body><div class=\"latex-chapter\"><h2>Chapter II</h2></div><p>Three.</p></body>"
                .to_owned(),
        ),
    ];
    for (name, text) in entries {
        zip.start_file(name, options).unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    path
}

fn entries(book: &Path) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(File::open(book).unwrap()).unwrap();
    (0..zip.len())
        .map(|index| {
            let mut entry = zip.by_index(index).unwrap();
            let name = entry.name().unwrap().into_owned();
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            (name, text)
        })
        .collect()
}

fn chapter(book: &Path, name: &str) -> String {
    entries(book)
        .into_iter()
        .find(|(entry, _)| entry == name)
        .unwrap()
        .1
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("melies-parity-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn every_book_cuts_the_introduction_and_keeps_the_other_entries() {
    let dir = folder("books");
    let sample = sample(&dir);
    for book in [Book::Short, Book::Specials, Book::Chinese] {
        let out = dir.join(format!("{book:?}.epub"));
        write_book(&sample, book, &out).unwrap();
        let names: Vec<_> = entries(&out).into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["mimetype", INTRODUCTION, CHAPTER_1, CHAPTER_2]);
        assert_eq!(chapter(&out, "mimetype"), "application/epub+zip");
        assert!(!chapter(&out, INTRODUCTION).contains("Second."), "{book:?}");
    }
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn each_recipe_edits_its_chapters() {
    let dir = folder("recipes");
    let sample = sample(&dir);
    let out = dir.join("book.epub");

    write_book(&sample, Book::Short, &out).unwrap();
    assert!(chapter(&out, INTRODUCTION).contains("<p>First.</p></body>"));
    let first = chapter(&out, CHAPTER_1);
    assert!(first.contains("One.") && !first.contains("Two."), "{first}");
    assert!(chapter(&out, CHAPTER_2).contains("Three."));

    write_book(&sample, Book::Specials, &out).unwrap();
    assert!(chapter(&out, INTRODUCTION).contains(
        "<p>First.</p><p class=\"asterisks\">* * *</p><p>Many years later, Dorothy still knew it.</p>"
    ));
    let ornament = chapter(&out, CHAPTER_1);
    assert!(
        ornament.ends_with("</div><p class=\"asterisks\">⁂</p></body>"),
        "{ornament}"
    );
    assert!(chapter(&out, CHAPTER_2).ends_with("</h2></div></body>"));

    write_book(&sample, Book::Chinese, &out).unwrap();
    let introduction = chapter(&out, INTRODUCTION);
    assert!(
        introduction.contains("<span class=\"mainmatter\">"),
        "{introduction}"
    );
    assert!(
        introduction.contains("<h2>Introduction</h2></div><p>多萝西"),
        "{introduction}"
    );
    let first = chapter(&out, CHAPTER_1);
    assert!(
        first.contains("<h2>第一章</h2><h3>龙卷风</h3></div><p>他们"),
        "{first}"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_book_unlike_the_sample_is_an_error() {
    let dir = folder("unlike");
    let path = dir.join("sample.epub");
    let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
    zip.start_file(INTRODUCTION, zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"<body><p>Only one.</p></body>").unwrap();
    zip.finish().unwrap();
    let error = write_book(&path, Book::Short, &dir.join("out.epub")).unwrap_err();
    assert!(error.to_string().contains("single paragraph"), "{error}");
    assert!(
        write_book(
            &dir.join("missing.epub"),
            Book::Short,
            &dir.join("out.epub")
        )
        .is_err()
    );
    let _ = fs::remove_dir_all(dir);
}

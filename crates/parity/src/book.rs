use crate::Book;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

/// manuzio's sample book, relative to its repository: the narration cases are edits of it.
pub const SAMPLE_BOOK: &str = "assets/samples/oz.epub";

const INTRODUCTION: &str = "OEBPS/Text/introduction.xhtml";
const CHAPTER_1: &str = "OEBPS/Text/chapter_01.xhtml";
const CHAPTER_2: &str = "OEBPS/Text/chapter_02.xhtml";

/// The Introduction and Chapter I, in Chinese.
const CHINESE: [&str; 2] = [
    "多萝西和亨利叔叔、爱姆婶婶一起住在堪萨斯州的大草原中间。",
    "他们的房子很小，因为造房子的木材必须用马车从很远的地方运来。屋子里只有一个房间。",
];

/// Write `book`, an edit of manuzio's sample book at `sample`, to `out`.
///
/// As manuzio's audiobook tests do (`sample_epub_with`): the Introduction is cut to its first
/// paragraph, then each recipe edits the chapter files; every other entry is copied as it is.
///
/// # Errors
/// When the sample cannot be read, or `out` written.
pub fn write_book(sample: &Path, book: Book, out: &Path) -> io::Result<()> {
    let mut source = zip::ZipArchive::new(File::open(sample)?).map_err(io::Error::other)?;
    let mut target = zip::ZipWriter::new(File::create(out)?);
    for index in 0..source.len() {
        let entry = source.by_index(index).map_err(io::Error::other)?;
        let name = entry.name().to_string();
        if !name.starts_with("OEBPS/Text/") {
            target.raw_copy_file(entry).map_err(io::Error::other)?;
            continue;
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        let mut chapter = String::new();
        { entry }.read_to_string(&mut chapter)?;
        if name == INTRODUCTION {
            cut_after_first_paragraph(&mut chapter, "<p>")?;
        }
        edit(book, &name, &mut chapter)?;
        target.start_file(name, options).map_err(io::Error::other)?;
        target.write_all(chapter.as_bytes())?;
    }
    target.finish().map_err(io::Error::other)?;
    Ok(())
}

fn edit(book: Book, entry: &str, chapter: &mut String) -> io::Result<()> {
    match (book, entry) {
        (Book::Short, CHAPTER_1) => cut_after_first_paragraph(chapter, "<p")?,
        (Book::Specials, INTRODUCTION) => {
            let end = find(chapter, "</body>")?;
            chapter.insert_str(
                end,
                "<p class=\"asterisks\">* * *</p><p>Many years later, Dorothy still knew it.</p>",
            );
        }
        (Book::Specials, CHAPTER_1) => replace_text(chapter, "<p class=\"asterisks\">⁂</p>")?,
        (Book::Specials, CHAPTER_2) => replace_text(chapter, "")?,
        (Book::Chinese, INTRODUCTION) => replace_text(chapter, &format!("<p>{}</p>", CHINESE[0]))?,
        (Book::Chinese, CHAPTER_1) => {
            *chapter = chapter
                .replacen("<h2>Chapter I</h2>", "<h2>第一章</h2>", 1)
                .replacen("<h3>The Cyc\u{ad}lone</h3>", "<h3>龙卷风</h3>", 1);
            replace_text(chapter, &format!("<p>{}</p>", CHINESE[1]))?;
        }
        _ => {}
    }
    Ok(())
}

/// Remove everything from the second `paragraph` opening to the end of the body.
fn cut_after_first_paragraph(chapter: &mut String, paragraph: &str) -> io::Result<()> {
    let second = chapter
        .match_indices(paragraph)
        .nth(1)
        .ok_or_else(|| io::Error::other("the chapter has a single paragraph"))?
        .0;
    let end = find(chapter, "</body>")?;
    chapter.replace_range(second..end, "");
    Ok(())
}

/// Replace the chapter's text, after its heading (the `div.latex-chapter`), with `text`.
fn replace_text(chapter: &mut String, text: &str) -> io::Result<()> {
    let heading = find(chapter, "<div class=\"latex-chapter")?;
    let start = heading + find(&chapter[heading..], "</div>")? + "</div>".len();
    let end = find(chapter, "</body>")?;
    chapter.replace_range(start..end, text);
    Ok(())
}

fn find(chapter: &str, what: &str) -> io::Result<usize> {
    chapter
        .find(what)
        .ok_or_else(|| io::Error::other(format!("the chapter has no {what}")))
}

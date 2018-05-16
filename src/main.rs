extern crate lopdf;

use lopdf::{Document, Object, Dictionary};
use std::error::Error as StdError;

// the below code has way too many unwraps...
// I should start handling these freakin errors!

// note: creating thumbnails:
// convert -thumbnail x300 -background white -alpha remove my.pdf thumb.png
//
// This will produce a png thumbnail for each page in the document.

#[derive(Debug)]
pub enum Error {
    Load,
    Save,
    NoCatalog,
    KeyNotFound,
    NoBoolean,
    NoInteger,
    NoReal,
    NoName,
    NoString,
    NoArray,
    NoDictionary,
    NoStream,
    NoReference,
    ReferenceResolve,
}

type PDFModResult<T> = Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Load => "PDF file could not be loaded from the provided path",
            Error::Save => "PDF file could not be saved to the provided location",
            Error::NoCatalog => "The object is not a catalog",
            Error::KeyNotFound => "The dictionary does not contain the requested key",
            Error::NoBoolean => "The object is not a boolean",
            Error::NoInteger => "The object is not an integer",
            Error::NoReal => "The object is not a real",
            Error::NoName => "The object is not a name",
            Error::NoString => "The object is not a string",
            Error::NoArray => "The object is not an array",
            Error::NoDictionary => "The object is not a dictionary",
            Error::NoStream => "The object is not a stream",
            Error::NoReference => "The object is not a reference",
            Error::ReferenceResolve => "Cannot resolve the given reference in the provided document",
        }
    }
}

fn get_pages<'a>(doc: &'a Document) -> PDFModResult<Vec<&'a Object>> {
    doc
        .get_object(
            doc
                .catalog()
                .ok_or(Error::NoCatalog)?
                .get("Pages")
                .ok_or(Error::KeyNotFound)?
                .as_reference()
                .ok_or(Error::NoReference)?
        )
        .ok_or(Error::ReferenceResolve)?
        .as_dict()
        .ok_or(Error::NoDictionary)?
        .get("Kids")
        .ok_or(Error::KeyNotFound)?
        .as_array()
        .ok_or(Error::NoArray)?
        .into_iter()
        .map(|kid| doc
             .get_object(kid.as_reference().ok_or(Error::NoReference)?)
             .ok_or(Error::ReferenceResolve)
        )
        .collect()
}

fn clone_dictionary(from: &Document, to: &mut Document, dict: &Dictionary) -> PDFModResult<Dictionary> {
    let mut new = Dictionary::new();

    for (key, val) in dict {
        let mut was_ref: bool;
        let deref = match val.as_reference() {
            None => {
                was_ref = false;
                val
            },
            Some(id) => {
                was_ref = true;
                from.get_object(id).ok_or(Error::ReferenceResolve)?
            },
        };
        let mut cloned = match deref.as_dict() {
            None => deref.clone(),
            Some(d) => clone_dictionary(from, to, &d)?.into(),
        };

        if was_ref {
            cloned = to.add_object(cloned).into();
        }

        new.set(key.clone(), cloned);
    }
    Ok(new)
}

fn clone_page_into(from: &Document, into: &mut Document, page: usize) -> PDFModResult<()> {
    let from_pages = get_pages(from)?;
    let from_page = from_pages
        .get(page)
        .ok_or(Error::KeyNotFound)?
        .as_dict()
        .ok_or(Error::NoDictionary)?;

    let parent_ref = get_pages(into)?[0]
        .as_dict()
        .ok_or(Error::NoDictionary)?
        .get("Parent")
        .ok_or(Error::KeyNotFound)?
        .as_reference()
        .ok_or(Error::NoReference)?;

    let mut clone = clone_dictionary(from, into, from_page)?;
    clone.set("Parent", parent_ref);
    let clone_id = into.add_object(clone);

    let into_pages_ref = into
        .catalog()
        .ok_or(Error::NoCatalog)?
        .get("Pages")
        .ok_or(Error::KeyNotFound)?
        .as_reference()
        .ok_or(Error::NoReference)?;

    let into_pages = into
        .get_object_mut(into_pages_ref)
        .ok_or(Error::ReferenceResolve)?
        .as_dict_mut()
        .ok_or(Error::NoDictionary)?;

    into_pages
        .get_mut("Kids")
        .ok_or(Error::KeyNotFound)?
        .as_array_mut()
        .ok_or(Error::NoArray)?
        .push(Object::Reference(clone_id));

    let count = into_pages
        .get("Count")
        .ok_or(Error::KeyNotFound)?
        .as_i64()
        .ok_or(Error::NoInteger)? + 1;

    into_pages.set("Count", count);

    Ok(())
}

fn append_doc(to: &mut Document, doc: &Document) -> PDFModResult<()> {
    for page in 0..get_pages(doc)?.len() {
        clone_page_into(doc, to, page)?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = || -> PDFModResult<()> {
        let mut doc1 = Document::load("1.pdf").map_err(|_| Error::Load)?;
        let doc2 = Document::load("2.pdf").map_err(|_| Error::Load)?;

        append_doc(&mut doc1, &doc2)?;

        doc1.save("modified.pdf").map_err(|_| Error::Save)?;

        Ok(())
    }() {
        println!("ERROR: {}", e);
        std::process::exit(1);
    }
}

extern crate lopdf;

use lopdf::{Document, Object};

fn get_pages<'a>(doc: &'a Document) -> Vec<&'a Object> {
    doc
        .get_object(
            doc
                .catalog()
                .unwrap()
                .get("Pages")
                .unwrap()
                .as_reference()
                .unwrap()
        )
        .unwrap()
        .as_dict()
        .unwrap()
        .get("Kids")
        .unwrap()
        .as_array()
        .unwrap()
        .into_iter()
        .map(|kid| doc.get_object(kid.as_reference().unwrap()).unwrap())
        .collect()
}

fn main() {
    let doc1 = Document::load("1.pdf").unwrap();
    let doc2 = Document::load("2.pdf").unwrap();

    println!("{:#?}", get_pages(&doc1));
    println!("{:#?}", get_pages(&doc2));
}

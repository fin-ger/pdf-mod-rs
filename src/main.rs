extern crate lopdf;

use lopdf::{Document, Object};

fn main() {
    let mut doc1 = Document::load("1.pdf").unwrap();
    let doc2 = Document::load("2.pdf").unwrap();

    doc1.version = "1.4".to_string();

    let (_, last_page_id) = doc1.get_pages().into_iter().last().unwrap();
    let (_, first_page_id) = doc2.get_pages().into_iter().next().unwrap();
    let page_id = doc1.add_object(first_page_id);
    doc1.get_dictionary(page_id)
        .unwrap()
        .get_mut("Parent")
        .unwrap() = Object::From(&mut last_page_id);
    doc1.renumber_objects();

    doc1.save("modified.pdf").unwrap();
}

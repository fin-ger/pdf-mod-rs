extern crate lopdf;

use lopdf::{Document};

fn main() {
    let mut doc1 = Document::load("1.pdf").unwrap();
    let doc2 = Document::load("2.pdf").unwrap();

    doc1.version = "1.4".to_string();

    for (_, object_ids) in doc2.get_pages() {
        doc1.add_object(object_ids);
    }

    doc1.save("modified.pdf").unwrap();
}

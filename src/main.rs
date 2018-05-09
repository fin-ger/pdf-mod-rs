extern crate lopdf;

use lopdf::{Document, Object, Dictionary};

// the below code has way too many unwraps...
// I should start handling these freakin errors!

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

fn clone_page_into(from: &Document, into: &mut Document, page: usize) {
    let from_pages = get_pages(from);
    let from_page = from_pages.get(page).unwrap().as_dict().unwrap();

    let contents = from.get_object(
        from_page.get("Contents").unwrap().as_reference().unwrap()
    ).unwrap().as_stream().unwrap();
    let contents_ref = into.add_object(contents.clone());

    let resources = from.get_object(
        from_page.get("Resources").unwrap().as_reference().unwrap()
    ).unwrap().as_dict().unwrap();
    let resources_ref = into.add_object(resources.clone());

    let media_box = from_page.get("MediaBox").unwrap().as_array().unwrap().clone();

    let parent_ref = get_pages(into)[0].as_dict().unwrap().get("Parent").unwrap().as_reference().unwrap();

    let mut clone = Dictionary::new();
    clone.set("Type", Object::Name("Page".into()));
    clone.set("Contents", contents_ref);
    clone.set("Resources", resources_ref);
    clone.set("MediaBox", media_box);
    clone.set("Parent", parent_ref);
    let clone_id = into.add_object(clone);

    let into_pages_ref = into
        .catalog()
        .unwrap()
        .get("Pages")
        .unwrap()
        .as_reference()
        .unwrap();
    let into_pages = into
        .get_object_mut(into_pages_ref)
        .unwrap()
        .as_dict_mut()
        .unwrap();
    into_pages
        .get_mut("Kids")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(Object::Reference(clone_id));
    let count = into_pages.get("Count").unwrap().as_i64().unwrap() + 1;
    into_pages.set("Count", count);
}

fn append_doc(to: &mut Document, doc: &Document) {
    for page in 0..get_pages(doc).len() {
        clone_page_into(doc, to, page);
    }
}

fn main() {
    let mut doc1 = Document::load("1.pdf").unwrap();
    let doc2 = Document::load("2.pdf").unwrap();

    append_doc(&mut doc1, &doc2);

    doc1.save("modified.pdf").unwrap();
}

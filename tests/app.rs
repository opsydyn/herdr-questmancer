use herdr_webmaster::app::{Model, View};

#[test]
fn starts_in_requested_view() {
    assert_eq!(Model::new(View::Cafe).view(), View::Cafe);
}

#[test]
fn switches_views() {
    let mut model = Model::new(View::Desk);
    model.switch_to(View::Cafe);
    assert_eq!(model.view(), View::Cafe);
}

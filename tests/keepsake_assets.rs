use questmancer::{domain::Keepsake, scene::assets::keepsakes};

#[test]
fn all_saved_keepsakes_have_distinct_complete_static_card_art() {
    let mut silhouettes = Vec::new();
    for keepsake in Keepsake::ALL {
        let frame = keepsakes::illustration(*keepsake);
        assert_eq!((frame.size().width, frame.size().height), (8, 8));
        let silhouette = frame
            .pixels()
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>();
        assert!(silhouette.iter().filter(|pixel| **pixel).count() >= 20);
        assert!(
            !silhouettes.contains(&silhouette),
            "{keepsake:?} needs a distinct silhouette"
        );
        silhouettes.push(silhouette);
        assert!(keepsakes::name(*keepsake).is_ascii());
        assert!(keepsakes::description(*keepsake).is_ascii());
        assert!(keepsakes::description(*keepsake).len() <= 39);
    }
}

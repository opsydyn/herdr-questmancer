use questmancer::storybook::{
    catalogue::catalogue,
    fixtures::{StoryContext, StoryFixture},
};
fn main() {
    let stories = catalogue();
    let data=stories.iter().enumerate().map(|(index,story)|serde_json::json!({
        "id":story.id.as_str(),"title":story.title,"category":format!("{:?}",story.category),
        "within_category":stories[..index].iter().filter(|other|other.category==story.category).count(),
        "scene":matches!((story.build)(StoryContext::fixed()),StoryFixture::SceneApplication(_)),
        "minimum":[story.viewport.minimum_width,story.viewport.minimum_height]
    })).collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&data).unwrap());
}

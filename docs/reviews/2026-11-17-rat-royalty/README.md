# Rat Royalty / The Gutter Crown

An original rat monarch with a crenellated gold crown, round ears, a pointed
muzzle and a curled tail. The burgundy mantle, cream collar and sapphire sceptre
carry the portrait’s palette into the smaller versions. At roster size the crown
and snout take priority over the trim, and the tail breaks the outline.

- [All sizes together](rat-royalty-variants.png)
- [Transparent illustrated portrait, 384×512](../../../src/assets/portraits/rat-royalty-card.png)
- [World sprite, 16×24](world.png)
- [Crowded roster sprite, 8×12](roster.png)
- [Profile fallback, 24×32](profile.png)

The world and roster sprites are authored independently. The profile centres the
world sprite at 1×. Storybook’s `asset.rat-royalty` gallery shows the three pixel
sizes at a minimum of 80×28 terminal cells. The art was initially reviewed on
its own. Rat Royalty is now available through the character picker in either
room, and `interaction.character-picker` previews the production parchment.

```sh
cargo run --features storybook --bin questmancer-storybook -- --story asset.rat-royalty --inspect
```

`just verify` passed 845 Rust tests and 28 shell tests. The checks verified
the native portrait dimensions, transparent margin, sprite foot anchors,
pixel-for-pixel profile alignment and the gallery at its smallest viewport.
The Storybook and Rustdoc builds passed.

The illustrated PNG was made with built-in ImageGen, using the existing Bard
portrait as a style and scale reference. Final prompt:

> A new original character asset for Questmancer, matching the provided Questmancer Bard portrait only as a style and scale reference. Full-body 3/4 front view of a stocky, proud anthropomorphic rat monarch, the Gutter Crown. Clever warm amber eyes, cool grey-brown fur, pink inner round ears behind a clearly visible chunky open gold crown with THREE squared crenellations and a small blue jewel, pointed muzzle, two tiny pale incisors, curled bare pink tail visible to one side. Burgundy velvet mantle with a narrow cream ermine collar, antique brass clasps, dark boots. Short ornate gold sceptre with a sapphire finial, held upright on the other side. Crisp chunky pixel clusters, warm gold rim lights, dark plum outlines, deep but restrained palette, the same head-to-body proportions and 16-bit illustrated game portrait style as the provided reference. Friendly but imperious, ragged undercity court without gore. Single centered character on genuine transparent background, full silhouette with breathing space on all sides, portrait 3:4 design, no text, no border, no floor, no drop shadow, no logos, no recognizable franchise costume or symbols. Crown and rat face remain immediately readable at thumbnail size.

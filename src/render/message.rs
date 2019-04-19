use layer::*;
use sprite::*;

pub enum RenderMessage {
    // Layer
    LayerCreate {
        layer: usize,
        desc: LayerDescription,
    },
    LayerUpdate {
        layer: usize,
        desc: LayerDescription,
    },
    LayerRemove {
        layer: usize,
    },
    LayerClear {
        layer: usize,
    },

    // Sprite
    SpriteCreate {
        layer: usize,
        desc: SpriteDescription,
    },
    SpriteUpdate {
        layer: usize,
        sprite: usize,
        desc: SpriteDescription,
    },
    SpriteRemove {
        layer: usize,
        sprite: usize,
    },

    // Texture
    TextureLoad {
        path: String,
    },

    // Text
    FontLoad {
        path: String,
    },
    // TextCreate {
    //     text: String,
    // },
    // TextUpdate {
    //     text: String,
    // },
    // TextRemove {},

    // Window
    WindowTitle {
        title: String,
    },
}

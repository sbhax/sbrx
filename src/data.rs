// Data for Sonic Battle

pub const PHI_PALETTE: i32 = 0x47AB78;
pub const DUST_CLOUD_PALETTE: i32 = 0xBF2058;
pub const SONIC_MINE_PALETTE: i32 = 0xBF20D8;
pub const TAILS_BLASTER_PALETTE: i32 = 0xBF2098;
pub const SHIELD_PALETTE: i32 = 0xBF2078;

#[derive(Copy, Clone)]
pub struct Character<'a> {
    pub name: &'a str,
    pub palette_offset: u64,
    pub text_offsets: (i32, i32),
    pub sprite_offset: i32,
    pub sprite_frames: &'a [i32],
}

pub const CHARACTERS: [Character; 10] = [
    SONIC_DATA, KNUCKLES_DATA, TAILS_DATA, SHADOW_DATA, ROUGE_DATA,
    AMY_DATA, E102_DATA, CREAM_DATA, CHAOS_DATA, EGGMAN_DATA
];

pub const SONIC_DATA: Character = Character {
    name: "Sonic",
    palette_offset: 0x47AFB8,
    text_offsets: (0x1DB3FC, 0x1E1467),
    sprite_offset: 0x47AFD8,
    sprite_frames: &[8, 4, 8, 4, 8, 4, 4, 4, 8, 4, 8, 8, 8, 8, 16, 12, 12, 8, 12, 8, 8, 16, 8, 12,
        8, 8, 4, 8, 4, 4, 8, 8, 4, 8, 4, 8, 4, 4],
};

pub const KNUCKLES_DATA: Character = Character {
    name: "Knuckles",
    palette_offset: 0x4CADD8,
    text_offsets: (0x1ED2C4, 0x1F417F),
    sprite_offset: 0x4CADF8,
    sprite_frames: &[8, 4, 8, 4, 8, 4, 4, 4, 8, 4, 8, 8, 8, 12, 16, 12, 12, 8, 12, 8, 8, 8, 8, 8,
        12, 8, 8, 4, 8, 8, 12, 8, 4, 8, 4, 4, 8, 8, 4, 8, 4, 4, 8, 4, 4],
};

pub const TAILS_DATA: Character = Character {
    name: "Tails",
    palette_offset: 0x5283F8,
    text_offsets: (0x1E146A, 0x1E6FA7),
    sprite_offset: 0x528418,
    sprite_frames: &[8, 4, 8, 4, 8, 4, 4, 4, 8, 4, 8, 8, 8, 8, 28, 12, 12, 8, 8, 8, 8, 20, 8, 20,
        16, 8, 8, 4, 8, 8, 8, 8, 4, 8, 8, 4, 8, 8, 8, 8, 8, 4, 4],
};

pub const SHADOW_DATA: Character = Character {
    name: "Shadow",
    palette_offset: 0x58D818,
    text_offsets: (0x1FE870, 0x206103),
    sprite_offset: 0x58D838,
    sprite_frames: &[8, 4, 28, 12, 8, 4, 4, 8, 8, 8, 4, 8, 8, 8, 12, 24, 16, 20, 8, 4, 8, 12, 12,
        8, 24, 8, 12, 8, 4, 8, 4, 4, 8, 8, 12, 4, 4, 4, 4, 4],
};

pub const ROUGE_DATA: Character = Character {
    name: "Rouge",
    palette_offset: 0x5F3E38,
    text_offsets: (0x1E6FA8, 0x1ED2C3),
    sprite_offset: 0x5F3E58,
    sprite_frames: &[8, 4, 8, 4, 8, 4, 8, 4, 8, 4, 8, 12, 16, 12, 8, 8, 12, 8, 4, 12, 8, 4, 8, 4,
        4, 8, 8, 12, 4, 4, 4, 4, 4],
};

pub const AMY_DATA: Character = Character {
    name: "Amy",
    palette_offset: 0x636458,
    text_offsets: (0x1F4180, 0x1F9CDB),
    sprite_offset: 0x636478,
    sprite_frames: &[8, 4, 8, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 8, 8, 16, 16, 8, 8, 12, 12, 8, 8, 8,
        12, 8, 4, 8, 8, 8, 8, 8, 4, 4, 8, 4, 4],
};

pub const E102_DATA: Character = Character {
    name: "E-102",
    palette_offset: 0x681A78,
    text_offsets: (-1, -1),
    sprite_offset: 0x681A98,
    sprite_frames: &[8, 4, 8, 4, 4, 4, 4, 4, 4, 8, 4, 8, 8, 8, 12, 16, 12, 12, 8, 12, 8, 8, 16,
        12, 12, 16, 12, 12, 28, 4, 4, 20, 40, 4, 8, 4, 4, 4, 4, 8, 4, 4, 8, 4, 8, 4, 4],
};

pub const CREAM_DATA: Character = Character {
    name: "Cream",
    palette_offset: 0x6F6A98,
    text_offsets: (0x1F9CDC, 0x1FE86F),
    sprite_offset: 0x6F6AB8,
    sprite_frames: &[8, 4, 20, 4, 4, 8, 8, 12, 8, 8, 8, 16, 8, 12, 8, 16, 12, 4, 16, 12, 4, 8, 4,
        4],
};

pub const CHAOS_DATA: Character = Character {
    name: "Chaos",
    palette_offset: 0x7336B8,
    text_offsets: (-1, -1),
    sprite_offset: 0x7336D8,
    sprite_frames: &[8, 4, 8, 8, 12, 4, 8, 8, 4, 8, 8, 12, 16, 16, 8, 8, 8, 8, 20, 8, 8, 12, 8, 4,
        8, 8, 8, 8, 8, 4, 4, 8, 4, 4],
};

pub const EMERL_DATA: Character = Character {
    name: "Emerl",
    palette_offset: 0x47AB38,
    text_offsets: (0x206104, 0x20B131),
    sprite_offset: 0x787D18,
    sprite_frames: &[-1],
};

pub const EGGMAN_DATA: Character = Character {
    name: "Eggman",
    palette_offset: 0x7822D8,
    text_offsets: (-1, -1),
    sprite_offset: 0x7822F8,
    sprite_frames: &[4, 4, 4, 4, 4],
};

pub fn compute_sprite_offsets(character: &Character) -> Vec<(i32, i32)> {
    let mut data = Vec::new();
    let mut o = 0;
    for frame in character.sprite_frames.iter() {
        data.push((character.sprite_offset + 0x480 * o, *frame));
        o += *frame;
    }
    data
}

//! Environment keys that control the visual aspect (theme) of common widgets.
use crate::{style::Style, Color, EnvKey, Environment, Font, Length, SideOffsets, UnitExt};
use once_cell::sync::Lazy;

macro_rules! theme_key {
    ($name:tt) => {
        EnvKey::new(atom!($name))
    };
}

/// Builtin themes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Theme {
    Dark,
    Light,
}

pub const FONT_SIZE: EnvKey<f64> = theme_key!("font-size"); // [14.0];
pub const TEXT_COLOR: EnvKey<Color> = theme_key!("text-color");
pub const DEFAULT_FONT: EnvKey<Font> = theme_key!("default-font");
pub const DARK_MODE: EnvKey<bool> = theme_key!("dark-mode");
pub const WINDOW_BACKGROUND_COLOR: EnvKey<Color> = theme_key!("window-background-color");
pub const TEXT_BACKGROUND_COLOR: EnvKey<Color> = theme_key!("text-background-color");

pub mod palette {
    use crate::Color;
    pub const RED_50: Color = Color::from_hex("#ffebee"); //   #ffebee;
    pub const RED_100: Color = Color::from_hex("#ffcdd2"); //  #ffcdd2;
    pub const RED_200: Color = Color::from_hex("#ef9a9a"); //  #ef9a9a;
    pub const RED_300: Color = Color::from_hex("#e57373"); //  #e57373;
    pub const RED_400: Color = Color::from_hex("#ef5350"); //  #ef5350;
    pub const RED_500: Color = Color::from_hex("#f44336"); //  #f44336;
    pub const RED_600: Color = Color::from_hex("#e53935"); //  #e53935;
    pub const RED_700: Color = Color::from_hex("#d32f2f"); //  #d32f2f;
    pub const RED_800: Color = Color::from_hex("#c62828"); //  #c62828;
    pub const RED_900: Color = Color::from_hex("#b71c1c"); //  #b71c1c;
    pub const RED_A100: Color = Color::from_hex("#ff8a80"); // #ff8a80;
    pub const RED_A200: Color = Color::from_hex("#ff5252"); // #ff5252;
    pub const RED_A400: Color = Color::from_hex("#ff1744"); // #ff1744;
    pub const RED_A700: Color = Color::from_hex("#d50000"); // #d50000;
    pub const PINK_50: Color = Color::from_hex("#fce4ec"); //   #fce4ec;
    pub const PINK_100: Color = Color::from_hex("#f8bbd0"); //  #f8bbd0;
    pub const PINK_200: Color = Color::from_hex("#f48fb1"); //  #f48fb1;
    pub const PINK_300: Color = Color::from_hex("#f06292"); //  #f06292;
    pub const PINK_400: Color = Color::from_hex("#ec407a"); //  #ec407a;
    pub const PINK_500: Color = Color::from_hex("#e91e63"); //  #e91e63;
    pub const PINK_600: Color = Color::from_hex("#d81b60"); //  #d81b60;
    pub const PINK_700: Color = Color::from_hex("#c2185b"); //  #c2185b;
    pub const PINK_800: Color = Color::from_hex("#ad1457"); //  #ad1457;
    pub const PINK_900: Color = Color::from_hex("#880e4f"); //  #880e4f;
    pub const PINK_A100: Color = Color::from_hex("#ff80ab"); // #ff80ab;
    pub const PINK_A200: Color = Color::from_hex("#ff4081"); // #ff4081;
    pub const PINK_A400: Color = Color::from_hex("#f50057"); // #f50057;
    pub const PINK_A700: Color = Color::from_hex("#c51162"); // #c51162;
    pub const PURPLE_50: Color = Color::from_hex("#f3e5f5"); // #f3e5f5;
    pub const PURPLE_100: Color = Color::from_hex("#e1bee7"); // #e1bee7;
    pub const PURPLE_200: Color = Color::from_hex("#ce93d8"); // #ce93d8;
    pub const PURPLE_300: Color = Color::from_hex("#ba68c8"); // #ba68c8;
    pub const PURPLE_400: Color = Color::from_hex("#ab47bc"); // #ab47bc;
    pub const PURPLE_500: Color = Color::from_hex("#9c27b0"); // #9c27b0;
    pub const PURPLE_600: Color = Color::from_hex("#8e24aa"); // #8e24aa;
    pub const PURPLE_700: Color = Color::from_hex("#7b1fa2"); // #7b1fa2;
    pub const PURPLE_800: Color = Color::from_hex("#6a1b9a"); // #6a1b9a;
    pub const PURPLE_900: Color = Color::from_hex("#4a148c"); // #4a148c;
    pub const PURPLE_A100: Color = Color::from_hex("#ea80fc"); // #ea80fc;
    pub const PURPLE_A200: Color = Color::from_hex("#e040fb"); // #e040fb;
    pub const PURPLE_A400: Color = Color::from_hex("#d500f9"); // #d500f9;
    pub const PURPLE_A700: Color = Color::from_hex("#aa00ff"); // #aa00ff;
    pub const DEEP_PURPLE_50: Color = Color::from_hex("#ede7f6"); // #ede7f6;
    pub const DEEP_PURPLE_100: Color = Color::from_hex("#d1c4e9"); // #d1c4e9;
    pub const DEEP_PURPLE_200: Color = Color::from_hex("#b39ddb"); // #b39ddb;
    pub const DEEP_PURPLE_300: Color = Color::from_hex("#9575cd"); // #9575cd;
    pub const DEEP_PURPLE_400: Color = Color::from_hex("#7e57c2"); // #7e57c2;
    pub const DEEP_PURPLE_500: Color = Color::from_hex("#673ab7"); // #673ab7;
    pub const DEEP_PURPLE_600: Color = Color::from_hex("#5e35b1"); // #5e35b1;
    pub const DEEP_PURPLE_700: Color = Color::from_hex("#512da8"); // #512da8;
    pub const DEEP_PURPLE_800: Color = Color::from_hex("#4527a0"); // #4527a0;
    pub const DEEP_PURPLE_900: Color = Color::from_hex("#311b92"); // #311b92;
    pub const DEEP_PURPLE_A100: Color = Color::from_hex("#b388ff"); // #b388ff;
    pub const DEEP_PURPLE_A200: Color = Color::from_hex("#7c4dff"); // #7c4dff;
    pub const DEEP_PURPLE_A400: Color = Color::from_hex("#651fff"); // #651fff;
    pub const DEEP_PURPLE_A700: Color = Color::from_hex("#6200ea"); // #6200ea;
    pub const INDIGO_50: Color = Color::from_hex("#e8eaf6"); // #e8eaf6;
    pub const INDIGO_100: Color = Color::from_hex("#c5cae9"); // #c5cae9;
    pub const INDIGO_200: Color = Color::from_hex("#9fa8da"); // #9fa8da;
    pub const INDIGO_300: Color = Color::from_hex("#7986cb"); // #7986cb;
    pub const INDIGO_400: Color = Color::from_hex("#5c6bc0"); // #5c6bc0;
    pub const INDIGO_500: Color = Color::from_hex("#3f51b5"); // #3f51b5;
    pub const INDIGO_600: Color = Color::from_hex("#3949ab"); // #3949ab;
    pub const INDIGO_700: Color = Color::from_hex("#303f9f"); // #303f9f;
    pub const INDIGO_800: Color = Color::from_hex("#283593"); // #283593;
    pub const INDIGO_900: Color = Color::from_hex("#1a237e"); // #1a237e;
    pub const INDIGO_A100: Color = Color::from_hex("#8c9eff"); // #8c9eff;
    pub const INDIGO_A200: Color = Color::from_hex("#536dfe"); // #536dfe;
    pub const INDIGO_A400: Color = Color::from_hex("#3d5afe"); // #3d5afe;
    pub const INDIGO_A700: Color = Color::from_hex("#304ffe"); // #304ffe;
    pub const BLUE_50: Color = Color::from_hex("#e3f2fd"); // #e3f2fd;
    pub const BLUE_100: Color = Color::from_hex("#bbdefb"); // #bbdefb;
    pub const BLUE_200: Color = Color::from_hex("#90caf9"); // #90caf9;
    pub const BLUE_300: Color = Color::from_hex("#64b5f6"); // #64b5f6;
    pub const BLUE_400: Color = Color::from_hex("#42a5f5"); // #42a5f5;
    pub const BLUE_500: Color = Color::from_hex("#2196f3"); // #2196f3;
    pub const BLUE_600: Color = Color::from_hex("#1e88e5"); // #1e88e5;
    pub const BLUE_700: Color = Color::from_hex("#1976d2"); // #1976d2;
    pub const BLUE_800: Color = Color::from_hex("#1565c0"); // #1565c0;
    pub const BLUE_900: Color = Color::from_hex("#0d47a1"); // #0d47a1;
    pub const BLUE_A100: Color = Color::from_hex("#82b1ff"); // #82b1ff;
    pub const BLUE_A200: Color = Color::from_hex("#448aff"); // #448aff;
    pub const BLUE_A400: Color = Color::from_hex("#2979ff"); // #2979ff;
    pub const BLUE_A700: Color = Color::from_hex("#2962ff"); // #2962ff;
    pub const LIGHT_BLUE_50: Color = Color::from_hex("#e1f5fe"); // #e1f5fe;
    pub const LIGHT_BLUE_100: Color = Color::from_hex("#b3e5fc"); // #b3e5fc;
    pub const LIGHT_BLUE_200: Color = Color::from_hex("#81d4fa"); // #81d4fa;
    pub const LIGHT_BLUE_300: Color = Color::from_hex("#4fc3f7"); // #4fc3f7;
    pub const LIGHT_BLUE_400: Color = Color::from_hex("#29b6f6"); // #29b6f6;
    pub const LIGHT_BLUE_500: Color = Color::from_hex("#03a9f4"); // #03a9f4;
    pub const LIGHT_BLUE_600: Color = Color::from_hex("#039be5"); // #039be5;
    pub const LIGHT_BLUE_700: Color = Color::from_hex("#0288d1"); // #0288d1;
    pub const LIGHT_BLUE_800: Color = Color::from_hex("#0277bd"); // #0277bd;
    pub const LIGHT_BLUE_900: Color = Color::from_hex("#01579b"); // #01579b;
    pub const LIGHT_BLUE_A100: Color = Color::from_hex("#80d8ff"); // #80d8ff;
    pub const LIGHT_BLUE_A200: Color = Color::from_hex("#40c4ff"); // #40c4ff;
    pub const LIGHT_BLUE_A400: Color = Color::from_hex("#00b0ff"); // #00b0ff;
    pub const LIGHT_BLUE_A700: Color = Color::from_hex("#0091ea"); // #0091ea;
    pub const CYAN_50: Color = Color::from_hex("#e0f7fa"); // #e0f7fa;
    pub const CYAN_100: Color = Color::from_hex("#b2ebf2"); // #b2ebf2;
    pub const CYAN_200: Color = Color::from_hex("#80deea"); // #80deea;
    pub const CYAN_300: Color = Color::from_hex("#4dd0e1"); // #4dd0e1;
    pub const CYAN_400: Color = Color::from_hex("#26c6da"); // #26c6da;
    pub const CYAN_500: Color = Color::from_hex("#00bcd4"); // #00bcd4;
    pub const CYAN_600: Color = Color::from_hex("#00acc1"); // #00acc1;
    pub const CYAN_700: Color = Color::from_hex("#0097a7"); // #0097a7;
    pub const CYAN_800: Color = Color::from_hex("#00838f"); // #00838f;
    pub const CYAN_900: Color = Color::from_hex("#006064"); // #006064;
    pub const CYAN_A100: Color = Color::from_hex("#84ffff"); // #84ffff;
    pub const CYAN_A200: Color = Color::from_hex("#18ffff"); // #18ffff;
    pub const CYAN_A400: Color = Color::from_hex("#00e5ff"); // #00e5ff;
    pub const CYAN_A700: Color = Color::from_hex("#00b8d4"); // #00b8d4;
    pub const TEAL_50: Color = Color::from_hex("#e0f2f1"); // #e0f2f1;
    pub const TEAL_100: Color = Color::from_hex("#b2dfdb"); // #b2dfdb;
    pub const TEAL_200: Color = Color::from_hex("#80cbc4"); // #80cbc4;
    pub const TEAL_300: Color = Color::from_hex("#4db6ac"); // #4db6ac;
    pub const TEAL_400: Color = Color::from_hex("#26a69a"); // #26a69a;
    pub const TEAL_500: Color = Color::from_hex("#009688"); // #009688;
    pub const TEAL_600: Color = Color::from_hex("#00897b"); // #00897b;
    pub const TEAL_700: Color = Color::from_hex("#00796b"); // #00796b;
    pub const TEAL_800: Color = Color::from_hex("#00695c"); // #00695c;
    pub const TEAL_900: Color = Color::from_hex("#004d40"); // #004d40;
    pub const TEAL_A100: Color = Color::from_hex("#a7ffeb"); // #a7ffeb;
    pub const TEAL_A200: Color = Color::from_hex("#64ffda"); // #64ffda;
    pub const TEAL_A400: Color = Color::from_hex("#1de9b6"); // #1de9b6;
    pub const TEAL_A700: Color = Color::from_hex("#00bfa5"); // #00bfa5;
    pub const GREEN_50: Color = Color::from_hex("#e8f5e9"); // #e8f5e9;
    pub const GREEN_100: Color = Color::from_hex("#c8e6c9"); // #c8e6c9;
    pub const GREEN_200: Color = Color::from_hex("#a5d6a7"); // #a5d6a7;
    pub const GREEN_300: Color = Color::from_hex("#81c784"); // #81c784;
    pub const GREEN_400: Color = Color::from_hex("#66bb6a"); // #66bb6a;
    pub const GREEN_500: Color = Color::from_hex("#4caf50"); // #4caf50;
    pub const GREEN_600: Color = Color::from_hex("#43a047"); // #43a047;
    pub const GREEN_700: Color = Color::from_hex("#388e3c"); // #388e3c;
    pub const GREEN_800: Color = Color::from_hex("#2e7d32"); // #2e7d32;
    pub const GREEN_900: Color = Color::from_hex("#1b5e20"); // #1b5e20;
    pub const GREEN_A100: Color = Color::from_hex("#b9f6ca"); // #b9f6ca;
    pub const GREEN_A200: Color = Color::from_hex("#69f0ae"); // #69f0ae;
    pub const GREEN_A400: Color = Color::from_hex("#00e676"); // #00e676;
    pub const GREEN_A700: Color = Color::from_hex("#00c853"); // #00c853;
    pub const LIGHT_GREEN_50: Color = Color::from_hex("#f1f8e9"); // #f1f8e9;
    pub const LIGHT_GREEN_100: Color = Color::from_hex("#dcedc8"); // #dcedc8;
    pub const LIGHT_GREEN_200: Color = Color::from_hex("#c5e1a5"); // #c5e1a5;
    pub const LIGHT_GREEN_300: Color = Color::from_hex("#aed581"); // #aed581;
    pub const LIGHT_GREEN_400: Color = Color::from_hex("#9ccc65"); // #9ccc65;
    pub const LIGHT_GREEN_500: Color = Color::from_hex("#8bc34a"); // #8bc34a;
    pub const LIGHT_GREEN_600: Color = Color::from_hex("#7cb342"); // #7cb342;
    pub const LIGHT_GREEN_700: Color = Color::from_hex("#689f38"); // #689f38;
    pub const LIGHT_GREEN_800: Color = Color::from_hex("#558b2f"); // #558b2f;
    pub const LIGHT_GREEN_900: Color = Color::from_hex("#33691e"); // #33691e;
    pub const LIGHT_GREEN_A100: Color = Color::from_hex("#ccff90"); // #ccff90;
    pub const LIGHT_GREEN_A200: Color = Color::from_hex("#b2ff59"); // #b2ff59;
    pub const LIGHT_GREEN_A400: Color = Color::from_hex("#76ff03"); // #76ff03;
    pub const LIGHT_GREEN_A700: Color = Color::from_hex("#64dd17"); // #64dd17;
    pub const LIME_50: Color = Color::from_hex("#f9fbe7"); // #f9fbe7;
    pub const LIME_100: Color = Color::from_hex("#f0f4c3"); // #f0f4c3;
    pub const LIME_200: Color = Color::from_hex("#e6ee9c"); // #e6ee9c;
    pub const LIME_300: Color = Color::from_hex("#dce775"); // #dce775;
    pub const LIME_400: Color = Color::from_hex("#d4e157"); // #d4e157;
    pub const LIME_500: Color = Color::from_hex("#cddc39"); // #cddc39;
    pub const LIME_600: Color = Color::from_hex("#c0ca33"); // #c0ca33;
    pub const LIME_700: Color = Color::from_hex("#afb42b"); // #afb42b;
    pub const LIME_800: Color = Color::from_hex("#9e9d24"); // #9e9d24;
    pub const LIME_900: Color = Color::from_hex("#827717"); // #827717;
    pub const LIME_A100: Color = Color::from_hex("#f4ff81"); // #f4ff81;
    pub const LIME_A200: Color = Color::from_hex("#eeff41"); // #eeff41;
    pub const LIME_A400: Color = Color::from_hex("#c6ff00"); // #c6ff00;
    pub const LIME_A700: Color = Color::from_hex("#aeea00"); // #aeea00;
    pub const YELLOW_50: Color = Color::from_hex("#fffde7"); // #fffde7;
    pub const YELLOW_100: Color = Color::from_hex("#fff9c4"); // #fff9c4;
    pub const YELLOW_200: Color = Color::from_hex("#fff59d"); // #fff59d;
    pub const YELLOW_300: Color = Color::from_hex("#fff176"); // #fff176;
    pub const YELLOW_400: Color = Color::from_hex("#ffee58"); // #ffee58;
    pub const YELLOW_500: Color = Color::from_hex("#ffeb3b"); // #ffeb3b;
    pub const YELLOW_600: Color = Color::from_hex("#fdd835"); // #fdd835;
    pub const YELLOW_700: Color = Color::from_hex("#fbc02d"); // #fbc02d;
    pub const YELLOW_800: Color = Color::from_hex("#f9a825"); // #f9a825;
    pub const YELLOW_900: Color = Color::from_hex("#f57f17"); // #f57f17;
    pub const YELLOW_A100: Color = Color::from_hex("#ffff8d"); // #ffff8d;
    pub const YELLOW_A200: Color = Color::from_hex("#ffff00"); // #ffff00;
    pub const YELLOW_A400: Color = Color::from_hex("#ffea00"); // #ffea00;
    pub const YELLOW_A700: Color = Color::from_hex("#ffd600"); // #ffd600;
    pub const AMBER_50: Color = Color::from_hex("#fff8e1"); // #fff8e1;
    pub const AMBER_100: Color = Color::from_hex("#ffecb3"); // #ffecb3;
    pub const AMBER_200: Color = Color::from_hex("#ffe082"); // #ffe082;
    pub const AMBER_300: Color = Color::from_hex("#ffd54f"); // #ffd54f;
    pub const AMBER_400: Color = Color::from_hex("#ffca28"); // #ffca28;
    pub const AMBER_500: Color = Color::from_hex("#ffc107"); // #ffc107;
    pub const AMBER_600: Color = Color::from_hex("#ffb300"); // #ffb300;
    pub const AMBER_700: Color = Color::from_hex("#ffa000"); // #ffa000;
    pub const AMBER_800: Color = Color::from_hex("#ff8f00"); // #ff8f00;
    pub const AMBER_900: Color = Color::from_hex("#ff6f00"); // #ff6f00;
    pub const AMBER_A100: Color = Color::from_hex("#ffe57f"); // #ffe57f;
    pub const AMBER_A200: Color = Color::from_hex("#ffd740"); // #ffd740;
    pub const AMBER_A400: Color = Color::from_hex("#ffc400"); // #ffc400;
    pub const AMBER_A700: Color = Color::from_hex("#ffab00"); // #ffab00;
    pub const ORANGE_50: Color = Color::from_hex("#fff3e0"); // #fff3e0;
    pub const ORANGE_100: Color = Color::from_hex("#ffe0b2"); // #ffe0b2;
    pub const ORANGE_200: Color = Color::from_hex("#ffcc80"); // #ffcc80;
    pub const ORANGE_300: Color = Color::from_hex("#ffb74d"); // #ffb74d;
    pub const ORANGE_400: Color = Color::from_hex("#ffa726"); // #ffa726;
    pub const ORANGE_500: Color = Color::from_hex("#ff9800"); // #ff9800;
    pub const ORANGE_600: Color = Color::from_hex("#fb8c00"); // #fb8c00;
    pub const ORANGE_700: Color = Color::from_hex("#f57c00"); // #f57c00;
    pub const ORANGE_800: Color = Color::from_hex("#ef6c00"); // #ef6c00;
    pub const ORANGE_900: Color = Color::from_hex("#e65100"); // #e65100;
    pub const ORANGE_A100: Color = Color::from_hex("#ffd180"); // #ffd180;
    pub const ORANGE_A200: Color = Color::from_hex("#ffab40"); // #ffab40;
    pub const ORANGE_A400: Color = Color::from_hex("#ff9100"); // #ff9100;
    pub const ORANGE_A700: Color = Color::from_hex("#ff6d00"); // #ff6d00;
    pub const DEEP_ORANGE_50: Color = Color::from_hex("#fbe9e7"); // #fbe9e7;
    pub const DEEP_ORANGE_100: Color = Color::from_hex("#ffccbc"); // #ffccbc;
    pub const DEEP_ORANGE_200: Color = Color::from_hex("#ffab91"); // #ffab91;
    pub const DEEP_ORANGE_300: Color = Color::from_hex("#ff8a65"); // #ff8a65;
    pub const DEEP_ORANGE_400: Color = Color::from_hex("#ff7043"); // #ff7043;
    pub const DEEP_ORANGE_500: Color = Color::from_hex("#ff5722"); // #ff5722;
    pub const DEEP_ORANGE_600: Color = Color::from_hex("#f4511e"); // #f4511e;
    pub const DEEP_ORANGE_700: Color = Color::from_hex("#e64a19"); // #e64a19;
    pub const DEEP_ORANGE_800: Color = Color::from_hex("#d84315"); // #d84315;
    pub const DEEP_ORANGE_900: Color = Color::from_hex("#bf360c"); // #bf360c;
    pub const DEEP_ORANGE_A100: Color = Color::from_hex("#ff9e80"); // #ff9e80;
    pub const DEEP_ORANGE_A200: Color = Color::from_hex("#ff6e40"); // #ff6e40;
    pub const DEEP_ORANGE_A400: Color = Color::from_hex("#ff3d00"); // #ff3d00;
    pub const DEEP_ORANGE_A700: Color = Color::from_hex("#dd2c00"); // #dd2c00;
    pub const BROWN_50: Color = Color::from_hex("#efebe9"); // #efebe9;
    pub const BROWN_100: Color = Color::from_hex("#d7ccc8"); // #d7ccc8;
    pub const BROWN_200: Color = Color::from_hex("#bcaaa4"); // #bcaaa4;
    pub const BROWN_300: Color = Color::from_hex("#a1887f"); // #a1887f;
    pub const BROWN_400: Color = Color::from_hex("#8d6e63"); // #8d6e63;
    pub const BROWN_500: Color = Color::from_hex("#795548"); // #795548;
    pub const BROWN_600: Color = Color::from_hex("#6d4c41"); // #6d4c41;
    pub const BROWN_700: Color = Color::from_hex("#5d4037"); // #5d4037;
    pub const BROWN_800: Color = Color::from_hex("#4e342e"); // #4e342e;
    pub const BROWN_900: Color = Color::from_hex("#3e2723"); // #3e2723;
    pub const BROWN_A100: Color = Color::from_hex("#d7ccc8"); // #d7ccc8;
    pub const BROWN_A200: Color = Color::from_hex("#bcaaa4"); // #bcaaa4;
    pub const BROWN_A400: Color = Color::from_hex("#8d6e63"); // #8d6e63;
    pub const BROWN_A700: Color = Color::from_hex("#5d4037"); // #5d4037;
    pub const GREY_50: Color = Color::from_hex("#fafafa"); // #fafafa;
    pub const GREY_100: Color = Color::from_hex("#f5f5f5"); // #f5f5f5;
    pub const GREY_200: Color = Color::from_hex("#eeeeee"); // #eeeeee;
    pub const GREY_300: Color = Color::from_hex("#e0e0e0"); // #e0e0e0;
    pub const GREY_400: Color = Color::from_hex("#bdbdbd"); // #bdbdbd;
    pub const GREY_500: Color = Color::from_hex("#9e9e9e"); // #9e9e9e;
    pub const GREY_600: Color = Color::from_hex("#757575"); // #757575;
    pub const GREY_700: Color = Color::from_hex("#616161"); // #616161;
    pub const GREY_800: Color = Color::from_hex("#424242"); // #424242;
    pub const GREY_900: Color = Color::from_hex("#212121"); // #212121;
    pub const GREY_A100: Color = Color::from_hex("#ffffff"); // #ffffff;
    pub const GREY_A200: Color = Color::from_hex("#eeeeee"); // #eeeeee;
    pub const GREY_A400: Color = Color::from_hex("#bdbdbd"); // #bdbdbd;
    pub const GREY_A700: Color = Color::from_hex("#616161"); // #616161;
    pub const BLUE_GREY_50: Color = Color::from_hex("#eceff1"); // #eceff1;
    pub const BLUE_GREY_100: Color = Color::from_hex("#cfd8dc"); // #cfd8dc;
    pub const BLUE_GREY_200: Color = Color::from_hex("#b0bec5"); // #b0bec5;
    pub const BLUE_GREY_300: Color = Color::from_hex("#90a4ae"); // #90a4ae;
    pub const BLUE_GREY_400: Color = Color::from_hex("#78909c"); // #78909c;
    pub const BLUE_GREY_500: Color = Color::from_hex("#607d8b"); // #607d8b;
    pub const BLUE_GREY_600: Color = Color::from_hex("#546e7a"); // #546e7a;
    pub const BLUE_GREY_700: Color = Color::from_hex("#455a64"); // #455a64;
    pub const BLUE_GREY_800: Color = Color::from_hex("#37474f"); // #37474f;
    pub const BLUE_GREY_900: Color = Color::from_hex("#263238"); // #263238;
    pub const BLUE_GREY_A100: Color = Color::from_hex("#cfd8dc"); // #cfd8dc;
    pub const BLUE_GREY_A200: Color = Color::from_hex("#b0bec5"); // #b0bec5;
    pub const BLUE_GREY_A400: Color = Color::from_hex("#78909c"); // #78909c;
    pub const BLUE_GREY_A700: Color = Color::from_hex("#455a64"); // #455a64;
}

static DARK_THEME: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set(&DARK_MODE, true);
    env.set(&TEXT_COLOR, Color::from_hex("#c8c8c8"));
    env.set(&WINDOW_BACKGROUND_COLOR, Color::from_hex("#515151"));
    env.set(&TEXT_BACKGROUND_COLOR, Color::from_hex("#1e1e1e"));
    env
});

static LIGHT_THEME: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set(&DARK_MODE, false);
    env.set(&TEXT_COLOR, Color::from_hex("#141414"));
    env.set(&WINDOW_BACKGROUND_COLOR, Color::from_hex("#f2f2f2"));
    env.set(&TEXT_BACKGROUND_COLOR, Color::from_hex("#ffffff"));
    env
});

pub fn dark_theme() -> Environment {
    DARK_THEME.clone()
}
pub fn light_theme() -> Environment {
    LIGHT_THEME.clone()
}

pub fn setup_default_style(env: &mut Environment) {
    *env = env.merged(dark_theme());
}

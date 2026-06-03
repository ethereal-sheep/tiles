use crate::font::{Font, Glyph};

// ' ' (32) — empty
#[rustfmt::skip]
static GLYPH_32: &[u8] = &[];
// '!' (33)
#[rustfmt::skip]
static GLYPH_33: &[u8] = &[0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x20];
// '"' (34)
#[rustfmt::skip]
static GLYPH_34: &[u8] = &[0x50, 0x50, 0x50];
// '#' (35)
#[rustfmt::skip]
static GLYPH_35: &[u8] = &[0x50, 0xF8, 0x50, 0x50, 0xF8, 0x50];
// '$' (36)
#[rustfmt::skip]
static GLYPH_36: &[u8] = &[0x20, 0x70, 0xA0, 0x60, 0x30, 0x30, 0xE0, 0x20];
// '%' (37)
#[rustfmt::skip]
static GLYPH_37: &[u8] = &[0x10, 0x90, 0xA0, 0x20, 0x40, 0x50, 0x90, 0x80];
// '&' (38)
#[rustfmt::skip]
static GLYPH_38: &[u8] = &[0x20, 0x50, 0x50, 0x60, 0xA0, 0x90, 0x68];
// '\'' (39)
#[rustfmt::skip]
static GLYPH_39: &[u8] = &[0x20, 0x20, 0x20];
// '(' (40)
#[rustfmt::skip]
static GLYPH_40: &[u8] = &[0x10, 0x20, 0x40, 0x40, 0x40, 0x40, 0x20, 0x10];
// ')' (41)
#[rustfmt::skip]
static GLYPH_41: &[u8] = &[0x40, 0x20, 0x10, 0x10, 0x10, 0x10, 0x20, 0x40];
// '*' (42)
#[rustfmt::skip]
static GLYPH_42: &[u8] = &[0x90, 0x60, 0xF0, 0x60, 0x90];
// '+' (43)
#[rustfmt::skip]
static GLYPH_43: &[u8] = &[0x20, 0x20, 0xF8, 0x20, 0x20];
// ',' (44)
#[rustfmt::skip]
static GLYPH_44: &[u8] = &[0x20, 0x20, 0x40];
// '-' (45)
#[rustfmt::skip]
static GLYPH_45: &[u8] = &[0xF0];
// '.' (46)
#[rustfmt::skip]
static GLYPH_46: &[u8] = &[0x20];
// '/' (47)
#[rustfmt::skip]
static GLYPH_47: &[u8] = &[0x10, 0x10, 0x20, 0x20, 0x40, 0x40, 0x80, 0x80];
// '0' (48)
#[rustfmt::skip]
static GLYPH_48: &[u8] = &[0x60, 0x90, 0xB0, 0xD0, 0x90, 0x60];
// '1' (49)
#[rustfmt::skip]
static GLYPH_49: &[u8] = &[0x20, 0x60, 0x20, 0x20, 0x20, 0x70];
// '2' (50)
#[rustfmt::skip]
static GLYPH_50: &[u8] = &[0x60, 0x90, 0x10, 0x60, 0x80, 0xF0];
// '3' (51)
#[rustfmt::skip]
static GLYPH_51: &[u8] = &[0x60, 0x90, 0x20, 0x10, 0x90, 0x60];
// '4' (52)
#[rustfmt::skip]
static GLYPH_52: &[u8] = &[0x80, 0xA0, 0xA0, 0xF0, 0x20, 0x20];
// '5' (53)
#[rustfmt::skip]
static GLYPH_53: &[u8] = &[0xF0, 0x80, 0xF0, 0x10, 0x10, 0xE0];
// '6' (54)
#[rustfmt::skip]
static GLYPH_54: &[u8] = &[0x60, 0x80, 0xE0, 0x90, 0x90, 0x60];
// '7' (55)
#[rustfmt::skip]
static GLYPH_55: &[u8] = &[0xF0, 0x90, 0x20, 0x20, 0x40, 0x40];
// '8' (56)
#[rustfmt::skip]
static GLYPH_56: &[u8] = &[0x60, 0x90, 0x60, 0x90, 0x90, 0x60];
// '9' (57)
#[rustfmt::skip]
static GLYPH_57: &[u8] = &[0x60, 0x90, 0x90, 0x70, 0x10, 0x60];
// ':' (58)
#[rustfmt::skip]
static GLYPH_58: &[u8] = &[0x20, 0x00, 0x00, 0x20];
// ';' (59)
#[rustfmt::skip]
static GLYPH_59: &[u8] = &[0x20, 0x00, 0x20, 0x20, 0x40];
// '<' (60)
#[rustfmt::skip]
static GLYPH_60: &[u8] = &[0x10, 0x20, 0x40, 0x40, 0x20, 0x10];
// '=' (61)
#[rustfmt::skip]
static GLYPH_61: &[u8] = &[0xF0, 0x00, 0xF0];
// '>' (62)
#[rustfmt::skip]
static GLYPH_62: &[u8] = &[0x40, 0x20, 0x10, 0x10, 0x20, 0x40];
// '?' (63)
#[rustfmt::skip]
static GLYPH_63: &[u8] = &[0x60, 0x90, 0x10, 0x20, 0x40, 0x00, 0x40];
// '@' (64)
#[rustfmt::skip]
static GLYPH_64: &[u8] = &[0x60, 0x90, 0xB0, 0xB0, 0x80, 0x70];
// 'A' (65)
#[rustfmt::skip]
static GLYPH_65: &[u8] = &[0x60, 0x90, 0x90, 0xF0, 0x90, 0x90];
// 'B' (66)
#[rustfmt::skip]
static GLYPH_66: &[u8] = &[0xE0, 0x90, 0xE0, 0x90, 0x90, 0xE0];
// 'C' (67)
#[rustfmt::skip]
static GLYPH_67: &[u8] = &[0x70, 0x80, 0x80, 0x80, 0x80, 0x70];
// 'D' (68)
#[rustfmt::skip]
static GLYPH_68: &[u8] = &[0xE0, 0x90, 0x90, 0x90, 0x90, 0xE0];
// 'E' (69)
#[rustfmt::skip]
static GLYPH_69: &[u8] = &[0x70, 0x80, 0xE0, 0x80, 0x80, 0x70];
// 'F' (70)
#[rustfmt::skip]
static GLYPH_70: &[u8] = &[0x70, 0x80, 0x80, 0xE0, 0x80, 0x80];
// 'G' (71)
#[rustfmt::skip]
static GLYPH_71: &[u8] = &[0x70, 0x80, 0xB0, 0x90, 0x90, 0x70];
// 'H' (72)
#[rustfmt::skip]
static GLYPH_72: &[u8] = &[0x90, 0x90, 0xF0, 0x90, 0x90, 0x90];
// 'I' (73)
#[rustfmt::skip]
static GLYPH_73: &[u8] = &[0x70, 0x20, 0x20, 0x20, 0x20, 0x70];
// 'J' (74)
#[rustfmt::skip]
static GLYPH_74: &[u8] = &[0x70, 0x20, 0x20, 0x20, 0x20, 0xC0];
// 'K' (75)
#[rustfmt::skip]
static GLYPH_75: &[u8] = &[0x90, 0x90, 0xE0, 0x90, 0x90, 0x90];
// 'L' (76)
#[rustfmt::skip]
static GLYPH_76: &[u8] = &[0x80, 0x80, 0x80, 0x80, 0x80, 0x70];
// 'M' (77)
#[rustfmt::skip]
static GLYPH_77: &[u8] = &[0x90, 0xF0, 0xF0, 0x90, 0x90, 0x90];
// 'N' (78)
#[rustfmt::skip]
static GLYPH_78: &[u8] = &[0x90, 0xD0, 0xD0, 0xB0, 0xB0, 0x90];
// 'O' (79)
#[rustfmt::skip]
static GLYPH_79: &[u8] = &[0x60, 0x90, 0x90, 0x90, 0x90, 0x60];
// 'P' (80)
#[rustfmt::skip]
static GLYPH_80: &[u8] = &[0xE0, 0x90, 0x90, 0xE0, 0x80, 0x80];
// 'Q' (81)
#[rustfmt::skip]
static GLYPH_81: &[u8] = &[0x60, 0x90, 0x90, 0x90, 0x90, 0x60, 0x30];
// 'R' (82)
#[rustfmt::skip]
static GLYPH_82: &[u8] = &[0xE0, 0x90, 0x90, 0xE0, 0x90, 0x90];
// 'S' (83)
#[rustfmt::skip]
static GLYPH_83: &[u8] = &[0x70, 0x80, 0x60, 0x10, 0x10, 0xE0];
// 'T' (84)
#[rustfmt::skip]
static GLYPH_84: &[u8] = &[0xF8, 0x20, 0x20, 0x20, 0x20, 0x20];
// 'U' (85)
#[rustfmt::skip]
static GLYPH_85: &[u8] = &[0x90, 0x90, 0x90, 0x90, 0x90, 0x70];
// 'V' (86)
#[rustfmt::skip]
static GLYPH_86: &[u8] = &[0x90, 0x90, 0x90, 0x90, 0x60, 0x60];
// 'W' (87)
#[rustfmt::skip]
static GLYPH_87: &[u8] = &[0x90, 0x90, 0x90, 0xF0, 0xF0, 0x90];
// 'X' (88)
#[rustfmt::skip]
static GLYPH_88: &[u8] = &[0x90, 0x90, 0x60, 0x60, 0x90, 0x90];
// 'Y' (89)
#[rustfmt::skip]
static GLYPH_89: &[u8] = &[0x90, 0x90, 0x90, 0x70, 0x10, 0xE0];
// 'Z' (90)
#[rustfmt::skip]
static GLYPH_90: &[u8] = &[0xF0, 0x10, 0x20, 0x40, 0x80, 0xF0];
// '[' (91)
#[rustfmt::skip]
static GLYPH_91: &[u8] = &[0x70, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x70];
// '\\' (92)
#[rustfmt::skip]
static GLYPH_92: &[u8] = &[0x80, 0x80, 0x40, 0x40, 0x20, 0x20, 0x10, 0x10];
// ']' (93)
#[rustfmt::skip]
static GLYPH_93: &[u8] = &[0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x70];
// '^' (94)
#[rustfmt::skip]
static GLYPH_94: &[u8] = &[0x20, 0x50, 0x88];
// '_' (95)
#[rustfmt::skip]
static GLYPH_95: &[u8] = &[0xF0];
// '`' (96)
#[rustfmt::skip]
static GLYPH_96: &[u8] = &[0x40, 0x20];
// 'a' (97)
#[rustfmt::skip]
static GLYPH_97: &[u8] = &[0x60, 0x10, 0x70, 0x90, 0x70];
// 'b' (98)
#[rustfmt::skip]
static GLYPH_98: &[u8] = &[0x80, 0x80, 0xE0, 0x90, 0x90, 0x90, 0xE0];
// 'c' (99)
#[rustfmt::skip]
static GLYPH_99: &[u8] = &[0x70, 0x80, 0x80, 0x80, 0x70];
// 'd' (100)
#[rustfmt::skip]
static GLYPH_100: &[u8] = &[0x10, 0x10, 0x70, 0x90, 0x90, 0x90, 0x70];
// 'e' (101)
#[rustfmt::skip]
static GLYPH_101: &[u8] = &[0x70, 0x90, 0xF0, 0x80, 0x70];
// 'f' (102)
#[rustfmt::skip]
static GLYPH_102: &[u8] = &[0x30, 0x40, 0x40, 0xE0, 0x40, 0x40, 0x40];
// 'g' (103)
#[rustfmt::skip]
static GLYPH_103: &[u8] = &[0x70, 0x90, 0x90, 0x60, 0x10, 0xE0];
// 'h' (104)
#[rustfmt::skip]
static GLYPH_104: &[u8] = &[0x80, 0x80, 0xE0, 0x90, 0x90, 0x90, 0x90];
// 'i' (105)
#[rustfmt::skip]
static GLYPH_105: &[u8] = &[0x20, 0x00, 0x20, 0x20, 0x20, 0x20];
// 'j' (106)
#[rustfmt::skip]
static GLYPH_106: &[u8] = &[0x20, 0x00, 0x20, 0x20, 0x20, 0x20, 0xC0];
// 'k' (107)
#[rustfmt::skip]
static GLYPH_107: &[u8] = &[0x80, 0x80, 0x90, 0xA0, 0xC0, 0xA0, 0x90];
// 'l' (108)
#[rustfmt::skip]
static GLYPH_108: &[u8] = &[0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x30];
// 'm' (109)
#[rustfmt::skip]
static GLYPH_109: &[u8] = &[0x90, 0xF0, 0xF0, 0x90, 0x90];
// 'n' (110)
#[rustfmt::skip]
static GLYPH_110: &[u8] = &[0xE0, 0x90, 0x90, 0x90, 0x90];
// 'o' (111)
#[rustfmt::skip]
static GLYPH_111: &[u8] = &[0x60, 0x90, 0x90, 0x90, 0x60];
// 'p' (112)
#[rustfmt::skip]
static GLYPH_112: &[u8] = &[0xE0, 0x90, 0x90, 0xE0, 0x80, 0x80];
// 'q' (113)
#[rustfmt::skip]
static GLYPH_113: &[u8] = &[0x70, 0x90, 0x90, 0x70, 0x10, 0x10];
// 'r' (114)
#[rustfmt::skip]
static GLYPH_114: &[u8] = &[0x70, 0x90, 0x80, 0x80, 0x80];
// 's' (115)
#[rustfmt::skip]
static GLYPH_115: &[u8] = &[0x70, 0x80, 0x60, 0x10, 0xE0];
// 't' (116)
#[rustfmt::skip]
static GLYPH_116: &[u8] = &[0x40, 0x40, 0xE0, 0x40, 0x40, 0x40, 0x30];
// 'u' (117)
#[rustfmt::skip]
static GLYPH_117: &[u8] = &[0x90, 0x90, 0x90, 0x90, 0x70];
// 'v' (118)
#[rustfmt::skip]
static GLYPH_118: &[u8] = &[0x90, 0x90, 0x90, 0x60, 0x60];
// 'w' (119)
#[rustfmt::skip]
static GLYPH_119: &[u8] = &[0x90, 0x90, 0xF0, 0xF0, 0x90];
// 'x' (120)
#[rustfmt::skip]
static GLYPH_120: &[u8] = &[0x90, 0x60, 0x60, 0x90, 0x90];
// 'y' (121)
#[rustfmt::skip]
static GLYPH_121: &[u8] = &[0x90, 0x90, 0x90, 0x70, 0x10, 0xE0];
// 'z' (122)
#[rustfmt::skip]
static GLYPH_122: &[u8] = &[0xF0, 0x10, 0x20, 0x40, 0xF0];
// '{' (123)
#[rustfmt::skip]
static GLYPH_123: &[u8] = &[0x30, 0x40, 0x40, 0xC0, 0xC0, 0x40, 0x40, 0x30];
// '|' (124)
#[rustfmt::skip]
static GLYPH_124: &[u8] = &[0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
// '}' (125)
#[rustfmt::skip]
static GLYPH_125: &[u8] = &[0xC0, 0x20, 0x20, 0x30, 0x30, 0x20, 0x20, 0xC0];
// '~' (126)
#[rustfmt::skip]
static GLYPH_126: &[u8] = &[0x48, 0xB0];

#[rustfmt::skip]
static GLYPHS: [Glyph; 95] = [
    Glyph { width: 5, height: 0, top: 0, bytes_per_row: 1, data: GLYPH_32 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_33 },
    Glyph { width: 5, height: 3, top: 0, bytes_per_row: 1, data: GLYPH_34 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_35 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_36 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_37 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_38 },
    Glyph { width: 5, height: 3, top: 0, bytes_per_row: 1, data: GLYPH_39 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_40 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_41 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_42 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_43 },
    Glyph { width: 5, height: 3, top: 5, bytes_per_row: 1, data: GLYPH_44 },
    Glyph { width: 5, height: 1, top: 4, bytes_per_row: 1, data: GLYPH_45 },
    Glyph { width: 5, height: 1, top: 6, bytes_per_row: 1, data: GLYPH_46 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_47 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_48 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_49 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_50 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_51 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_52 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_53 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_54 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_55 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_56 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_57 },
    Glyph { width: 5, height: 4, top: 3, bytes_per_row: 1, data: GLYPH_58 },
    Glyph { width: 5, height: 5, top: 3, bytes_per_row: 1, data: GLYPH_59 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_60 },
    Glyph { width: 5, height: 3, top: 3, bytes_per_row: 1, data: GLYPH_61 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_62 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_63 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_64 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_65 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_66 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_67 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_68 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_69 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_70 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_71 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_72 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_73 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_74 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_75 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_76 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_77 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_78 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_79 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_80 },
    Glyph { width: 5, height: 7, top: 1, bytes_per_row: 1, data: GLYPH_81 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_82 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_83 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_84 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_85 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_86 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_87 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_88 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_89 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_90 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_91 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_92 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_93 },
    Glyph { width: 5, height: 3, top: 1, bytes_per_row: 1, data: GLYPH_94 },
    Glyph { width: 5, height: 1, top: 7, bytes_per_row: 1, data: GLYPH_95 },
    Glyph { width: 5, height: 2, top: 0, bytes_per_row: 1, data: GLYPH_96 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_97 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_98 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_99 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_100 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_101 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_102 },
    Glyph { width: 5, height: 6, top: 2, bytes_per_row: 1, data: GLYPH_103 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_104 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_105 },
    Glyph { width: 5, height: 7, top: 1, bytes_per_row: 1, data: GLYPH_106 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_107 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_108 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_109 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_110 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_111 },
    Glyph { width: 5, height: 6, top: 2, bytes_per_row: 1, data: GLYPH_112 },
    Glyph { width: 5, height: 6, top: 2, bytes_per_row: 1, data: GLYPH_113 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_114 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_115 },
    Glyph { width: 5, height: 7, top: 0, bytes_per_row: 1, data: GLYPH_116 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_117 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_118 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_119 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_120 },
    Glyph { width: 5, height: 6, top: 2, bytes_per_row: 1, data: GLYPH_121 },
    Glyph { width: 5, height: 5, top: 2, bytes_per_row: 1, data: GLYPH_122 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_123 },
    Glyph { width: 5, height: 6, top: 1, bytes_per_row: 1, data: GLYPH_124 },
    Glyph { width: 5, height: 8, top: 0, bytes_per_row: 1, data: GLYPH_125 },
    Glyph { width: 5, height: 2, top: 1, bytes_per_row: 1, data: GLYPH_126 },
];

pub static MONO_SPLEEN_5X8: Font = Font::new(8, 0, &GLYPHS);

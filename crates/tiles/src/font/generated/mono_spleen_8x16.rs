use crate::font::{Font, Glyph};

// ' ' (32) — empty
#[rustfmt::skip]
static GLYPH_32: &[u8] = &[];
// '!' (33)
#[rustfmt::skip]
static GLYPH_33: &[u8] = &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18];
// '"' (34)
#[rustfmt::skip]
static GLYPH_34: &[u8] = &[0x66, 0x66, 0x66, 0x66];
// '#' (35)
#[rustfmt::skip]
static GLYPH_35: &[u8] = &[0x6C, 0x6C, 0xFE, 0x6C, 0x6C, 0x6C, 0x6C, 0xFE, 0x6C, 0x6C];
// '$' (36)
#[rustfmt::skip]
static GLYPH_36: &[u8] = &[0x10, 0x7E, 0xD0, 0xD0, 0xD0, 0x7C, 0x16, 0x16, 0x16, 0x16, 0xFC, 0x10];
// '%' (37)
#[rustfmt::skip]
static GLYPH_37: &[u8] = &[0x06, 0x66, 0x6C, 0x0C, 0x18, 0x18, 0x30, 0x36, 0x66, 0x60];
// '&' (38)
#[rustfmt::skip]
static GLYPH_38: &[u8] = &[0x38, 0x6C, 0x6C, 0x6C, 0x38, 0x70, 0xDA, 0xCC, 0xCC, 0x7A];
// '\'' (39)
#[rustfmt::skip]
static GLYPH_39: &[u8] = &[0x18, 0x18, 0x18, 0x18];
// '(' (40)
#[rustfmt::skip]
static GLYPH_40: &[u8] = &[0x0E, 0x18, 0x30, 0x30, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x30, 0x30, 0x18, 0x0E];
// ')' (41)
#[rustfmt::skip]
static GLYPH_41: &[u8] = &[0x70, 0x18, 0x0C, 0x0C, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x0C, 0x0C, 0x18, 0x70];
// '*' (42)
#[rustfmt::skip]
static GLYPH_42: &[u8] = &[0x66, 0x3C, 0x18, 0xFF, 0x18, 0x3C, 0x66];
// '+' (43)
#[rustfmt::skip]
static GLYPH_43: &[u8] = &[0x18, 0x18, 0x7E, 0x18, 0x18];
// ',' (44)
#[rustfmt::skip]
static GLYPH_44: &[u8] = &[0x18, 0x18, 0x30];
// '-' (45)
#[rustfmt::skip]
static GLYPH_45: &[u8] = &[0x7E];
// '.' (46)
#[rustfmt::skip]
static GLYPH_46: &[u8] = &[0x18, 0x18];
// '/' (47)
#[rustfmt::skip]
static GLYPH_47: &[u8] = &[0x03, 0x03, 0x06, 0x06, 0x0C, 0x0C, 0x18, 0x18, 0x30, 0x30, 0x60, 0x60, 0xC0, 0xC0];
// '0' (48)
#[rustfmt::skip]
static GLYPH_48: &[u8] = &[0x7C, 0xC6, 0xC6, 0xCE, 0xDE, 0xF6, 0xE6, 0xC6, 0xC6, 0x7C];
// '1' (49)
#[rustfmt::skip]
static GLYPH_49: &[u8] = &[0x18, 0x38, 0x78, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E];
// '2' (50)
#[rustfmt::skip]
static GLYPH_50: &[u8] = &[0x7C, 0xC6, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC6, 0xFE];
// '3' (51)
#[rustfmt::skip]
static GLYPH_51: &[u8] = &[0x7C, 0xC6, 0x06, 0x06, 0x3C, 0x06, 0x06, 0x06, 0xC6, 0x7C];
// '4' (52)
#[rustfmt::skip]
static GLYPH_52: &[u8] = &[0xC0, 0xC0, 0xCC, 0xCC, 0xCC, 0xCC, 0xFE, 0x0C, 0x0C, 0x0C];
// '5' (53)
#[rustfmt::skip]
static GLYPH_53: &[u8] = &[0xFE, 0xC6, 0xC0, 0xC0, 0xFC, 0x06, 0x06, 0x06, 0xC6, 0x7C];
// '6' (54)
#[rustfmt::skip]
static GLYPH_54: &[u8] = &[0x7C, 0xC6, 0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C];
// '7' (55)
#[rustfmt::skip]
static GLYPH_55: &[u8] = &[0xFE, 0xC6, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x30];
// '8' (56)
#[rustfmt::skip]
static GLYPH_56: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C];
// '9' (57)
#[rustfmt::skip]
static GLYPH_57: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0xC6, 0x7C];
// ':' (58)
#[rustfmt::skip]
static GLYPH_58: &[u8] = &[0x18, 0x18, 0x00, 0x00, 0x00, 0x18, 0x18];
// ';' (59)
#[rustfmt::skip]
static GLYPH_59: &[u8] = &[0x18, 0x18, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30];
// '<' (60)
#[rustfmt::skip]
static GLYPH_60: &[u8] = &[0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x30, 0x18, 0x0C, 0x06];
// '=' (61)
#[rustfmt::skip]
static GLYPH_61: &[u8] = &[0x7E, 0x00, 0x00, 0x7E];
// '>' (62)
#[rustfmt::skip]
static GLYPH_62: &[u8] = &[0x60, 0x30, 0x18, 0x0C, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x60];
// '?' (63)
#[rustfmt::skip]
static GLYPH_63: &[u8] = &[0x7C, 0xC6, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x00, 0x30, 0x30];
// '@' (64)
#[rustfmt::skip]
static GLYPH_64: &[u8] = &[0x7C, 0xC2, 0xDA, 0xDA, 0xDA, 0xDA, 0xDE, 0xC0, 0x7C];
// 'A' (65)
#[rustfmt::skip]
static GLYPH_65: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'B' (66)
#[rustfmt::skip]
static GLYPH_66: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xFC];
// 'C' (67)
#[rustfmt::skip]
static GLYPH_67: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0x7E];
// 'D' (68)
#[rustfmt::skip]
static GLYPH_68: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xFC];
// 'E' (69)
#[rustfmt::skip]
static GLYPH_69: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xC0, 0xC0, 0x7E];
// 'F' (70)
#[rustfmt::skip]
static GLYPH_70: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0];
// 'G' (71)
#[rustfmt::skip]
static GLYPH_71: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0xDE, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E];
// 'H' (72)
#[rustfmt::skip]
static GLYPH_72: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'I' (73)
#[rustfmt::skip]
static GLYPH_73: &[u8] = &[0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E];
// 'J' (74)
#[rustfmt::skip]
static GLYPH_74: &[u8] = &[0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0xF0];
// 'K' (75)
#[rustfmt::skip]
static GLYPH_75: &[u8] = &[0xC6, 0xC6, 0xC6, 0xCC, 0xF8, 0xCC, 0xC6, 0xC6, 0xC6, 0xC6];
// 'L' (76)
#[rustfmt::skip]
static GLYPH_76: &[u8] = &[0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0x7E];
// 'M' (77)
#[rustfmt::skip]
static GLYPH_77: &[u8] = &[0xC6, 0xEE, 0xFE, 0xD6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'N' (78)
#[rustfmt::skip]
static GLYPH_78: &[u8] = &[0xC6, 0xC6, 0xE6, 0xE6, 0xD6, 0xD6, 0xCE, 0xCE, 0xC6, 0xC6];
// 'O' (79)
#[rustfmt::skip]
static GLYPH_79: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C];
// 'P' (80)
#[rustfmt::skip]
static GLYPH_80: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xFC, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0];
// 'Q' (81)
#[rustfmt::skip]
static GLYPH_81: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xD6, 0xD6, 0x7C, 0x18, 0x0C];
// 'R' (82)
#[rustfmt::skip]
static GLYPH_82: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'S' (83)
#[rustfmt::skip]
static GLYPH_83: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0x7C, 0x06, 0x06, 0x06, 0x06, 0xFC];
// 'T' (84)
#[rustfmt::skip]
static GLYPH_84: &[u8] = &[0xFF, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18];
// 'U' (85)
#[rustfmt::skip]
static GLYPH_85: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E];
// 'V' (86)
#[rustfmt::skip]
static GLYPH_86: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10];
// 'W' (87)
#[rustfmt::skip]
static GLYPH_87: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xD6, 0xFE, 0xEE, 0xC6];
// 'X' (88)
#[rustfmt::skip]
static GLYPH_88: &[u8] = &[0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0xC6, 0xC6, 0xC6];
// 'Y' (89)
#[rustfmt::skip]
static GLYPH_89: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0x06, 0x06, 0xFC];
// 'Z' (90)
#[rustfmt::skip]
static GLYPH_90: &[u8] = &[0xFE, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0xC0, 0xFE];
// '[' (91)
#[rustfmt::skip]
static GLYPH_91: &[u8] = &[0x3E, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3E];
// '\\' (92)
#[rustfmt::skip]
static GLYPH_92: &[u8] = &[0xC0, 0xC0, 0x60, 0x60, 0x30, 0x30, 0x18, 0x18, 0x0C, 0x0C, 0x06, 0x06, 0x03, 0x03];
// ']' (93)
#[rustfmt::skip]
static GLYPH_93: &[u8] = &[0x7C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x7C];
// '^' (94)
#[rustfmt::skip]
static GLYPH_94: &[u8] = &[0x10, 0x38, 0x6C, 0xC6];
// '_' (95)
#[rustfmt::skip]
static GLYPH_95: &[u8] = &[0xFE];
// '`' (96)
#[rustfmt::skip]
static GLYPH_96: &[u8] = &[0x30, 0x18, 0x0C];
// 'a' (97)
#[rustfmt::skip]
static GLYPH_97: &[u8] = &[0x7C, 0x06, 0x7E, 0xC6, 0xC6, 0xC6, 0x7E];
// 'b' (98)
#[rustfmt::skip]
static GLYPH_98: &[u8] = &[0xC0, 0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xFC];
// 'c' (99)
#[rustfmt::skip]
static GLYPH_99: &[u8] = &[0x7E, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0x7E];
// 'd' (100)
#[rustfmt::skip]
static GLYPH_100: &[u8] = &[0x06, 0x06, 0x06, 0x7E, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E];
// 'e' (101)
#[rustfmt::skip]
static GLYPH_101: &[u8] = &[0x7E, 0xC6, 0xC6, 0xFE, 0xC0, 0xC0, 0x7E];
// 'f' (102)
#[rustfmt::skip]
static GLYPH_102: &[u8] = &[0x1E, 0x30, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0x30];
// 'g' (103)
#[rustfmt::skip]
static GLYPH_103: &[u8] = &[0x7E, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x06, 0x06, 0xFC];
// 'h' (104)
#[rustfmt::skip]
static GLYPH_104: &[u8] = &[0xC0, 0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'i' (105)
#[rustfmt::skip]
static GLYPH_105: &[u8] = &[0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18];
// 'j' (106)
#[rustfmt::skip]
static GLYPH_106: &[u8] = &[0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x70];
// 'k' (107)
#[rustfmt::skip]
static GLYPH_107: &[u8] = &[0xC0, 0xC0, 0xC0, 0xCC, 0xD8, 0xF0, 0xF0, 0xD8, 0xCC, 0xC6];
// 'l' (108)
#[rustfmt::skip]
static GLYPH_108: &[u8] = &[0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x1C];
// 'm' (109)
#[rustfmt::skip]
static GLYPH_109: &[u8] = &[0xEC, 0xD6, 0xD6, 0xD6, 0xD6, 0xC6, 0xC6];
// 'n' (110)
#[rustfmt::skip]
static GLYPH_110: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6];
// 'o' (111)
#[rustfmt::skip]
static GLYPH_111: &[u8] = &[0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C];
// 'p' (112)
#[rustfmt::skip]
static GLYPH_112: &[u8] = &[0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xFC, 0xC0, 0xC0, 0xC0];
// 'q' (113)
#[rustfmt::skip]
static GLYPH_113: &[u8] = &[0x7E, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0x06];
// 'r' (114)
#[rustfmt::skip]
static GLYPH_114: &[u8] = &[0x7E, 0xC6, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0];
// 's' (115)
#[rustfmt::skip]
static GLYPH_115: &[u8] = &[0x7E, 0xC0, 0xC0, 0x7C, 0x06, 0x06, 0xFC];
// 't' (116)
#[rustfmt::skip]
static GLYPH_116: &[u8] = &[0x30, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x1E];
// 'u' (117)
#[rustfmt::skip]
static GLYPH_117: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E];
// 'v' (118)
#[rustfmt::skip]
static GLYPH_118: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10];
// 'w' (119)
#[rustfmt::skip]
static GLYPH_119: &[u8] = &[0xC6, 0xC6, 0xD6, 0xD6, 0xD6, 0xD6, 0x6E];
// 'x' (120)
#[rustfmt::skip]
static GLYPH_120: &[u8] = &[0xC6, 0x6C, 0x38, 0x38, 0x6C, 0xC6, 0xC6];
// 'y' (121)
#[rustfmt::skip]
static GLYPH_121: &[u8] = &[0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0xFC];
// 'z' (122)
#[rustfmt::skip]
static GLYPH_122: &[u8] = &[0xFE, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFE];
// '{' (123)
#[rustfmt::skip]
static GLYPH_123: &[u8] = &[0x0E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x70, 0x70, 0x18, 0x18, 0x18, 0x18, 0x18, 0x0E];
// '|' (124)
#[rustfmt::skip]
static GLYPH_124: &[u8] = &[0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18];
// '}' (125)
#[rustfmt::skip]
static GLYPH_125: &[u8] = &[0x70, 0x18, 0x18, 0x18, 0x18, 0x18, 0x0E, 0x0E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x70];
// '~' (126)
#[rustfmt::skip]
static GLYPH_126: &[u8] = &[0x32, 0x7E, 0x4C];

#[rustfmt::skip]
static GLYPHS: [Glyph; 95] = [
    Glyph { width: 8, height: 0, top: 0, bytes_per_row: 1, data: GLYPH_32 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_33 },
    Glyph { width: 8, height: 4, top: 1, bytes_per_row: 1, data: GLYPH_34 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_35 },
    Glyph { width: 8, height: 12, top: 1, bytes_per_row: 1, data: GLYPH_36 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_37 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_38 },
    Glyph { width: 8, height: 4, top: 1, bytes_per_row: 1, data: GLYPH_39 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_40 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_41 },
    Glyph { width: 8, height: 7, top: 4, bytes_per_row: 1, data: GLYPH_42 },
    Glyph { width: 8, height: 5, top: 5, bytes_per_row: 1, data: GLYPH_43 },
    Glyph { width: 8, height: 3, top: 10, bytes_per_row: 1, data: GLYPH_44 },
    Glyph { width: 8, height: 1, top: 7, bytes_per_row: 1, data: GLYPH_45 },
    Glyph { width: 8, height: 2, top: 10, bytes_per_row: 1, data: GLYPH_46 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_47 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_48 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_49 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_50 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_51 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_52 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_53 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_54 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_55 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_56 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_57 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_58 },
    Glyph { width: 8, height: 8, top: 5, bytes_per_row: 1, data: GLYPH_59 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_60 },
    Glyph { width: 8, height: 4, top: 5, bytes_per_row: 1, data: GLYPH_61 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_62 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_63 },
    Glyph { width: 8, height: 9, top: 3, bytes_per_row: 1, data: GLYPH_64 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_65 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_66 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_67 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_68 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_69 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_70 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_71 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_72 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_73 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_74 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_75 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_76 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_77 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_78 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_79 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_80 },
    Glyph { width: 8, height: 12, top: 2, bytes_per_row: 1, data: GLYPH_81 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_82 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_83 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_84 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_85 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_86 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_87 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_88 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_89 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_90 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_91 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_92 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_93 },
    Glyph { width: 8, height: 4, top: 1, bytes_per_row: 1, data: GLYPH_94 },
    Glyph { width: 8, height: 1, top: 14, bytes_per_row: 1, data: GLYPH_95 },
    Glyph { width: 8, height: 3, top: 1, bytes_per_row: 1, data: GLYPH_96 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_97 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_98 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_99 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_100 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_101 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_102 },
    Glyph { width: 8, height: 10, top: 5, bytes_per_row: 1, data: GLYPH_103 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_104 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_105 },
    Glyph { width: 8, height: 13, top: 2, bytes_per_row: 1, data: GLYPH_106 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_107 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_108 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_109 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_110 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_111 },
    Glyph { width: 8, height: 10, top: 5, bytes_per_row: 1, data: GLYPH_112 },
    Glyph { width: 8, height: 10, top: 5, bytes_per_row: 1, data: GLYPH_113 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_114 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_115 },
    Glyph { width: 8, height: 10, top: 2, bytes_per_row: 1, data: GLYPH_116 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_117 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_118 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_119 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_120 },
    Glyph { width: 8, height: 10, top: 5, bytes_per_row: 1, data: GLYPH_121 },
    Glyph { width: 8, height: 7, top: 5, bytes_per_row: 1, data: GLYPH_122 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_123 },
    Glyph { width: 8, height: 12, top: 2, bytes_per_row: 1, data: GLYPH_124 },
    Glyph { width: 8, height: 14, top: 1, bytes_per_row: 1, data: GLYPH_125 },
    Glyph { width: 8, height: 3, top: 1, bytes_per_row: 1, data: GLYPH_126 },
];

pub static MONO_SPLEEN_8X16: Font = Font::new(16, 0, &GLYPHS);

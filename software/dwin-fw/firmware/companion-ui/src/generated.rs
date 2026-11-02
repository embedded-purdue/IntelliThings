// generated file
#![allow(dead_code)]

use crate::text::GlyphStyle;

/// basic graphics control
pub struct Part {
    pub icl: u8,
    pub vp: u16,
    pub capacity: u16,
    pub first: u16,
    pub end: u16,
}

pub mod page {
    pub const BOOT: u16 = 0;
    pub const HOME: u16 = 1;
    pub const SENSORS: u16 = 2;
    pub const ROOMS: u16 = 3;
    pub const ALERT: u16 = 4;
    pub const NIGHT: u16 = 5;
    pub const AWAY: u16 = 6;
    pub const SETUP: u16 = 7;
    pub const TEST: u16 = 8;
}

pub static SKIN: [Part; 1] = [
    Part { icl: 34, vp: 0x5000, capacity: 64, first: 0, end: 3894 },
];
pub static MAIN: [Part; 2] = [
    Part { icl: 34, vp: 0x5100, capacity: 200, first: 0, end: 3894 },
    Part { icl: 34, vp: 0x535b, capacity: 200, first: 0, end: 3894 },
];
pub static OVER: [Part; 1] = [
    Part { icl: 34, vp: 0x5600, capacity: 100, first: 0, end: 3894 },
];

pub const TREND_ROWS: usize = 6;
pub const TREND_POINTS: usize = 72;
/// trend polylines
pub const TREND_VP: [u16; 12] = [0x5800, 0x5900, 0x5a00, 0x5b00, 0x5c00, 0x5d00, 0x5e00, 0x5f00, 0x6000, 0x6100, 0x6200, 0x6300];
pub const QR_VP: u16 = 0x6400;
pub const QR_MAX_BYTES: usize = 150; // max payload
pub const SCRIM_VP: u16 = 0x6500;
/// session marker
pub const SESSION_VP: u16 = 0x6501;
/// signature marker
pub const SIGNATURE_VP: u16 = 0x6502;
pub const SIGNATURE_VALUE: u16 = 0xc0de;
pub const VP_BASE: u16 = 0x5000;
pub const VP_WORDS: usize = 0x1501;

/// room tiles
pub static ROOMS: [(i16, i16, i16, i16, bool); 5] = [
    (24, 60, 400, 232, true), // living room
    (432, 60, 344, 150, true), // kitchen
    (432, 218, 208, 204, true), // bedroom
    (648, 218, 128, 204, false), // office
    (24, 300, 400, 122, false), // entry
];
pub const ROOM_NAMES: [&str; 5] = ["Living Room", "Kitchen", "Bedroom", "Office", "Entry"];

pub mod sprite {
    pub const PERSON: u16 = 0;
    pub const PERSON_FAINT: u16 = 1;
    pub const WIFI: u16 = 2;
    pub const WIFI_OFF: u16 = 3;
    pub const HUB: u16 = 4;
    pub const HUB_OFF: u16 = 5;
    pub const HUB_OFF_DOT: u16 = 6;
    pub const ALERT_PERSON: u16 = 7;
    pub const ALERT_WIFI: u16 = 8;
    pub const ALERT_HUB: u16 = 9;
    pub const TILE_TEMP_MODERATE: u16 = 10;
    pub const TILE_TEMP_POOR: u16 = 11;
    pub const TILE_TEMP_BAD: u16 = 12;
    pub const TILE_HUMIDITY_MODERATE: u16 = 13;
    pub const TILE_HUMIDITY_POOR: u16 = 14;
    pub const TILE_HUMIDITY_BAD: u16 = 15;
    pub const TILE_CO2_MODERATE: u16 = 16;
    pub const TILE_CO2_POOR: u16 = 17;
    pub const TILE_CO2_BAD: u16 = 18;
    pub const TILE_PM_MODERATE: u16 = 19;
    pub const TILE_PM_POOR: u16 = 20;
    pub const TILE_PM_BAD: u16 = 21;
    pub const TILE_VOC_MODERATE: u16 = 22;
    pub const TILE_VOC_POOR: u16 = 23;
    pub const TILE_VOC_BAD: u16 = 24;
    pub const TILE_LIGHT_MODERATE: u16 = 25;
    pub const TILE_LIGHT_POOR: u16 = 26;
    pub const TILE_LIGHT_BAD: u16 = 27;
    pub const STATE_DOT_GOOD: u16 = 28;
    pub const STATE_DOT_MODERATE: u16 = 29;
    pub const STATE_DOT_POOR: u16 = 30;
    pub const STATE_DOT_BAD: u16 = 31;
    pub const STATE_DOT_NEUTRAL: u16 = 32;
    pub const ROW_TEMP_MODERATE: u16 = 33;
    pub const ROW_TEMP_POOR: u16 = 34;
    pub const ROW_TEMP_BAD: u16 = 35;
    pub const ROW_HUMIDITY_MODERATE: u16 = 36;
    pub const ROW_HUMIDITY_POOR: u16 = 37;
    pub const ROW_HUMIDITY_BAD: u16 = 38;
    pub const ROW_CO2_MODERATE: u16 = 39;
    pub const ROW_CO2_POOR: u16 = 40;
    pub const ROW_CO2_BAD: u16 = 41;
    pub const ROW_PM_MODERATE: u16 = 42;
    pub const ROW_PM_POOR: u16 = 43;
    pub const ROW_PM_BAD: u16 = 44;
    pub const ROW_VOC_MODERATE: u16 = 45;
    pub const ROW_VOC_POOR: u16 = 46;
    pub const ROW_VOC_BAD: u16 = 47;
    pub const ROW_LIGHT_MODERATE: u16 = 48;
    pub const ROW_LIGHT_POOR: u16 = 49;
    pub const ROW_LIGHT_BAD: u16 = 50;
    pub const ROW_DOT_GOOD: u16 = 51;
    pub const ROW_DOT_MODERATE: u16 = 52;
    pub const ROW_DOT_POOR: u16 = 53;
    pub const ROW_DOT_BAD: u16 = 54;
    pub const ROW_DOT_NEUTRAL: u16 = 55;
    pub const PULSE_GOOD_0: u16 = 56;
    pub const PULSE_GOOD_1: u16 = 57;
    pub const PULSE_GOOD_2: u16 = 58;
    pub const PULSE_GOOD_3: u16 = 59;
    pub const PULSE_GOOD_4: u16 = 60;
    pub const PULSE_GOOD_5: u16 = 61;
    pub const PULSE_GOOD_6: u16 = 62;
    pub const PULSE_GOOD_7: u16 = 63;
    pub const PULSE_MODERATE_0: u16 = 64;
    pub const PULSE_MODERATE_1: u16 = 65;
    pub const PULSE_MODERATE_2: u16 = 66;
    pub const PULSE_MODERATE_3: u16 = 67;
    pub const PULSE_MODERATE_4: u16 = 68;
    pub const PULSE_MODERATE_5: u16 = 69;
    pub const PULSE_MODERATE_6: u16 = 70;
    pub const PULSE_MODERATE_7: u16 = 71;
    pub const PULSE_POOR_0: u16 = 72;
    pub const PULSE_POOR_1: u16 = 73;
    pub const PULSE_POOR_2: u16 = 74;
    pub const PULSE_POOR_3: u16 = 75;
    pub const PULSE_POOR_4: u16 = 76;
    pub const PULSE_POOR_5: u16 = 77;
    pub const PULSE_POOR_6: u16 = 78;
    pub const PULSE_POOR_7: u16 = 79;
    pub const PULSE_BAD_0: u16 = 80;
    pub const PULSE_BAD_1: u16 = 81;
    pub const PULSE_BAD_2: u16 = 82;
    pub const PULSE_BAD_3: u16 = 83;
    pub const PULSE_BAD_4: u16 = 84;
    pub const PULSE_BAD_5: u16 = 85;
    pub const PULSE_BAD_6: u16 = 86;
    pub const PULSE_BAD_7: u16 = 87;
    pub const PULSE_NEUTRAL_0: u16 = 88;
    pub const PULSE_NEUTRAL_1: u16 = 89;
    pub const PULSE_NEUTRAL_2: u16 = 90;
    pub const PULSE_NEUTRAL_3: u16 = 91;
    pub const PULSE_NEUTRAL_4: u16 = 92;
    pub const PULSE_NEUTRAL_5: u16 = 93;
    pub const PULSE_NEUTRAL_6: u16 = 94;
    pub const PULSE_NEUTRAL_7: u16 = 95;
    pub const ROOM0_GOOD_HERE: u16 = 96;
    pub const ROOM0_MODERATE: u16 = 97;
    pub const ROOM0_MODERATE_HERE: u16 = 98;
    pub const ROOM0_POOR: u16 = 99;
    pub const ROOM0_POOR_HERE: u16 = 100;
    pub const ROOM0_BAD: u16 = 101;
    pub const ROOM0_BAD_HERE: u16 = 102;
    pub const ROOM1_GOOD_HERE: u16 = 103;
    pub const ROOM1_MODERATE: u16 = 104;
    pub const ROOM1_MODERATE_HERE: u16 = 105;
    pub const ROOM1_POOR: u16 = 106;
    pub const ROOM1_POOR_HERE: u16 = 107;
    pub const ROOM1_BAD: u16 = 108;
    pub const ROOM1_BAD_HERE: u16 = 109;
    pub const ROOM2_GOOD_HERE: u16 = 110;
    pub const ROOM2_MODERATE: u16 = 111;
    pub const ROOM2_MODERATE_HERE: u16 = 112;
    pub const ROOM2_POOR: u16 = 113;
    pub const ROOM2_POOR_HERE: u16 = 114;
    pub const ROOM2_BAD: u16 = 115;
    pub const ROOM2_BAD_HERE: u16 = 116;
    pub const ROOM_DOT_GOOD: u16 = 117;
    pub const ROOM_DOT_MODERATE: u16 = 118;
    pub const ROOM_DOT_POOR: u16 = 119;
    pub const ROOM_DOT_BAD: u16 = 120;
    pub const ROOM_DOT_NEUTRAL: u16 = 121;
    pub const ROOM_RING_GOOD_0: u16 = 122;
    pub const ROOM_RING_GOOD_1: u16 = 123;
    pub const ROOM_RING_GOOD_2: u16 = 124;
    pub const ROOM_RING_GOOD_3: u16 = 125;
    pub const ROOM_RING_GOOD_4: u16 = 126;
    pub const ROOM_RING_GOOD_5: u16 = 127;
    pub const ROOM_RING_GOOD_6: u16 = 128;
    pub const ROOM_RING_GOOD_7: u16 = 129;
    pub const ROOM_RING_MODERATE_0: u16 = 130;
    pub const ROOM_RING_MODERATE_1: u16 = 131;
    pub const ROOM_RING_MODERATE_2: u16 = 132;
    pub const ROOM_RING_MODERATE_3: u16 = 133;
    pub const ROOM_RING_MODERATE_4: u16 = 134;
    pub const ROOM_RING_MODERATE_5: u16 = 135;
    pub const ROOM_RING_MODERATE_6: u16 = 136;
    pub const ROOM_RING_MODERATE_7: u16 = 137;
    pub const ROOM_RING_POOR_0: u16 = 138;
    pub const ROOM_RING_POOR_1: u16 = 139;
    pub const ROOM_RING_POOR_2: u16 = 140;
    pub const ROOM_RING_POOR_3: u16 = 141;
    pub const ROOM_RING_POOR_4: u16 = 142;
    pub const ROOM_RING_POOR_5: u16 = 143;
    pub const ROOM_RING_POOR_6: u16 = 144;
    pub const ROOM_RING_POOR_7: u16 = 145;
    pub const ROOM_RING_BAD_0: u16 = 146;
    pub const ROOM_RING_BAD_1: u16 = 147;
    pub const ROOM_RING_BAD_2: u16 = 148;
    pub const ROOM_RING_BAD_3: u16 = 149;
    pub const ROOM_RING_BAD_4: u16 = 150;
    pub const ROOM_RING_BAD_5: u16 = 151;
    pub const ROOM_RING_BAD_6: u16 = 152;
    pub const ROOM_RING_BAD_7: u16 = 153;
    pub const ROOM_PERSON_FAINT_GOOD: u16 = 154;
    pub const ROOM_PERSON_FAINT_MODERATE: u16 = 155;
    pub const ROOM_PERSON_FAINT_POOR: u16 = 156;
    pub const ROOM_PERSON_FAINT_BAD: u16 = 157;
    pub const ROOM_BULB: u16 = 158;
    pub const ORB_IDLE_0: u16 = 159;
    pub const ORB_IDLE_1: u16 = 160;
    pub const ORB_IDLE_2: u16 = 161;
    pub const ORB_IDLE_3: u16 = 162;
    pub const ORB_IDLE_4: u16 = 163;
    pub const ORB_IDLE_5: u16 = 164;
    pub const ORB_IDLE_6: u16 = 165;
    pub const ORB_IDLE_7: u16 = 166;
    pub const ORB_IDLE_8: u16 = 167;
    pub const ORB_IDLE_9: u16 = 168;
    pub const ORB_IDLE_10: u16 = 169;
    pub const ORB_IDLE_11: u16 = 170;
    pub const ORB_IDLE_12: u16 = 171;
    pub const ORB_IDLE_13: u16 = 172;
    pub const ORB_IDLE_14: u16 = 173;
    pub const ORB_IDLE_15: u16 = 174;
    pub const ORB_THINKING_0: u16 = 175;
    pub const ORB_THINKING_1: u16 = 176;
    pub const ORB_THINKING_2: u16 = 177;
    pub const ORB_THINKING_3: u16 = 178;
    pub const ORB_THINKING_4: u16 = 179;
    pub const ORB_THINKING_5: u16 = 180;
    pub const ORB_THINKING_6: u16 = 181;
    pub const ORB_THINKING_7: u16 = 182;
    pub const ORB_THINKING_8: u16 = 183;
    pub const ORB_THINKING_9: u16 = 184;
    pub const ORB_THINKING_10: u16 = 185;
    pub const ORB_THINKING_11: u16 = 186;
    pub const ORB_THINKING_12: u16 = 187;
    pub const ORB_THINKING_13: u16 = 188;
    pub const ORB_THINKING_14: u16 = 189;
    pub const ORB_THINKING_15: u16 = 190;
    pub const ORB_ALERT_0: u16 = 191;
    pub const ORB_ALERT_1: u16 = 192;
    pub const ORB_ALERT_2: u16 = 193;
    pub const ORB_ALERT_3: u16 = 194;
    pub const ORB_ALERT_4: u16 = 195;
    pub const ORB_ALERT_5: u16 = 196;
    pub const ORB_ALERT_6: u16 = 197;
    pub const ORB_ALERT_7: u16 = 198;
    pub const ORB_ALERT_8: u16 = 199;
    pub const ORB_ALERT_9: u16 = 200;
    pub const ORB_ALERT_10: u16 = 201;
    pub const ORB_ALERT_11: u16 = 202;
    pub const ORB_ALERT_12: u16 = 203;
    pub const ORB_ALERT_13: u16 = 204;
    pub const ORB_ALERT_14: u16 = 205;
    pub const ORB_ALERT_15: u16 = 206;
    pub const ORB_OFFLINE_0: u16 = 207;
    pub const ALERT_ORB_IDLE_0: u16 = 208;
    pub const ALERT_ORB_IDLE_1: u16 = 209;
    pub const ALERT_ORB_IDLE_2: u16 = 210;
    pub const ALERT_ORB_IDLE_3: u16 = 211;
    pub const ALERT_ORB_IDLE_4: u16 = 212;
    pub const ALERT_ORB_IDLE_5: u16 = 213;
    pub const ALERT_ORB_IDLE_6: u16 = 214;
    pub const ALERT_ORB_IDLE_7: u16 = 215;
    pub const ALERT_ORB_IDLE_8: u16 = 216;
    pub const ALERT_ORB_IDLE_9: u16 = 217;
    pub const ALERT_ORB_IDLE_10: u16 = 218;
    pub const ALERT_ORB_IDLE_11: u16 = 219;
    pub const ALERT_ORB_IDLE_12: u16 = 220;
    pub const ALERT_ORB_IDLE_13: u16 = 221;
    pub const ALERT_ORB_IDLE_14: u16 = 222;
    pub const ALERT_ORB_IDLE_15: u16 = 223;
    pub const ALERT_ORB_THINKING_0: u16 = 224;
    pub const ALERT_ORB_THINKING_1: u16 = 225;
    pub const ALERT_ORB_THINKING_2: u16 = 226;
    pub const ALERT_ORB_THINKING_3: u16 = 227;
    pub const ALERT_ORB_THINKING_4: u16 = 228;
    pub const ALERT_ORB_THINKING_5: u16 = 229;
    pub const ALERT_ORB_THINKING_6: u16 = 230;
    pub const ALERT_ORB_THINKING_7: u16 = 231;
    pub const ALERT_ORB_THINKING_8: u16 = 232;
    pub const ALERT_ORB_THINKING_9: u16 = 233;
    pub const ALERT_ORB_THINKING_10: u16 = 234;
    pub const ALERT_ORB_THINKING_11: u16 = 235;
    pub const ALERT_ORB_THINKING_12: u16 = 236;
    pub const ALERT_ORB_THINKING_13: u16 = 237;
    pub const ALERT_ORB_THINKING_14: u16 = 238;
    pub const ALERT_ORB_THINKING_15: u16 = 239;
    pub const ALERT_ORB_ALERT_0: u16 = 240;
    pub const ALERT_ORB_ALERT_1: u16 = 241;
    pub const ALERT_ORB_ALERT_2: u16 = 242;
    pub const ALERT_ORB_ALERT_3: u16 = 243;
    pub const ALERT_ORB_ALERT_4: u16 = 244;
    pub const ALERT_ORB_ALERT_5: u16 = 245;
    pub const ALERT_ORB_ALERT_6: u16 = 246;
    pub const ALERT_ORB_ALERT_7: u16 = 247;
    pub const ALERT_ORB_ALERT_8: u16 = 248;
    pub const ALERT_ORB_ALERT_9: u16 = 249;
    pub const ALERT_ORB_ALERT_10: u16 = 250;
    pub const ALERT_ORB_ALERT_11: u16 = 251;
    pub const ALERT_ORB_ALERT_12: u16 = 252;
    pub const ALERT_ORB_ALERT_13: u16 = 253;
    pub const ALERT_ORB_ALERT_14: u16 = 254;
    pub const ALERT_ORB_ALERT_15: u16 = 255;
    pub const ALERT_ORB_OFFLINE_0: u16 = 256;
    pub const BOOT_ORB_IDLE_0: u16 = 257;
    pub const BOOT_ORB_IDLE_1: u16 = 258;
    pub const BOOT_ORB_IDLE_2: u16 = 259;
    pub const BOOT_ORB_IDLE_3: u16 = 260;
    pub const BOOT_ORB_IDLE_4: u16 = 261;
    pub const BOOT_ORB_IDLE_5: u16 = 262;
    pub const BOOT_ORB_IDLE_6: u16 = 263;
    pub const BOOT_ORB_IDLE_7: u16 = 264;
    pub const BOOT_ORB_IDLE_8: u16 = 265;
    pub const BOOT_ORB_IDLE_9: u16 = 266;
    pub const BOOT_ORB_IDLE_10: u16 = 267;
    pub const BOOT_ORB_IDLE_11: u16 = 268;
    pub const BOOT_ORB_IDLE_12: u16 = 269;
    pub const BOOT_ORB_IDLE_13: u16 = 270;
    pub const BOOT_ORB_IDLE_14: u16 = 271;
    pub const BOOT_ORB_IDLE_15: u16 = 272;
    pub const BOOT_ORB_THINKING_0: u16 = 273;
    pub const BOOT_ORB_THINKING_1: u16 = 274;
    pub const BOOT_ORB_THINKING_2: u16 = 275;
    pub const BOOT_ORB_THINKING_3: u16 = 276;
    pub const BOOT_ORB_THINKING_4: u16 = 277;
    pub const BOOT_ORB_THINKING_5: u16 = 278;
    pub const BOOT_ORB_THINKING_6: u16 = 279;
    pub const BOOT_ORB_THINKING_7: u16 = 280;
    pub const BOOT_ORB_THINKING_8: u16 = 281;
    pub const BOOT_ORB_THINKING_9: u16 = 282;
    pub const BOOT_ORB_THINKING_10: u16 = 283;
    pub const BOOT_ORB_THINKING_11: u16 = 284;
    pub const BOOT_ORB_THINKING_12: u16 = 285;
    pub const BOOT_ORB_THINKING_13: u16 = 286;
    pub const BOOT_ORB_THINKING_14: u16 = 287;
    pub const BOOT_ORB_THINKING_15: u16 = 288;
    pub const BOOT_ORB_ALERT_0: u16 = 289;
    pub const BOOT_ORB_ALERT_1: u16 = 290;
    pub const BOOT_ORB_ALERT_2: u16 = 291;
    pub const BOOT_ORB_ALERT_3: u16 = 292;
    pub const BOOT_ORB_ALERT_4: u16 = 293;
    pub const BOOT_ORB_ALERT_5: u16 = 294;
    pub const BOOT_ORB_ALERT_6: u16 = 295;
    pub const BOOT_ORB_ALERT_7: u16 = 296;
    pub const BOOT_ORB_ALERT_8: u16 = 297;
    pub const BOOT_ORB_ALERT_9: u16 = 298;
    pub const BOOT_ORB_ALERT_10: u16 = 299;
    pub const BOOT_ORB_ALERT_11: u16 = 300;
    pub const BOOT_ORB_ALERT_12: u16 = 301;
    pub const BOOT_ORB_ALERT_13: u16 = 302;
    pub const BOOT_ORB_ALERT_14: u16 = 303;
    pub const BOOT_ORB_ALERT_15: u16 = 304;
    pub const BOOT_ORB_OFFLINE_0: u16 = 305;
    pub const CARET: u16 = 306;
    pub const CHIP_CARD_ACT_L: u16 = 307;
    pub const CHIP_CARD_ACT_M: u16 = 308;
    pub const CHIP_CARD_ACT_R: u16 = 309;
    pub const CHIP_CARD_ACT_FAN: u16 = 310;
    pub const CHIP_CARD_ACT_BULB: u16 = 311;
    pub const CHIP_CARD_ACT_THERMOSTAT: u16 = 312;
    pub const CHIP_CARD_ACT_PURIFIER: u16 = 313;
    pub const CHIP_CARD_ACT_HUMIDIFIER: u16 = 314;
    pub const CHIP_CARD_ACT_WINDOW: u16 = 315;
    pub const CHIP_CARD_ACT_DOOR: u16 = 316;
    pub const CHIP_CARD_ACT_LOCK: u16 = 317;
    pub const CHIP_CARD_ACT_ARROW: u16 = 318;
    pub const CHIP_CARD_ACT_CHECK: u16 = 319;
    pub const CHIP_CARD_ASK_L: u16 = 320;
    pub const CHIP_CARD_ASK_M: u16 = 321;
    pub const CHIP_CARD_ASK_R: u16 = 322;
    pub const CHIP_CARD_ASK_FAN: u16 = 323;
    pub const CHIP_CARD_ASK_BULB: u16 = 324;
    pub const CHIP_CARD_ASK_THERMOSTAT: u16 = 325;
    pub const CHIP_CARD_ASK_PURIFIER: u16 = 326;
    pub const CHIP_CARD_ASK_HUMIDIFIER: u16 = 327;
    pub const CHIP_CARD_ASK_WINDOW: u16 = 328;
    pub const CHIP_CARD_ASK_DOOR: u16 = 329;
    pub const CHIP_CARD_ASK_LOCK: u16 = 330;
    pub const CHIP_CARD_ASK_ARROW: u16 = 331;
    pub const CHIP_CARD_ASK_CHECK: u16 = 332;
    pub const CHIP_ALERT_ACT_L: u16 = 333;
    pub const CHIP_ALERT_ACT_M: u16 = 334;
    pub const CHIP_ALERT_ACT_R: u16 = 335;
    pub const CHIP_ALERT_ACT_FAN: u16 = 336;
    pub const CHIP_ALERT_ACT_BULB: u16 = 337;
    pub const CHIP_ALERT_ACT_THERMOSTAT: u16 = 338;
    pub const CHIP_ALERT_ACT_PURIFIER: u16 = 339;
    pub const CHIP_ALERT_ACT_HUMIDIFIER: u16 = 340;
    pub const CHIP_ALERT_ACT_WINDOW: u16 = 341;
    pub const CHIP_ALERT_ACT_DOOR: u16 = 342;
    pub const CHIP_ALERT_ACT_LOCK: u16 = 343;
    pub const CHIP_ALERT_ACT_ARROW: u16 = 344;
    pub const CHIP_ALERT_ACT_CHECK: u16 = 345;
    pub const CHIP_ALERT_ASK_L: u16 = 346;
    pub const CHIP_ALERT_ASK_M: u16 = 347;
    pub const CHIP_ALERT_ASK_R: u16 = 348;
    pub const CHIP_ALERT_ASK_FAN: u16 = 349;
    pub const CHIP_ALERT_ASK_BULB: u16 = 350;
    pub const CHIP_ALERT_ASK_THERMOSTAT: u16 = 351;
    pub const CHIP_ALERT_ASK_PURIFIER: u16 = 352;
    pub const CHIP_ALERT_ASK_HUMIDIFIER: u16 = 353;
    pub const CHIP_ALERT_ASK_WINDOW: u16 = 354;
    pub const CHIP_ALERT_ASK_DOOR: u16 = 355;
    pub const CHIP_ALERT_ASK_LOCK: u16 = 356;
    pub const CHIP_ALERT_ASK_ARROW: u16 = 357;
    pub const CHIP_ALERT_ASK_CHECK: u16 = 358;
    pub const CHIP_TOAST_ACT_L: u16 = 359;
    pub const CHIP_TOAST_ACT_M: u16 = 360;
    pub const CHIP_TOAST_ACT_R: u16 = 361;
    pub const CHIP_TOAST_ACT_FAN: u16 = 362;
    pub const CHIP_TOAST_ACT_BULB: u16 = 363;
    pub const CHIP_TOAST_ACT_THERMOSTAT: u16 = 364;
    pub const CHIP_TOAST_ACT_PURIFIER: u16 = 365;
    pub const CHIP_TOAST_ACT_HUMIDIFIER: u16 = 366;
    pub const CHIP_TOAST_ACT_WINDOW: u16 = 367;
    pub const CHIP_TOAST_ACT_DOOR: u16 = 368;
    pub const CHIP_TOAST_ACT_LOCK: u16 = 369;
    pub const CHIP_TOAST_ACT_ARROW: u16 = 370;
    pub const CHIP_TOAST_ACT_CHECK: u16 = 371;
    pub const CHIP_TOAST_ASK_L: u16 = 372;
    pub const CHIP_TOAST_ASK_M: u16 = 373;
    pub const CHIP_TOAST_ASK_R: u16 = 374;
    pub const CHIP_TOAST_ASK_FAN: u16 = 375;
    pub const CHIP_TOAST_ASK_BULB: u16 = 376;
    pub const CHIP_TOAST_ASK_THERMOSTAT: u16 = 377;
    pub const CHIP_TOAST_ASK_PURIFIER: u16 = 378;
    pub const CHIP_TOAST_ASK_HUMIDIFIER: u16 = 379;
    pub const CHIP_TOAST_ASK_WINDOW: u16 = 380;
    pub const CHIP_TOAST_ASK_DOOR: u16 = 381;
    pub const CHIP_TOAST_ASK_LOCK: u16 = 382;
    pub const CHIP_TOAST_ASK_ARROW: u16 = 383;
    pub const CHIP_TOAST_ASK_CHECK: u16 = 384;
    pub const TOAST_CARD: u16 = 385;
    pub const TOAST_FAN: u16 = 386;
    pub const TOAST_BULB: u16 = 387;
    pub const TOAST_THERMOSTAT: u16 = 388;
    pub const TOAST_PURIFIER: u16 = 389;
    pub const TOAST_HUMIDIFIER: u16 = 390;
    pub const TOAST_WINDOW: u16 = 391;
    pub const TOAST_DOOR: u16 = 392;
    pub const TOAST_LOCK: u16 = 393;
    pub const ALERT_EDGE_T_0: u16 = 394;
    pub const ALERT_EDGE_B_0: u16 = 395;
    pub const ALERT_EDGE_L_0: u16 = 396;
    pub const ALERT_EDGE_R_0: u16 = 397;
    pub const ALERT_EDGE_T_1: u16 = 398;
    pub const ALERT_EDGE_B_1: u16 = 399;
    pub const ALERT_EDGE_L_1: u16 = 400;
    pub const ALERT_EDGE_R_1: u16 = 401;
    pub const ALERT_EDGE_T_2: u16 = 402;
    pub const ALERT_EDGE_B_2: u16 = 403;
    pub const ALERT_EDGE_L_2: u16 = 404;
    pub const ALERT_EDGE_R_2: u16 = 405;
    pub const ALERT_EDGE_T_3: u16 = 406;
    pub const ALERT_EDGE_B_3: u16 = 407;
    pub const ALERT_EDGE_L_3: u16 = 408;
    pub const ALERT_EDGE_R_3: u16 = 409;
    pub const ALERT_EDGE_T_4: u16 = 410;
    pub const ALERT_EDGE_B_4: u16 = 411;
    pub const ALERT_EDGE_L_4: u16 = 412;
    pub const ALERT_EDGE_R_4: u16 = 413;
    pub const ALERT_EDGE_T_5: u16 = 414;
    pub const ALERT_EDGE_B_5: u16 = 415;
    pub const ALERT_EDGE_L_5: u16 = 416;
    pub const ALERT_EDGE_R_5: u16 = 417;
    pub const BOOT_SPIN_0: u16 = 418;
    pub const BOOT_SPIN_1: u16 = 419;
    pub const BOOT_SPIN_2: u16 = 420;
    pub const BOOT_SPIN_3: u16 = 421;
    pub const BOOT_SPIN_4: u16 = 422;
    pub const BOOT_SPIN_5: u16 = 423;
    pub const BOOT_SPIN_6: u16 = 424;
    pub const BOOT_SPIN_7: u16 = 425;
    pub const BOOT_SPIN_8: u16 = 426;
    pub const BOOT_SPIN_9: u16 = 427;
    pub const BOOT_SPIN_10: u16 = 428;
    pub const BOOT_SPIN_11: u16 = 429;
    pub const BOOT_CHECK: u16 = 430;
    pub const BOOT_PENDING: u16 = 431;
    pub const TEST_MARKER: u16 = 432;
}

/// sprite dimensions
pub static SPRITE_SIZE: [(u16, u16); 433] = [
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (9, 9),
    (20, 20),
    (20, 20),
    (20, 20),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (176, 124),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (18, 18),
    (400, 232),
    (400, 232),
    (400, 232),
    (400, 232),
    (400, 232),
    (400, 232),
    (400, 232),
    (344, 150),
    (344, 150),
    (344, 150),
    (344, 150),
    (344, 150),
    (344, 150),
    (344, 150),
    (208, 204),
    (208, 204),
    (208, 204),
    (208, 204),
    (208, 204),
    (208, 204),
    (208, 204),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (11, 11),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (52, 52),
    (24, 24),
    (24, 24),
    (24, 24),
    (24, 24),
    (20, 20),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (46, 46),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (43, 43),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (135, 135),
    (3, 25),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (18, 34),
    (16, 34),
    (18, 34),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (20, 20),
    (18, 18),
    (18, 18),
    (520, 76),
    (28, 28),
    (28, 28),
    (28, 28),
    (28, 28),
    (28, 28),
    (28, 28),
    (28, 28),
    (28, 28),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (752, 22),
    (752, 22),
    (4, 352),
    (4, 352),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (22, 22),
    (20, 20),
    (10, 10),
    (20, 20),
];

const CHARS_0: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_0: [u16; 78] = [72, 264, 216, 80, 96, 96, 192, 72, 96, 72, 112, 176, 176, 176, 176, 176, 176, 176, 176, 176, 176, 88, 200, 184, 200, 224, 176, 168, 224, 216, 80, 144, 200, 144, 264, 224, 248, 176, 248, 184, 168, 160, 208, 192, 288, 192, 176, 176, 160, 184, 144, 184, 168, 96, 184, 168, 80, 80, 160, 80, 256, 168, 176, 184, 184, 104, 128, 96, 168, 144, 232, 152, 144, 136, 112, 72, 288, 104];
static OFF_0: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_0: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_1: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_1: [u16; 78] = [56, 192, 160, 56, 72, 72, 152, 56, 72, 56, 88, 136, 136, 136, 136, 136, 136, 136, 136, 136, 136, 64, 160, 144, 160, 168, 136, 128, 176, 160, 64, 112, 144, 112, 200, 176, 192, 136, 192, 136, 128, 128, 160, 144, 224, 152, 136, 136, 120, 144, 112, 144, 128, 72, 144, 128, 56, 56, 120, 56, 200, 128, 136, 144, 144, 80, 96, 72, 128, 112, 176, 112, 112, 104, 88, 56, 224, 80];
static OFF_1: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_1: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_2: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_2: [u16; 78] = [56, 192, 160, 56, 72, 72, 152, 56, 72, 56, 88, 136, 136, 136, 136, 136, 136, 136, 136, 136, 136, 64, 160, 144, 160, 168, 136, 128, 176, 160, 64, 112, 144, 112, 200, 176, 192, 136, 192, 136, 128, 128, 160, 144, 224, 152, 136, 136, 120, 144, 112, 144, 128, 72, 144, 128, 56, 56, 120, 56, 200, 128, 136, 144, 144, 80, 96, 72, 128, 112, 176, 112, 112, 104, 88, 56, 224, 80];
static OFF_2: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_2: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_3: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_3: [u16; 78] = [72, 264, 216, 80, 96, 96, 192, 72, 96, 72, 112, 176, 176, 176, 176, 176, 176, 176, 176, 176, 176, 88, 200, 184, 200, 224, 176, 168, 224, 216, 80, 144, 200, 144, 264, 224, 248, 176, 248, 184, 168, 160, 208, 192, 288, 192, 176, 176, 160, 184, 144, 184, 168, 96, 184, 168, 80, 80, 160, 80, 256, 168, 176, 184, 184, 104, 128, 96, 168, 144, 232, 152, 144, 136, 112, 72, 288, 104];
static OFF_3: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_3: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_4: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_4: [u16; 78] = [56, 192, 160, 56, 72, 72, 152, 56, 72, 56, 88, 136, 136, 136, 136, 136, 136, 136, 136, 136, 136, 64, 160, 144, 160, 168, 136, 128, 176, 160, 64, 112, 144, 112, 200, 176, 192, 136, 192, 136, 128, 128, 160, 144, 224, 152, 136, 136, 120, 144, 112, 144, 128, 72, 144, 128, 56, 56, 120, 56, 200, 128, 136, 144, 144, 80, 96, 72, 128, 112, 176, 112, 112, 104, 88, 56, 224, 80];
static OFF_4: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_4: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_5: &str = " %+,-.0123456789:−";
static ADV_5: [u16; 18] = [416, 1448, 1112, 432, 536, 432, 992, 992, 992, 992, 992, 992, 992, 992, 992, 992, 496, 1112];
static OFF_5: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_5: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_6: &str = "AMP";
static ADV_6: [u16; 3] = [320, 416, 272];
static OFF_6: [i8; 3] = [-1, 0, 0];
static ASCII_6: [u8; 96] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_7: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_7: [u16; 78] = [80, 280, 232, 80, 104, 104, 216, 80, 104, 80, 120, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 96, 224, 208, 224, 240, 192, 184, 248, 232, 88, 160, 208, 160, 288, 248, 272, 192, 272, 200, 184, 184, 224, 208, 320, 216, 200, 192, 176, 200, 160, 200, 184, 96, 200, 184, 80, 80, 168, 80, 280, 184, 192, 200, 200, 112, 144, 104, 184, 160, 248, 160, 160, 144, 128, 80, 320, 120];
static OFF_7: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_7: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_8: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_8: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_8: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_8: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_9: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_9: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_9: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_9: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_10: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_10: [u16; 78] = [56, 192, 160, 56, 72, 72, 152, 56, 72, 56, 88, 136, 136, 136, 136, 136, 136, 136, 136, 136, 136, 64, 160, 144, 160, 168, 136, 128, 176, 160, 64, 112, 144, 112, 200, 176, 192, 136, 192, 136, 128, 128, 160, 144, 224, 152, 136, 136, 120, 144, 112, 144, 128, 72, 144, 128, 56, 56, 120, 56, 200, 128, 136, 144, 144, 80, 96, 72, 128, 112, 176, 112, 112, 104, 88, 56, 224, 80];
static OFF_10: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_10: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_11: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_11: [u16; 109] = [88, 112, 144, 192, 200, 296, 240, 88, 104, 104, 152, 224, 88, 104, 88, 128, 200, 200, 200, 200, 200, 200, 200, 200, 200, 200, 104, 104, 224, 224, 224, 168, 272, 240, 216, 240, 256, 200, 192, 264, 248, 88, 168, 224, 168, 304, 256, 288, 200, 288, 208, 192, 192, 240, 216, 336, 224, 208, 200, 104, 128, 104, 224, 168, 88, 184, 216, 168, 216, 192, 104, 216, 200, 88, 88, 176, 88, 296, 192, 208, 216, 216, 120, 152, 112, 192, 168, 256, 168, 168, 152, 104, 72, 104, 224, 136, 128, 192, 88, 224, 136, 200, 80, 80, 144, 144, 336, 120, 224];
static OFF_11: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_11: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_12: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_12: [u16; 109] = [80, 112, 144, 192, 192, 296, 240, 88, 104, 104, 152, 216, 80, 104, 80, 128, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 96, 96, 216, 216, 216, 168, 256, 224, 208, 224, 248, 192, 184, 248, 240, 88, 160, 224, 160, 296, 248, 272, 192, 272, 200, 184, 184, 232, 208, 320, 208, 200, 192, 104, 128, 104, 216, 160, 80, 176, 200, 160, 200, 184, 104, 200, 184, 88, 88, 176, 88, 288, 184, 192, 200, 200, 120, 144, 112, 184, 168, 256, 168, 168, 152, 104, 72, 104, 216, 128, 128, 184, 80, 216, 128, 192, 88, 88, 152, 152, 320, 120, 216];
static OFF_12: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_12: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_13: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_13: [u16; 109] = [64, 80, 104, 136, 144, 208, 176, 64, 72, 72, 112, 160, 64, 80, 64, 88, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 72, 160, 160, 160, 120, 192, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 160, 120, 216, 184, 200, 144, 200, 144, 136, 136, 168, 152, 240, 160, 152, 144, 72, 88, 72, 160, 120, 64, 128, 152, 120, 152, 136, 72, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 184, 120, 120, 112, 72, 56, 72, 160, 96, 96, 136, 64, 160, 96, 144, 56, 56, 104, 104, 240, 88, 160];
static OFF_13: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_13: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_14: &str = " %+,-.0123456789:−";
static ADV_14: [u16; 18] = [104, 384, 280, 112, 136, 112, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 128, 280];
static OFF_14: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_14: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_15: &str = " %/CFdegilmnpx°³µ";
static ADV_15: [u16; 17] = [56, 192, 88, 160, 128, 144, 128, 144, 56, 56, 200, 128, 144, 112, 88, 88, 128];
static OFF_15: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_15: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_16: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_16: [u16; 109] = [56, 72, 96, 128, 136, 192, 160, 56, 72, 72, 104, 152, 56, 72, 56, 88, 136, 136, 136, 136, 136, 136, 136, 136, 136, 136, 64, 64, 152, 152, 152, 112, 176, 160, 144, 160, 168, 136, 128, 176, 160, 64, 112, 144, 112, 200, 176, 192, 136, 192, 136, 128, 128, 160, 144, 224, 152, 136, 136, 72, 88, 72, 152, 112, 56, 120, 144, 112, 144, 128, 72, 144, 128, 56, 56, 120, 56, 200, 128, 136, 144, 144, 80, 96, 72, 128, 112, 176, 112, 112, 104, 72, 48, 72, 152, 88, 88, 128, 56, 152, 88, 136, 56, 56, 96, 96, 224, 80, 152];
static OFF_16: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_16: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_17: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_17: [u16; 109] = [112, 152, 208, 264, 272, 416, 336, 128, 144, 144, 208, 296, 120, 144, 120, 176, 272, 272, 272, 272, 272, 272, 272, 272, 272, 272, 136, 136, 296, 296, 296, 232, 360, 320, 288, 312, 344, 272, 256, 344, 328, 128, 224, 312, 232, 416, 352, 384, 272, 384, 280, 256, 256, 320, 296, 448, 296, 280, 272, 144, 176, 144, 296, 224, 120, 248, 288, 216, 288, 256, 144, 280, 264, 120, 120, 240, 120, 400, 264, 272, 280, 280, 160, 200, 152, 264, 232, 360, 240, 232, 216, 144, 96, 144, 296, 176, 176, 256, 120, 296, 176, 272, 120, 120, 208, 208, 448, 168, 296];
static OFF_17: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_17: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_18: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_18: [u16; 109] = [64, 88, 112, 144, 152, 224, 184, 64, 80, 80, 120, 168, 64, 80, 64, 96, 152, 152, 152, 152, 152, 152, 152, 152, 152, 152, 80, 80, 168, 168, 168, 128, 208, 184, 168, 184, 192, 152, 144, 200, 184, 72, 128, 168, 128, 232, 200, 216, 152, 216, 160, 144, 144, 184, 168, 256, 168, 160, 152, 80, 96, 80, 168, 128, 64, 136, 160, 128, 160, 144, 80, 160, 152, 64, 64, 136, 64, 224, 152, 160, 160, 160, 96, 112, 88, 152, 128, 200, 128, 128, 120, 80, 56, 80, 168, 104, 96, 152, 64, 168, 104, 152, 64, 64, 112, 112, 256, 96, 168];
static OFF_18: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_18: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_19: &str = " %+,-.0123456789:−";
static ADV_19: [u16; 18] = [448, 1560, 1192, 464, 576, 464, 1064, 1064, 1064, 1064, 1064, 1064, 1064, 1064, 1064, 1064, 536, 1192];
static OFF_19: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_19: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_20: &str = " %/CFdegilmnpx°³µ";
static ADV_20: [u16; 17] = [88, 304, 136, 248, 200, 224, 200, 224, 88, 88, 312, 208, 224, 176, 144, 136, 200];
static OFF_20: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_20: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_21: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_21: [u16; 78] = [88, 328, 264, 96, 120, 120, 232, 88, 112, 88, 136, 216, 216, 216, 216, 216, 216, 216, 216, 216, 216, 104, 248, 224, 248, 272, 216, 200, 272, 264, 96, 176, 240, 176, 320, 272, 296, 216, 296, 224, 200, 200, 248, 232, 352, 232, 216, 216, 192, 224, 168, 224, 200, 112, 224, 208, 96, 96, 192, 96, 312, 208, 216, 224, 224, 128, 160, 120, 208, 184, 280, 192, 184, 168, 144, 88, 352, 128];
static OFF_21: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_21: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_22: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_22: [u16; 109] = [72, 96, 120, 168, 168, 248, 208, 72, 88, 88, 128, 192, 72, 96, 72, 112, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 88, 192, 192, 192, 144, 232, 208, 184, 200, 216, 176, 160, 224, 208, 80, 144, 192, 144, 264, 224, 248, 168, 240, 176, 160, 160, 208, 184, 288, 192, 176, 168, 88, 112, 88, 192, 144, 72, 160, 184, 144, 184, 168, 88, 184, 168, 72, 72, 152, 72, 256, 168, 176, 184, 184, 104, 128, 96, 168, 144, 224, 144, 144, 136, 88, 64, 88, 192, 112, 112, 168, 72, 192, 112, 176, 72, 72, 120, 120, 288, 104, 192];
static OFF_22: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_22: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_23: &str = " %+,-.0123456789:−";
static ADV_23: [u16; 18] = [600, 1968, 1600, 576, 720, 576, 1344, 1344, 1344, 1344, 1344, 1344, 1344, 1344, 1344, 1344, 672, 1600];
static OFF_23: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_23: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_24: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_24: [u16; 109] = [80, 104, 136, 184, 192, 280, 232, 80, 104, 104, 144, 216, 80, 104, 80, 120, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 96, 96, 216, 216, 216, 160, 256, 224, 208, 224, 240, 192, 184, 248, 232, 88, 160, 208, 160, 288, 248, 272, 192, 272, 200, 184, 184, 224, 208, 320, 216, 200, 192, 104, 120, 104, 216, 160, 80, 176, 200, 160, 200, 184, 96, 200, 184, 80, 80, 168, 80, 280, 184, 192, 200, 200, 112, 144, 104, 184, 160, 248, 160, 160, 144, 104, 72, 104, 216, 128, 120, 184, 80, 216, 128, 192, 80, 80, 136, 136, 320, 120, 216];
static OFF_24: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_24: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_25: &str = " %+,-.0123456789:−";
static ADV_25: [u16; 18] = [336, 1104, 896, 320, 400, 320, 752, 752, 752, 752, 752, 752, 752, 752, 752, 752, 376, 896];
static OFF_25: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_25: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_26: &str = "AMP";
static ADV_26: [u16; 3] = [248, 320, 216];
static OFF_26: [i8; 3] = [-1, 0, 0];
static ASCII_26: [u8; 96] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_27: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_27: [u16; 109] = [72, 96, 120, 168, 168, 248, 208, 72, 88, 88, 128, 192, 72, 96, 72, 112, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 88, 192, 192, 192, 144, 232, 208, 184, 200, 216, 176, 160, 224, 208, 80, 144, 192, 144, 264, 224, 248, 168, 240, 176, 160, 160, 208, 184, 288, 192, 176, 168, 88, 112, 88, 192, 144, 72, 160, 184, 144, 184, 168, 88, 184, 168, 72, 72, 152, 72, 256, 168, 176, 184, 184, 104, 128, 96, 168, 144, 224, 144, 144, 136, 88, 64, 88, 192, 112, 112, 168, 72, 192, 112, 176, 72, 72, 120, 120, 288, 104, 192];
static OFF_27: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_27: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_28: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_28: [u16; 40] = [72, 88, 192, 176, 192, 208, 168, 152, 208, 200, 136, 248, 168, 232, 160, 152, 176, 272, 152, 176, 136, 176, 160, 88, 168, 160, 72, 144, 72, 240, 160, 168, 176, 96, 120, 96, 160, 136, 216, 136];
static OFF_28: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_28: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_29: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_29: [u16; 40] = [72, 88, 192, 176, 192, 208, 168, 152, 208, 200, 136, 248, 168, 232, 160, 152, 176, 272, 152, 176, 136, 176, 160, 88, 168, 160, 72, 144, 72, 240, 160, 168, 176, 96, 120, 96, 160, 136, 216, 136];
static OFF_29: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_29: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_30: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_30: [u16; 109] = [64, 80, 104, 136, 144, 208, 176, 64, 72, 72, 112, 160, 64, 80, 64, 88, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 72, 160, 160, 160, 120, 192, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 160, 120, 216, 184, 200, 144, 200, 144, 136, 136, 168, 152, 240, 160, 152, 144, 72, 88, 72, 160, 120, 64, 128, 152, 120, 152, 136, 72, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 184, 120, 120, 112, 72, 56, 72, 160, 96, 96, 136, 64, 160, 96, 144, 56, 56, 104, 104, 240, 88, 160];
static OFF_30: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_30: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_31: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_31: [u16; 78] = [56, 184, 152, 56, 64, 64, 136, 56, 64, 56, 80, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 64, 144, 136, 144, 160, 128, 120, 160, 152, 56, 104, 136, 104, 184, 160, 176, 120, 176, 128, 120, 120, 152, 136, 208, 136, 128, 120, 112, 136, 104, 136, 120, 64, 128, 120, 56, 56, 112, 56, 184, 120, 128, 128, 128, 72, 96, 72, 120, 104, 160, 104, 104, 96, 80, 56, 208, 80];
static OFF_31: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_31: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_32: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°³µ·×–—‘’“”…₂−";
static ADV_32: [u16; 109] = [112, 152, 208, 264, 272, 416, 336, 128, 144, 144, 208, 296, 120, 144, 120, 176, 272, 272, 272, 272, 272, 272, 272, 272, 272, 272, 136, 136, 296, 296, 296, 232, 360, 320, 288, 312, 344, 272, 256, 344, 328, 128, 224, 312, 232, 416, 352, 384, 272, 384, 280, 256, 256, 320, 296, 448, 296, 280, 272, 144, 176, 144, 296, 224, 120, 248, 288, 216, 288, 256, 144, 280, 264, 120, 120, 240, 120, 400, 264, 272, 280, 280, 160, 200, 152, 264, 232, 360, 240, 232, 216, 144, 96, 144, 296, 176, 176, 256, 120, 296, 176, 272, 120, 120, 208, 208, 448, 168, 296];
static OFF_32: [i8; 109] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_32: [u8; 96] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 255];
const CHARS_33: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_33: [u16; 78] = [64, 224, 184, 64, 80, 80, 168, 64, 80, 64, 96, 152, 152, 152, 152, 152, 152, 152, 152, 152, 152, 80, 184, 168, 184, 192, 152, 144, 200, 184, 72, 128, 168, 128, 232, 200, 216, 152, 216, 160, 144, 144, 184, 168, 256, 168, 160, 152, 136, 160, 128, 160, 144, 80, 160, 152, 64, 64, 136, 64, 224, 152, 160, 160, 160, 96, 112, 88, 152, 128, 200, 128, 128, 120, 104, 64, 256, 96];
static OFF_33: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_33: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_34: &str = " %+,-.0123456789:−";
static ADV_34: [u16; 18] = [160, 592, 424, 168, 208, 168, 392, 392, 392, 392, 392, 392, 392, 392, 392, 392, 192, 424];
static OFF_34: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_34: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_35: &str = " %/CFdegilmnpx°³µ";
static ADV_35: [u16; 17] = [64, 224, 96, 184, 144, 160, 144, 160, 64, 64, 224, 152, 160, 128, 104, 96, 152];
static OFF_35: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_35: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_36: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_36: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 216, 144, 200, 136, 136, 152, 240, 128, 152, 120, 152, 136, 72, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 184, 120];
static OFF_36: [i8; 40] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_36: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_37: &str = " %+,-.0123456789:°−";
static ADV_37: [u16; 19] = [256, 888, 680, 264, 328, 264, 608, 608, 608, 608, 608, 608, 608, 608, 608, 608, 304, 408, 680];
static OFF_37: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_37: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_38: &str = " %+,-.0123456789:°−";
static ADV_38: [u16; 19] = [176, 616, 472, 184, 224, 184, 416, 416, 416, 416, 416, 416, 416, 416, 416, 416, 208, 280, 472];
static OFF_38: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_38: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_39: &str = " %+,-.0123456789:−";
static ADV_39: [u16; 18] = [72, 248, 192, 72, 96, 72, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 192];
static OFF_39: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_39: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_40: &str = " %+,-.0123456789:−";
static ADV_40: [u16; 18] = [160, 592, 424, 168, 208, 168, 392, 392, 392, 392, 392, 392, 392, 392, 392, 392, 192, 424];
static OFF_40: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_40: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_41: &str = " %/CFdegilmnpx°³µ";
static ADV_41: [u16; 17] = [64, 224, 96, 184, 144, 160, 144, 160, 64, 64, 224, 152, 160, 128, 104, 96, 152];
static OFF_41: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_41: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_42: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_42: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_42: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_42: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_43: &str = " %+,-.0123456789:°−";
static ADV_43: [u16; 19] = [256, 888, 680, 264, 328, 264, 608, 608, 608, 608, 608, 608, 608, 608, 608, 608, 304, 408, 680];
static OFF_43: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_43: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_44: &str = " %+,-.0123456789:°−";
static ADV_44: [u16; 19] = [176, 616, 472, 184, 224, 184, 416, 416, 416, 416, 416, 416, 416, 416, 416, 416, 208, 280, 472];
static OFF_44: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_44: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_45: &str = " %+,-.0123456789:−";
static ADV_45: [u16; 18] = [72, 248, 192, 72, 96, 72, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 192];
static OFF_45: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_45: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_46: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_46: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_46: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_46: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_47: &str = " %+,-.0123456789:−";
static ADV_47: [u16; 18] = [160, 592, 424, 168, 208, 168, 392, 392, 392, 392, 392, 392, 392, 392, 392, 392, 192, 424];
static OFF_47: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_47: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_48: &str = " %/CFdegilmnpx°³µ";
static ADV_48: [u16; 17] = [64, 224, 96, 184, 144, 160, 144, 160, 64, 64, 224, 152, 160, 128, 104, 96, 152];
static OFF_48: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_48: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_49: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_49: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_49: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_49: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_50: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_50: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_50: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_50: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_51: &str = " %+,-.0123456789:°−";
static ADV_51: [u16; 19] = [256, 888, 680, 264, 328, 264, 608, 608, 608, 608, 608, 608, 608, 608, 608, 608, 304, 408, 680];
static OFF_51: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_51: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_52: &str = " %+,-.0123456789:°−";
static ADV_52: [u16; 19] = [176, 616, 472, 184, 224, 184, 416, 416, 416, 416, 416, 416, 416, 416, 416, 416, 208, 280, 472];
static OFF_52: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_52: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_53: &str = " %+,-.0123456789:−";
static ADV_53: [u16; 18] = [72, 248, 192, 72, 96, 72, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 192];
static OFF_53: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_53: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_54: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_54: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_54: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_54: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_55: &str = " %+,-.0123456789:−";
static ADV_55: [u16; 18] = [160, 592, 424, 168, 208, 168, 392, 392, 392, 392, 392, 392, 392, 392, 392, 392, 192, 424];
static OFF_55: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_55: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_56: &str = " %/CFdegilmnpx°³µ";
static ADV_56: [u16; 17] = [64, 224, 96, 184, 144, 160, 144, 160, 64, 64, 224, 152, 160, 128, 104, 96, 152];
static OFF_56: [i8; 17] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_56: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 255, 255, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 6, 255, 7, 255, 8, 255, 255, 9, 10, 11, 255, 12, 255, 255, 255, 255, 255, 255, 255, 13, 255, 255, 255, 255, 255, 255, 255];
const CHARS_57: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_57: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_57: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_57: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_58: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_58: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_58: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_58: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_59: &str = " %+,-.0123456789:°−";
static ADV_59: [u16; 19] = [256, 888, 680, 264, 328, 264, 608, 608, 608, 608, 608, 608, 608, 608, 608, 608, 304, 408, 680];
static OFF_59: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_59: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_60: &str = " %+,-.0123456789:°−";
static ADV_60: [u16; 19] = [176, 616, 472, 184, 224, 184, 416, 416, 416, 416, 416, 416, 416, 416, 416, 416, 208, 280, 472];
static OFF_60: [i8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_60: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_61: &str = " %+,-.0123456789:−";
static ADV_61: [u16; 18] = [72, 248, 192, 72, 96, 72, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 88, 192];
static OFF_61: [i8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_61: [u8; 96] = [0, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
const CHARS_62: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_62: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_62: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_62: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_63: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_63: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 216, 144, 200, 136, 136, 152, 240, 128, 152, 120, 152, 136, 72, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 184, 120];
static OFF_63: [i8; 40] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_63: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_64: &str = " -ABCDEFGHLMPQSTVWabcdefghiklmnoprstuvwy";
static ADV_64: [u16; 40] = [64, 80, 168, 152, 168, 184, 144, 136, 184, 176, 120, 224, 144, 208, 136, 136, 160, 240, 136, 152, 120, 152, 136, 80, 152, 144, 64, 128, 64, 216, 136, 144, 152, 88, 104, 80, 136, 120, 192, 120];
static OFF_64: [i8; 40] = [0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static ASCII_64: [u8; 96] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 10, 11, 255, 255, 12, 13, 255, 14, 15, 255, 16, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 19, 20, 21, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 32, 255, 33, 34, 35, 36, 37, 38, 255, 39, 255, 255, 255, 255, 255, 255];
const CHARS_65: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_65: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_65: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_65: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_66: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_66: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_66: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_66: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_67: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_67: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_67: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_67: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];
const CHARS_68: &str = " %&'()+,-./0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz°·…₂";
static ADV_68: [u16; 78] = [64, 224, 176, 64, 80, 80, 160, 64, 80, 64, 96, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 72, 168, 152, 168, 184, 144, 136, 184, 176, 64, 120, 168, 120, 224, 184, 200, 144, 208, 152, 136, 136, 168, 160, 240, 160, 152, 144, 136, 152, 120, 152, 136, 80, 152, 144, 64, 64, 128, 64, 216, 136, 144, 152, 152, 88, 104, 80, 136, 120, 192, 128, 120, 112, 96, 64, 240, 96];
static OFF_68: [i8; 78] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0];
static ASCII_68: [u8; 96] = [0, 255, 255, 255, 255, 1, 2, 3, 4, 5, 255, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 255, 255, 255, 255, 255, 255, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 255, 255, 255, 255, 255, 255, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 255, 255, 255, 255, 255];

pub mod style {
    use super::*;

    pub static TITLE: GlyphStyle = GlyphStyle { first: 433, chars: CHARS_0, ascii: &ASCII_0, advance: &ADV_0, x_offset: &OFF_0, ascent: 19, height: 27 };
    pub static TOPBAR: GlyphStyle = GlyphStyle { first: 511, chars: CHARS_1, ascii: &ASCII_1, advance: &ADV_1, x_offset: &OFF_1, ascent: 15, height: 22 };
    pub static TOPBAR_WARN: GlyphStyle = GlyphStyle { first: 589, chars: CHARS_2, ascii: &ASCII_2, advance: &ADV_2, x_offset: &OFF_2, ascent: 15, height: 22 };
    pub static ALERT_TITLE_BAR: GlyphStyle = GlyphStyle { first: 667, chars: CHARS_3, ascii: &ASCII_3, advance: &ADV_3, x_offset: &OFF_3, ascent: 19, height: 27 };
    pub static ALERT_TOPBAR: GlyphStyle = GlyphStyle { first: 745, chars: CHARS_4, ascii: &ASCII_4, advance: &ADV_4, x_offset: &OFF_4, ascent: 15, height: 22 };
    pub static CLOCK: GlyphStyle = GlyphStyle { first: 823, chars: CHARS_5, ascii: &ASCII_5, advance: &ADV_5, x_offset: &OFF_5, ascent: 105, height: 145 };
    pub static AMPM: GlyphStyle = GlyphStyle { first: 841, chars: CHARS_6, ascii: &ASCII_6, advance: &ADV_6, x_offset: &OFF_6, ascent: 29, height: 41 };
    pub static DATE: GlyphStyle = GlyphStyle { first: 844, chars: CHARS_7, ascii: &ASCII_7, advance: &ADV_7, x_offset: &OFF_7, ascent: 21, height: 30 };
    pub static CARD_HEADER: GlyphStyle = GlyphStyle { first: 922, chars: CHARS_8, ascii: &ASCII_8, advance: &ADV_8, x_offset: &OFF_8, ascent: 16, height: 23 };
    pub static CARD_HEADER_WARN: GlyphStyle = GlyphStyle { first: 1000, chars: CHARS_9, ascii: &ASCII_9, advance: &ADV_9, x_offset: &OFF_9, ascent: 16, height: 23 };
    pub static CARD_AGE: GlyphStyle = GlyphStyle { first: 1078, chars: CHARS_10, ascii: &ASCII_10, advance: &ADV_10, x_offset: &OFF_10, ascent: 15, height: 22 };
    pub static STATEMENT: GlyphStyle = GlyphStyle { first: 1156, chars: CHARS_11, ascii: &ASCII_11, advance: &ADV_11, x_offset: &OFF_11, ascent: 22, height: 31 };
    pub static TOAST_TITLE: GlyphStyle = GlyphStyle { first: 1265, chars: CHARS_12, ascii: &ASCII_12, advance: &ADV_12, x_offset: &OFF_12, ascent: 21, height: 30 };
    pub static TOAST_SUB: GlyphStyle = GlyphStyle { first: 1374, chars: CHARS_13, ascii: &ASCII_13, advance: &ADV_13, x_offset: &OFF_13, ascent: 16, height: 23 };
    pub static ROW_VALUE: GlyphStyle = GlyphStyle { first: 1483, chars: CHARS_14, ascii: &ASCII_14, advance: &ADV_14, x_offset: &OFF_14, ascent: 27, height: 38 };
    pub static ROW_UNIT: GlyphStyle = GlyphStyle { first: 1501, chars: CHARS_15, ascii: &ASCII_15, advance: &ADV_15, x_offset: &OFF_15, ascent: 15, height: 22 };
    pub static ROOM_NOTE: GlyphStyle = GlyphStyle { first: 1518, chars: CHARS_16, ascii: &ASCII_16, advance: &ADV_16, x_offset: &OFF_16, ascent: 15, height: 22 };
    pub static ALERT_HEAD: GlyphStyle = GlyphStyle { first: 1627, chars: CHARS_17, ascii: &ASCII_17, advance: &ADV_17, x_offset: &OFF_17, ascent: 29, height: 41 };
    pub static ALERT_SUB: GlyphStyle = GlyphStyle { first: 1736, chars: CHARS_18, ascii: &ASCII_18, advance: &ADV_18, x_offset: &OFF_18, ascent: 17, height: 24 };
    pub static ALERT_VALUE: GlyphStyle = GlyphStyle { first: 1845, chars: CHARS_19, ascii: &ASCII_19, advance: &ADV_19, x_offset: &OFF_19, ascent: 113, height: 155 };
    pub static ALERT_UNIT: GlyphStyle = GlyphStyle { first: 1863, chars: CHARS_20, ascii: &ASCII_20, advance: &ADV_20, x_offset: &OFF_20, ascent: 23, height: 33 };
    pub static ALERT_METRIC: GlyphStyle = GlyphStyle { first: 1880, chars: CHARS_21, ascii: &ASCII_21, advance: &ADV_21, x_offset: &OFF_21, ascent: 23, height: 33 };
    pub static ALERT_MSG: GlyphStyle = GlyphStyle { first: 1958, chars: CHARS_22, ascii: &ASCII_22, advance: &ADV_22, x_offset: &OFF_22, ascent: 19, height: 27 };
    pub static NIGHT_CLOCK: GlyphStyle = GlyphStyle { first: 2067, chars: CHARS_23, ascii: &ASCII_23, advance: &ADV_23, x_offset: &OFF_23, ascent: 151, height: 207 };
    pub static NIGHT_LINE: GlyphStyle = GlyphStyle { first: 2085, chars: CHARS_24, ascii: &ASCII_24, advance: &ADV_24, x_offset: &OFF_24, ascent: 21, height: 30 };
    pub static AWAY_CLOCK: GlyphStyle = GlyphStyle { first: 2194, chars: CHARS_25, ascii: &ASCII_25, advance: &ADV_25, x_offset: &OFF_25, ascent: 85, height: 117 };
    pub static AWAY_AMPM: GlyphStyle = GlyphStyle { first: 2212, chars: CHARS_26, ascii: &ASCII_26, advance: &ADV_26, x_offset: &OFF_26, ascent: 23, height: 33 };
    pub static AWAY_LINE: GlyphStyle = GlyphStyle { first: 2215, chars: CHARS_27, ascii: &ASCII_27, advance: &ADV_27, x_offset: &OFF_27, ascent: 19, height: 27 };
    pub static BOOT_STEP: GlyphStyle = GlyphStyle { first: 2324, chars: CHARS_28, ascii: &ASCII_28, advance: &ADV_28, x_offset: &OFF_28, ascent: 18, height: 26 };
    pub static BOOT_STEP_DIM: GlyphStyle = GlyphStyle { first: 2364, chars: CHARS_29, ascii: &ASCII_29, advance: &ADV_29, x_offset: &OFF_29, ascent: 18, height: 26 };
    pub static BOOT_DETAIL: GlyphStyle = GlyphStyle { first: 2404, chars: CHARS_30, ascii: &ASCII_30, advance: &ADV_30, x_offset: &OFF_30, ascent: 16, height: 23 };
    pub static BOOT_VERSION: GlyphStyle = GlyphStyle { first: 2513, chars: CHARS_31, ascii: &ASCII_31, advance: &ADV_31, x_offset: &OFF_31, ascent: 14, height: 20 };
    pub static SETUP_VALUE: GlyphStyle = GlyphStyle { first: 2591, chars: CHARS_32, ascii: &ASCII_32, advance: &ADV_32, x_offset: &OFF_32, ascent: 29, height: 41 };
    pub static SETUP_WAIT: GlyphStyle = GlyphStyle { first: 2700, chars: CHARS_33, ascii: &ASCII_33, advance: &ADV_33, x_offset: &OFF_33, ascent: 17, height: 24 };
    pub static TILE_VALUE_GOOD: GlyphStyle = GlyphStyle { first: 2778, chars: CHARS_34, ascii: &ASCII_34, advance: &ADV_34, x_offset: &OFF_34, ascent: 41, height: 57 };
    pub static TILE_UNIT_GOOD: GlyphStyle = GlyphStyle { first: 2796, chars: CHARS_35, ascii: &ASCII_35, advance: &ADV_35, x_offset: &OFF_35, ascent: 17, height: 24 };
    pub static TILE_WORD_GOOD: GlyphStyle = GlyphStyle { first: 2813, chars: CHARS_36, ascii: &ASCII_36, advance: &ADV_36, x_offset: &OFF_36, ascent: 16, height: 23 };
    pub static ROW_WORD_GOOD: GlyphStyle = GlyphStyle { first: 2813, chars: CHARS_36, ascii: &ASCII_36, advance: &ADV_36, x_offset: &OFF_36, ascent: 16, height: 23 };
    pub static ROOM_BIG_GOOD: GlyphStyle = GlyphStyle { first: 2853, chars: CHARS_37, ascii: &ASCII_37, advance: &ADV_37, x_offset: &OFF_37, ascent: 65, height: 90 };
    pub static ROOM_SMALL_GOOD: GlyphStyle = GlyphStyle { first: 2872, chars: CHARS_38, ascii: &ASCII_38, advance: &ADV_38, x_offset: &OFF_38, ascent: 45, height: 63 };
    pub static ROOM_HUM_GOOD: GlyphStyle = GlyphStyle { first: 2891, chars: CHARS_39, ascii: &ASCII_39, advance: &ADV_39, x_offset: &OFF_39, ascent: 19, height: 27 };
    pub static ROOM_WORD_GOOD: GlyphStyle = GlyphStyle { first: 2813, chars: CHARS_36, ascii: &ASCII_36, advance: &ADV_36, x_offset: &OFF_36, ascent: 16, height: 23 };
    pub static TILE_VALUE_MODERATE: GlyphStyle = GlyphStyle { first: 2909, chars: CHARS_40, ascii: &ASCII_40, advance: &ADV_40, x_offset: &OFF_40, ascent: 41, height: 57 };
    pub static TILE_UNIT_MODERATE: GlyphStyle = GlyphStyle { first: 2927, chars: CHARS_41, ascii: &ASCII_41, advance: &ADV_41, x_offset: &OFF_41, ascent: 17, height: 24 };
    pub static TILE_WORD_MODERATE: GlyphStyle = GlyphStyle { first: 2944, chars: CHARS_42, ascii: &ASCII_42, advance: &ADV_42, x_offset: &OFF_42, ascent: 16, height: 23 };
    pub static ROW_WORD_MODERATE: GlyphStyle = GlyphStyle { first: 1000, chars: CHARS_9, ascii: &ASCII_9, advance: &ADV_9, x_offset: &OFF_9, ascent: 16, height: 23 };
    pub static ROOM_BIG_MODERATE: GlyphStyle = GlyphStyle { first: 2984, chars: CHARS_43, ascii: &ASCII_43, advance: &ADV_43, x_offset: &OFF_43, ascent: 65, height: 90 };
    pub static ROOM_SMALL_MODERATE: GlyphStyle = GlyphStyle { first: 3003, chars: CHARS_44, ascii: &ASCII_44, advance: &ADV_44, x_offset: &OFF_44, ascent: 45, height: 63 };
    pub static ROOM_HUM_MODERATE: GlyphStyle = GlyphStyle { first: 3022, chars: CHARS_45, ascii: &ASCII_45, advance: &ADV_45, x_offset: &OFF_45, ascent: 19, height: 27 };
    pub static ROOM_WORD_MODERATE: GlyphStyle = GlyphStyle { first: 3040, chars: CHARS_46, ascii: &ASCII_46, advance: &ADV_46, x_offset: &OFF_46, ascent: 16, height: 23 };
    pub static TILE_VALUE_POOR: GlyphStyle = GlyphStyle { first: 3080, chars: CHARS_47, ascii: &ASCII_47, advance: &ADV_47, x_offset: &OFF_47, ascent: 41, height: 57 };
    pub static TILE_UNIT_POOR: GlyphStyle = GlyphStyle { first: 3098, chars: CHARS_48, ascii: &ASCII_48, advance: &ADV_48, x_offset: &OFF_48, ascent: 17, height: 24 };
    pub static TILE_WORD_POOR: GlyphStyle = GlyphStyle { first: 3115, chars: CHARS_49, ascii: &ASCII_49, advance: &ADV_49, x_offset: &OFF_49, ascent: 16, height: 23 };
    pub static ROW_WORD_POOR: GlyphStyle = GlyphStyle { first: 3155, chars: CHARS_50, ascii: &ASCII_50, advance: &ADV_50, x_offset: &OFF_50, ascent: 16, height: 23 };
    pub static ROOM_BIG_POOR: GlyphStyle = GlyphStyle { first: 3195, chars: CHARS_51, ascii: &ASCII_51, advance: &ADV_51, x_offset: &OFF_51, ascent: 65, height: 90 };
    pub static ROOM_SMALL_POOR: GlyphStyle = GlyphStyle { first: 3214, chars: CHARS_52, ascii: &ASCII_52, advance: &ADV_52, x_offset: &OFF_52, ascent: 45, height: 63 };
    pub static ROOM_HUM_POOR: GlyphStyle = GlyphStyle { first: 3233, chars: CHARS_53, ascii: &ASCII_53, advance: &ADV_53, x_offset: &OFF_53, ascent: 19, height: 27 };
    pub static ROOM_WORD_POOR: GlyphStyle = GlyphStyle { first: 3251, chars: CHARS_54, ascii: &ASCII_54, advance: &ADV_54, x_offset: &OFF_54, ascent: 16, height: 23 };
    pub static TILE_VALUE_BAD: GlyphStyle = GlyphStyle { first: 3291, chars: CHARS_55, ascii: &ASCII_55, advance: &ADV_55, x_offset: &OFF_55, ascent: 41, height: 57 };
    pub static TILE_UNIT_BAD: GlyphStyle = GlyphStyle { first: 3309, chars: CHARS_56, ascii: &ASCII_56, advance: &ADV_56, x_offset: &OFF_56, ascent: 17, height: 24 };
    pub static TILE_WORD_BAD: GlyphStyle = GlyphStyle { first: 3326, chars: CHARS_57, ascii: &ASCII_57, advance: &ADV_57, x_offset: &OFF_57, ascent: 16, height: 23 };
    pub static ROW_WORD_BAD: GlyphStyle = GlyphStyle { first: 3366, chars: CHARS_58, ascii: &ASCII_58, advance: &ADV_58, x_offset: &OFF_58, ascent: 16, height: 23 };
    pub static ROOM_BIG_BAD: GlyphStyle = GlyphStyle { first: 3406, chars: CHARS_59, ascii: &ASCII_59, advance: &ADV_59, x_offset: &OFF_59, ascent: 65, height: 90 };
    pub static ROOM_SMALL_BAD: GlyphStyle = GlyphStyle { first: 3425, chars: CHARS_60, ascii: &ASCII_60, advance: &ADV_60, x_offset: &OFF_60, ascent: 45, height: 63 };
    pub static ROOM_HUM_BAD: GlyphStyle = GlyphStyle { first: 3444, chars: CHARS_61, ascii: &ASCII_61, advance: &ADV_61, x_offset: &OFF_61, ascent: 19, height: 27 };
    pub static ROOM_WORD_BAD: GlyphStyle = GlyphStyle { first: 3462, chars: CHARS_62, ascii: &ASCII_62, advance: &ADV_62, x_offset: &OFF_62, ascent: 16, height: 23 };
    pub static TILE_VALUE_NEUTRAL: GlyphStyle = GlyphStyle { first: 2778, chars: CHARS_34, ascii: &ASCII_34, advance: &ADV_34, x_offset: &OFF_34, ascent: 41, height: 57 };
    pub static TILE_UNIT_NEUTRAL: GlyphStyle = GlyphStyle { first: 2796, chars: CHARS_35, ascii: &ASCII_35, advance: &ADV_35, x_offset: &OFF_35, ascent: 17, height: 24 };
    pub static TILE_WORD_NEUTRAL: GlyphStyle = GlyphStyle { first: 3502, chars: CHARS_63, ascii: &ASCII_63, advance: &ADV_63, x_offset: &OFF_63, ascent: 16, height: 23 };
    pub static ROW_WORD_NEUTRAL: GlyphStyle = GlyphStyle { first: 3542, chars: CHARS_64, ascii: &ASCII_64, advance: &ADV_64, x_offset: &OFF_64, ascent: 16, height: 23 };
    pub static CHIP_CARD_ACT: GlyphStyle = GlyphStyle { first: 3582, chars: CHARS_65, ascii: &ASCII_65, advance: &ADV_65, x_offset: &OFF_65, ascent: 16, height: 23 };
    pub static CHIP_CARD_ACT_HI: GlyphStyle = GlyphStyle { first: 3660, chars: CHARS_66, ascii: &ASCII_66, advance: &ADV_66, x_offset: &OFF_66, ascent: 16, height: 23 };
    pub static CHIP_CARD_ASK: GlyphStyle = GlyphStyle { first: 3738, chars: CHARS_67, ascii: &ASCII_67, advance: &ADV_67, x_offset: &OFF_67, ascent: 16, height: 23 };
    pub static CHIP_CARD_ASK_HI: GlyphStyle = GlyphStyle { first: 3816, chars: CHARS_68, ascii: &ASCII_68, advance: &ADV_68, x_offset: &OFF_68, ascent: 16, height: 23 };
}


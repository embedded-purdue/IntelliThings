//! input vp map

pub const BASE: u16 = 0x1000;
pub const WORDS: usize = 0x800;
pub const VERSION: u16 = 0x0100;
/// missing sensor reading
pub const NO_READING: u16 = 0x8000;
pub const RESET_MAGIC: u16 = 0x5aa5;
/// rooms in order
pub const ROOM_NAMES: [&str; 5] = ["Living Room", "Kitchen", "Bedroom", "Office", "Entry"];

// sensor readings
pub const TEMPERATURE: u16 = 0x1000;
pub const HUMIDITY: u16 = 0x1001;
pub const VOC_INDEX: u16 = 0x1002;
pub const CO2: u16 = 0x1003;
pub const LUX: u16 = 0x1004;
pub const DISTANCE: u16 = 0x1005;
pub const PM1_0: u16 = 0x1006;
pub const PM2_5: u16 = 0x1007;
pub const PM10: u16 = 0x1008;
pub const PRESENCE: u16 = 0x1009;

// connection
pub const WIFI: u16 = 0x1010;
pub const HUB: u16 = 0x1011;
pub const SYNCED_AGO: u16 = 0x1012;
pub const RETRY_IN: u16 = 0x1013;

// clock
pub const UNIX_TIME: u16 = 0x1020;
pub const UTC_OFFSET: u16 = 0x1022;

// settings
pub const UNITS: u16 = 0x1030;
pub const CLOCK_24H: u16 = 0x1031;
pub const ROOM: u16 = 0x1032;
pub const NIGHT_START: u16 = 0x1033;
pub const NIGHT_END: u16 = 0x1034;
pub const AWAY_AFTER: u16 = 0x1035;
pub const ROTATE_PAGES: u16 = 0x1036;
pub const SCREEN: u16 = 0x1037;
pub const MODE: u16 = 0x1038;
pub const LED_BRIGHTNESS: u16 = 0x1039;
pub const INPUTS_RESET: u16 = 0x103f;
pub const ROOM_NAME: u16 = 0x1040;
pub const ROOM_NAME_WORDS: usize = 12;

// display status
pub const INTERFACE_VERSION: u16 = 0x10f0;
pub const PANEL_STATE: u16 = 0x10f1;
pub const SCREEN_SHOWN: u16 = 0x10f2;
pub const MODE_NOW: u16 = 0x10f3;
pub const WRITES_OK: u16 = 0x10f4;
pub const WRITES_REJECTED: u16 = 0x10f5;

// ai message
pub const MESSAGE_TS: u16 = 0x10fe;
pub const MESSAGE_TEXT: u16 = 0x1100;
pub const MESSAGE_TEXT_WORDS: usize = 256;
pub const THINKING: u16 = 0x1200;
pub const THINKING_WORDS: usize = 32;
pub const CHIP1: u16 = 0x1220; // chip 1
pub const CHIP2: u16 = 0x1240; // chip 2
pub const MESSAGE_CHIPS: [u16; 2] = [CHIP1, CHIP2];
/// entry field offsets
pub mod chip {
    pub const DEVICE: u16 = 0x0;
    pub const LABEL: u16 = 0x1;
    pub const LABEL_WORDS: usize = 14;
    pub const VALUE: u16 = 0x10;
    pub const VALUE_WORDS: usize = 6;
}

// toast
pub const TOAST_TRIGGER: u16 = 0x1300;
pub const TOAST_DEVICE: u16 = 0x1301;
pub const TOAST_TITLE: u16 = 0x1302;
pub const TOAST_TITLE_WORDS: usize = 20;
pub const TOAST_DETAIL: u16 = 0x1316;
pub const TOAST_DETAIL_WORDS: usize = 30;

// alert
pub const ALERT_ACTIVE: u16 = 0x1400;
pub const ALERT_METRIC: u16 = 0x1401;
pub const ALERT_VALUE: u16 = 0x1402;
pub const ALERT_SINCE: u16 = 0x1403;
pub const ALERT_TITLE: u16 = 0x1404;
pub const ALERT_TITLE_WORDS: usize = 20;
pub const ALERT_PLACE: u16 = 0x1418;
pub const ALERT_PLACE_WORDS: usize = 12;
pub const ALERT_NORMAL: u16 = 0x1424;
pub const ALERT_NORMAL_WORDS: usize = 14;
pub const ALERT_MESSAGE: u16 = 0x1432;
pub const ALERT_MESSAGE_WORDS: usize = 80;
pub const ALERT_DONE1: u16 = 0x1490; // done 1
pub const ALERT_DONE2: u16 = 0x14b0; // done 2
pub const ALERT_ASK: u16 = 0x14d0; // user request
pub const ALERT_CHIPS: [u16; 3] = [ALERT_DONE1, ALERT_DONE2, ALERT_ASK];

// rooms
pub const ROOM1: u16 = 0x1500; // living room
pub const ROOM2: u16 = 0x1510; // kitchen
pub const ROOM3: u16 = 0x1520; // bedroom
pub const ROOM4: u16 = 0x1530; // office
pub const ROOM5: u16 = 0x1540; // entry
pub const ROOMS: [u16; 5] = [ROOM1, ROOM2, ROOM3, ROOM4, ROOM5];
/// entry field offsets
pub mod room {
    pub const ONLINE: u16 = 0x0;
    pub const TEMPERATURE: u16 = 0x1;
    pub const HUMIDITY: u16 = 0x2;
    pub const CO2: u16 = 0x3;
    pub const PM2_5: u16 = 0x4;
    pub const VOC_INDEX: u16 = 0x5;
    pub const PRESENCE: u16 = 0x6;
    pub const NOTE: u16 = 0x7;
    pub const NOTE_WORDS: usize = 8;
}

// boot checklist
pub const BOOT0: u16 = 0x1600; // display
pub const BOOT1: u16 = 0x1620; // sensors
pub const BOOT2: u16 = 0x1640; // wi-fi
pub const BOOT3: u16 = 0x1660; // mqtt broker
pub const BOOT4: u16 = 0x1680; // home assistant
pub const BOOT_STEPS: [u16; 5] = [BOOT0, BOOT1, BOOT2, BOOT3, BOOT4];
/// entry field offsets
pub mod step {
    pub const STATE: u16 = 0x0;
    pub const DETAIL: u16 = 0x1;
    pub const DETAIL_WORDS: usize = 16;
}
pub const BOOT_VERSION: u16 = 0x16a0;
pub const BOOT_VERSION_WORDS: usize = 20;

// wi-fi setup
pub const SETUP_SSID: u16 = 0x1700;
pub const SETUP_SSID_WORDS: usize = 16;
pub const SETUP_PASSWORD: u16 = 0x1710;
pub const SETUP_PASSWORD_WORDS: usize = 16;
pub const SETUP_URL: u16 = 0x1720;
pub const SETUP_URL_WORDS: usize = 16;

/// power-up non-zero words
pub const DEFAULTS: &[(u16, u16)] = &[
    (0x1000, 0x8000),
    (0x1001, 0x8000),
    (0x1002, 0x8000),
    (0x1003, 0x8000),
    (0x1004, 0x8000),
    (0x1005, 0x8000),
    (0x1006, 0x8000),
    (0x1007, 0x8000),
    (0x1008, 0x8000),
    (0x1009, 0x8000),
    (0x1012, 0xffff),
    (0x1033, 0x0546),
    (0x1034, 0x0186),
    (0x1035, 0x0258),
    (0x1039, 0x0064),
    (0x10f0, 0x0100),
    (0x1220, 0xffff),
    (0x1240, 0xffff),
    (0x1490, 0xffff),
    (0x14b0, 0xffff),
    (0x14d0, 0xffff),
    (0x1500, 0xffff),
    (0x1501, 0x8000),
    (0x1502, 0x8000),
    (0x1503, 0x8000),
    (0x1504, 0x8000),
    (0x1505, 0x8000),
    (0x1510, 0xffff),
    (0x1511, 0x8000),
    (0x1512, 0x8000),
    (0x1513, 0x8000),
    (0x1514, 0x8000),
    (0x1515, 0x8000),
    (0x1520, 0xffff),
    (0x1521, 0x8000),
    (0x1522, 0x8000),
    (0x1523, 0x8000),
    (0x1524, 0x8000),
    (0x1525, 0x8000),
    (0x1530, 0xffff),
    (0x1531, 0x8000),
    (0x1532, 0x8000),
    (0x1533, 0x8000),
    (0x1534, 0x8000),
    (0x1535, 0x8000),
    (0x1540, 0xffff),
    (0x1541, 0x8000),
    (0x1542, 0x8000),
    (0x1543, 0x8000),
    (0x1544, 0x8000),
    (0x1545, 0x8000),
    (0x1600, 0xffff),
    (0x1620, 0xffff),
    (0x1640, 0xffff),
    (0x1660, 0xffff),
    (0x1680, 0xffff),
];

/// display-only words
pub const READ_ONLY: &[(u16, usize)] = &[
    (0x10f0, 1),
    (0x10f1, 1),
    (0x10f2, 1),
    (0x10f3, 1),
    (0x10f4, 1),
    (0x10f5, 1),
    (0x1600, 1),
    (0x1601, 16),
];

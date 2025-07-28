const DEVICE_TYPE_CODE: u8 = 0b10100000;

const DEVICE_ADDRESS_CODE: u8 = 0b000000;   // 3 bits for device address | default A0 = 0 A1 = 0 A2 = 0 

const WRITE_CODE: u8 = 0b00000000; // 0 for write
const READ_CODE: u8 = 0b00000001; // 1 for read

const DEVICE_ADDRESS_WRITE: u8 = DEVICE_TYPE_CODE | DEVICE_ADDRESS_CODE | WRITE_CODE;  // I2C address write for FRAM
const DEVICE_ADDRESS_READ: u8 = DEVICE_TYPE_CODE | DEVICE_ADDRESS_CODE | READ_CODE;  // I2C address read for FRAM 
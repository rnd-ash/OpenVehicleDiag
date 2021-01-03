/**
 * Javascript implementation of the J2534 library
 */
const PROTOCOL = {
  "J1850VPW": 0x01,
  "J1850PWM": 0x02,
  "ISO9141": 0x03,
  "ISO14230": 0x04,
  "CAN": 0x05,
  "ISO15765": 0x06,
  "SCI_A_ENGINE": 0x07,
  "SCI_A_TRANS": 0x08,
  "SCI_B_ENGINE": 0x09,
  "SCI_B_TRANS": 0x0A
};
const IOCTL_ID = {
  "GET_CONFIG": 0x01,
  "SET_CONFIG": 0x02,
  "READ_VBATT": 0x03,
  "FIVE_BAUD_INIT": 0x04,
  "FAST_INIT": 0x06,
  "CLEAR_TX_BUFFER": 0x07,
  "CLEAR_RX_BUFFER": 0x08,
  "CLEAR_PERIODIC_MSGS": 0x09,
  "CLEAR_MSG_FILTERS": 0x0A,
  "CLEAR_FUNCT_MSG_LOOKUP_TABLE": 0x0B,
  "ADD_TO_FUNCT_MSG_LOOKUP_TABLE": 0x0C,
  "DELETE_FROM_FUNCT_MSG_LOOKUP_TABLE": 0x0D,
  "READ_PROG_VOLTAGE": 0x0E
};
const IOCTL_PARAM = {
  "IOCTL_ID": 0x01,
  "LOOPBACK": 0x03,
  "NODE_ADDRESS": 0x04,
  "NETWORK_LINE": 0x05,
  "P1_MIN": 0x06,
  "P1_MAX": 0x07,
  "P2_MIN": 0x08,
  "P2_MAX": 0x09,
  "P3_MIN": 0x0A,
  "P3_MAX": 0x0B,
  "P4_MIN": 0x0C,
  "P4_MAX": 0x0D,
  "W1": 0x0E,
  "W2": 0x0F,
  "W3": 0x10,
  "W4": 0x11,
  "W5": 0x12,
  "TIDLE": 0x13,
  "TINL": 0x14,
  "TWUP": 0x15,
  "PARITY": 0x16,
  "BIT_SAMPLE_POINT": 0x17,
  "SYNCH_JUMP_WIDTH": 0x18,
  "W0": 0x19,
  "T1_MAX": 0x1A,
  "T2_MAX": 0x1B,
  "T4_MAX": 0x1C,
  "T5_MAX": 0x1D,
  "ISO15765_BS": 0x1E,
  "ISO15765_STMIN": 0x1F,
  "DATA_BITS": 0x20,
  "FIVE_BAUD_INIT": 0x21,
  "BS_TX": 0x22,
  "STMIN_TX": 0x23,
  "T3_MAX": 0x24,
  "ISO157655_WFT_MAX": 0x25
};
const PASSTHRU_ERROR = {
  "STATUS_NOERROR": 0x00,
  "ERR_NOT_SUPPORTED": 0x01,
  "ERR_INVALID_CHANNEL_ID": 0x02,
  "ERR_INVALID_PROTOCOL_ID": 0x03,
  "ERR_NULL_PARAMETER": 0x04,
  "ERR_INVALID_IOCTL_VALUE": 0x05,
  "ERR_INVALID_FLAGS": 0x06,
  "ERR_FAILED": 0x07,
  "ERR_DEVICE_NOT_CONNECTED": 0x08,
  "ERR_TIMEOUT": 0x09,
  "ERR_INVALID_MSG": 0x0A,
  "ERR_INVALID_TIME_INTERVAL": 0x0B,
  "ERR_EXCEEDED_LIMIT": 0x0C,
  "ERR_INVALID_MSG_ID": 0x0D,
  "ERR_DEVICE_IN_USE": 0x0E,
  "ERR_INVALID_IOCTL_ID": 0x0F,
  "ERR_BUFFER_EMPTY": 0x10,
  "ERR_BUFFER_FULL": 0x11,
  "ERR_BUFFER_OVERFLOW": 0x12,
  "ERR_PIN_INVALID": 0x13,
  "ERR_CHANNEL_IN_USE": 0x14,
  "ERR_MSG_PROTOCOL_ID": 0x15,
  "ERR_INVALID_FILTER_ID": 0x16,
  "ERR_NO_FLOW_CONTROL": 0x17,
  "ERR_NOT_UNIQUE": 0x18,
  "ERR_INVALID_BAUDRATE": 0x19,
  "ERR_INVALID_DEVICE_ID": 0x1A
};
const FILTER_TYPE = {
  "PASS_FILTER": 0x01,
  "BLOCK_FILTER": 0x02,
  "FLOW_CONTROL_FILTER": 0x03
};
const LOOPBACK_SETTING = {
  "OFF": 0x00,
  "ON": 0x01
};
const DATA_BITS = {
  "DATA_BITS_8": 0x00,
  "DATA_BITS_7": 0x01
};
const PARITY_SETTING = {
  "NO_PARITY": 0x00,
  "ODD_PARITY": 0x01,
  "EVEN_PARITY": 0x02
};
const J1850PWM_NETWORK_LINE = {
  "BUS_NORMAL": 0x00,
  "BUS_PLUG": 0x01,
  "BUS_MINUS": 0x02
};
const CONNECT_FLAGS = {
  "CAN_29BIT_ID": 0x00000100,
  "ISO1941_NO_CHECKSUM": 0x00000200,
  "CAN_ID_BOTH": 0x00000800,
  "ISO9141_K_LINE_ONLY": 0x00001000
};
const RX_FLAG = {
  "CAN_29BIT_ID": 0x00000100,
  "ISO15765_ADDR_TYPE": 0x00000080,
  "ISO15765_PADDING_ERROR": 0x00000010,
  "TX_DONE": 0x0000008,
  "RX_BREAK": 0x00000004,
  "ISO15765_FIRST_FRAME": 0x00000002,
  "START_OF_MESSAGE": 0x00000002,
  "TX_MSG_TYPE": 0x00000001
};
const TX_FLAG = {
  "SCI_TX_VOLTAGE": 0x00800000,
  "SCI_MODE": 0x00400000,
  "BLOCKING": 0x00010000,
  "WAIT_P3_MIN_ONLY": 0x00000200,
  "CAN_29BIT_ID": 0x00000100,
  "CAN_EXTENDED_ID": 0x00000100,
  "ISO15765_ADDR_TYPE": 0x00000080,
  "ISO15765_EXT_ADDR": 0x00000080,
  "ISO15765_FRAME_PAD": 0x00000040,
  "TX_NORMAL_TRANSIT": 0x00000000
};

class PASSTHRU_MSG {
  constructor(protocol, data) {
    this.protocol = void 0;
    this.rx_status = void 0;
    this.tx_flags = void 0;
    this.timestamp = void 0;
    this.data_size = void 0;
    this.extra_data_size = void 0;
    this.data = void 0;
    this.protocol = protocol;
    this.data = data;
    this.data_size = data.length;
    this.timestamp = 0;
    this.rx_status = 0;
    this.tx_flags = 0;
    this.extra_data_size = 0;
  }

  static from_raw(raw) {}

  to_raw() {
    return Uint8Array.of(0);
  }

  set_protocol(protocol) {
    this.protocol = protocol;
  }
  /**
   *
   * @param {Array} flags - Array of Tx flags
   */


  set_tx_flags(flags) {
    this.tx_flags = 0;

    for (let i = 0; i < flags.length; i++) {
      this.tx_flags = this.tx_flags | flags[i].valueOf();
    }
  }

  set_rx_status(flags) {
    this.rx_status = 0;

    for (let i = 0; i < flags.length; i++) {
      this.rx_status = this.rx_status | flags[i].valueOf();
    }
  }

  set_extra_data_size(size) {
    this.data_size = size;
  }

  set_data(data) {
    this.data = data;
    this.data_size = data.length;
  }

  set_timestamp(time) {
    this.timestamp = time.valueOf();
  }

}

module.exports = {
  PROTOCOL,
  IOCTL_ID,
  IOCTL_PARAM,
  PASSTHRU_ERROR,
  FILTER_TYPE,
  LOOPBACK_SETTING,
  DATA_BITS,
  PARITY_SETTING,
  J1850PWM_NETWORK_LINE,
  CONNECT_FLAGS,
  RX_FLAG,
  TX_FLAG,
  PASSTHRU_MSG
};
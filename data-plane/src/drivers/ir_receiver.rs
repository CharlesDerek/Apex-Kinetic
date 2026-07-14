use log::info;

pub const RECV_PIN: u8 = 9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrCommand {
    Up,
    Down,
    Left,
    Right,
    Ok,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
}

#[derive(Clone, Debug, Default)]
pub struct IrReceiverDriver {
    pub last_received_ms: Option<u64>,
}

impl IrReceiverDriver {
    pub fn new() -> Self {
        info!("Initializing NEC infrared receiver on pin {}", RECV_PIN);
        Self::default()
    }

    pub fn decode(&mut self, value: u32, now_ms: u64) -> Option<IrCommand> {
        let command = decode_nec_value(value)?;
        self.last_received_ms = Some(now_ms);
        Some(command)
    }
}

pub fn decode_nec_value(value: u32) -> Option<IrCommand> {
    match value {
        16_736_925 | 5_316_027 => Some(IrCommand::Up),
        16_754_775 | 2_747_854_299 => Some(IrCommand::Down),
        16_720_605 | 1_386_468_383 => Some(IrCommand::Left),
        16_761_405 | 553_536_955 => Some(IrCommand::Right),
        16_712_445 | 3_622_325_019 => Some(IrCommand::Ok),
        16_738_455 | 3_238_126_971 => Some(IrCommand::Num1),
        16_750_695 | 2_538_093_563 => Some(IrCommand::Num2),
        16_756_815 | 4_039_382_595 => Some(IrCommand::Num3),
        16_724_175 | 2_534_850_111 => Some(IrCommand::Num4),
        16_718_055 | 1_033_561_079 => Some(IrCommand::Num5),
        16_743_045 | 1_635_910_171 => Some(IrCommand::Num6),
        16_716_015 | 2_351_064_443 => Some(IrCommand::Num7),
        16_726_215 | 1_217_346_747 => Some(IrCommand::Num8),
        16_734_885 | 71_952_287 => Some(IrCommand::Num9),
        _ => None,
    }
}

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use crate::{
    pins::OutputArray,
    virtual_display::{Notifier, VirtualDisplay, CELL_COUNT0, SEGMENT_COUNT0},
};

const BLINK_OFF_DELAY: Duration = Duration::from_millis(50); // const cmk
const BLINK_ON_DELAY: Duration = Duration::from_millis(150); // const cmk

pub struct BlinkableDisplay(
    &'static Signal<CriticalSectionRawMutex, (BlinkMode, [char; CELL_COUNT0])>,
);
pub type BlinkableNotifier = (
    Signal<CriticalSectionRawMutex, (BlinkMode, [char; CELL_COUNT0])>,
    Notifier<CELL_COUNT0>,
);

impl BlinkableDisplay {
    pub fn new(
        digit_pins: OutputArray<CELL_COUNT0>,
        segment_pins: OutputArray<SEGMENT_COUNT0>,
        notifier: &'static BlinkableNotifier,
        spawner: Spawner,
    ) -> Self {
        let virtual_display = VirtualDisplay::new(digit_pins, segment_pins, &notifier.1, spawner);

        let blinkable_display = Self(&notifier.0);
        unwrap!(spawner.spawn(blinkable_display_task(virtual_display, &notifier.0)));
        blinkable_display
    }

    pub const fn new_notifier() -> BlinkableNotifier {
        (Signal::new(), VirtualDisplay::new_notifier())
    }
}

#[embassy_executor::task]
#[allow(clippy::needless_range_loop)]
async fn blinkable_display_task(
    virtual_display: VirtualDisplay<CELL_COUNT0>,
    notifier: &'static Signal<CriticalSectionRawMutex, (BlinkMode, [char; CELL_COUNT0])>,
) -> ! {
    let mut blink_mode = BlinkMode::Solid;
    let mut chars = [' '; CELL_COUNT0];
    loop {
        (blink_mode, chars) = match blink_mode {
            BlinkMode::Solid => {
                virtual_display.write_chars(chars);
                notifier.wait().await
            }
            BlinkMode::BlinkingAndOn => {
                virtual_display.write_chars(chars);
                if let Either::First((new_blink_mode, new_chars)) =
                    select(notifier.wait(), Timer::after(BLINK_ON_DELAY)).await
                {
                    (new_blink_mode, new_chars)
                } else {
                    (BlinkMode::BlinkingButOff, chars)
                }
            }
            BlinkMode::BlinkingButOff => {
                virtual_display.write_chars([' '; CELL_COUNT0]);
                if let Either::First((new_blink_mode, new_chars)) =
                    select(notifier.wait(), Timer::after(BLINK_OFF_DELAY)).await
                {
                    (new_blink_mode, new_chars)
                } else {
                    (BlinkMode::BlinkingAndOn, chars)
                }
            }
        };
    }
}

impl BlinkableDisplay {
    pub fn write_chars(&self, chars: [char; CELL_COUNT0], blink_mode: BlinkMode) {
        info!("write_chars: {:?}, blink_mode: {:?}", chars, blink_mode);
        self.0.signal((blink_mode, chars));
    }
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum BlinkMode {
    Solid,
    BlinkingAndOn,
    BlinkingButOff,
}

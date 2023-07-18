#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use microbit::{
    board::Board,
    display::nonblocking::{Display, Frame, MicrobitFrame},
    hal::{
        clocks::Clocks,
        rtc::{Rtc, RtcInterrupt},
    },
    pac,
};
use microbit_text::{scrolling::Animate, scrolling_text::ScrollingStaticText};
use panic_halt as _;
use rtic::app;

#[app(device = microbit::pac, peripherals = true)]
mod app {
    use defmt::println;

    use super::*;

    const MESSAGE: &[u8] = b"Hello, world!";

    #[shared]
    struct Shared {
        display: Display<pac::TIMER1>,
    }

    #[local]
    struct Local {
        anim_timer: Rtc<pac::RTC0>,
        scroller: ScrollingStaticText,
        frame: MicrobitFrame,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        println!("start init");
        let board = Board::new(cx.device, cx.core);

        // Starting the low-frequency clock (needed for RTC to work)
        Clocks::new(board.CLOCK).start_lfclk();

        // RTC at 16Hz (32_768 / (2047 + 1))
        // 16Hz; 62.5ms period
        let mut rtc0 = Rtc::new(board.RTC0, 2047 * 2).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        let display = Display::new(board.TIMER1, board.display_pins);

        let mut scroller = ScrollingStaticText::default();
        scroller.set_message(MESSAGE);

        let frame = MicrobitFrame::default();
        println!("Done init");
        (
            Shared { display },
            Local {
                anim_timer: rtc0,
                scroller,
                frame,
            },
        )
    }

    #[task(binds = TIMER1, priority = 2, shared = [display])]
    fn timer1(mut cx: timer1::Context) {
        println!("timer1 irq");
        cx.shared
            .display
            .lock(|display| display.handle_display_event());
    }

    #[task(binds = RTC0, priority = 1, shared = [display],
           local = [anim_timer, scroller, frame])]
    fn rtc0(cx: rtc0::Context) {
        println!("rtc irq");
        let mut shared = cx.shared;
        let local = cx.local;
        local.anim_timer.reset_event(RtcInterrupt::Tick);
        if !local.scroller.is_finished() {
            local.scroller.tick();
            local.frame.set(local.scroller);
            shared.display.lock(|display| {
                display.show_frame(local.frame);
            });
        } else {
            local.scroller.reset();
        }
    }
}

#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;

use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

use embedded_hal::storage::ReadWrite;
use is25xp::IS25xP;
use stm32l4xx_hal::{
    gpio::{
        gpioc::{PC7, PC9},
        gpioe::{PE10, PE11, PE12, PE13, PE14, PE15},
        Alternate, Floating, Input, Output, PushPull, AF10,
    },
    prelude::*,
    qspi::{Qspi, QspiConfig},
    rcc::{ClockSecuritySystem, CrystalBypass, MsiFreq, PllConfig, PllDivider, PllSource},
};

type Clk = PE10<Alternate<AF10, Input<Floating>>>;
type Ncs = PE11<Alternate<AF10, Input<Floating>>>;
type QSpiIO0 = PE12<Alternate<AF10, Input<Floating>>>;
type QSpiIO1 = PE13<Alternate<AF10, Input<Floating>>>;
type QSpiIO2 = PE14<Alternate<AF10, Input<Floating>>>;
type QSpiIO3 = PE15<Alternate<AF10, Input<Floating>>>;

#[app(device = stm32l4xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        green: PC7<Output<PushPull>>,
        red: PC9<Output<PushPull>>,
        ext_flash: IS25xP<Qspi<(Clk, Ncs, QSpiIO0, QSpiIO1, QSpiIO2, QSpiIO3)>>,
    }

    #[init(spawn = [flash_test])]
    fn init(mut ctx: init::Context) -> init::LateResources {
        // Enable the DWT monotonic cycle counter for RTIC scheduling
        ctx.core.DCB.enable_trace();
        DWT::unlock();
        ctx.core.DWT.enable_cycle_counter();

        rtt_init_print!();

        rprintln!("[Init] Begin..");

        let mut flash = ctx.device.FLASH.constrain();
        let mut rcc = ctx.device.RCC.constrain();
        let mut pwr = ctx.device.PWR.constrain(&mut rcc.apb1r1);
        let mut gpioc = ctx.device.GPIOC.split(&mut rcc.ahb2);
        let mut gpioe = ctx.device.GPIOE.split(&mut rcc.ahb2);

        let _ = rcc
            .cfgr
            .lse(CrystalBypass::Disable, ClockSecuritySystem::Disable)
            .hse(
                8.mhz(),
                CrystalBypass::Disable,
                ClockSecuritySystem::Disable,
            )
            .sysclk_with_pll(80.mhz(), PllConfig::new(1, 20, PllDivider::Div2))
            .pll_source(PllSource::HSE)
            .msi(MsiFreq::RANGE48M)
            .hclk(80.mhz())
            .pclk1(80.mhz())
            .pclk2(80.mhz())
            .freeze(&mut flash.acr, &mut pwr);

        let mut green = gpioc
            .pc7
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        green.set_high().ok();
        let mut red = gpioc
            .pc9
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        red.set_high().ok();

        let ext_flash = {
            let clk = gpioe.pe10.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let ncs = gpioe.pe11.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let io_0 = gpioe.pe12.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let io_1 = gpioe.pe13.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let io_2 = gpioe.pe14.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let io_3 = gpioe.pe15.into_af10(&mut gpioe.moder, &mut gpioe.afrh);
            let qspi = Qspi::new(
                ctx.device.QUADSPI,
                (clk, ncs, io_0, io_1, io_2, io_3),
                &mut rcc.ahb3,
                QspiConfig::default().clock_prescaler(201),
            );
            is25xp::IS25xP::try_new(qspi).expect("Failed to initaite external flash driver")
        };

        rprintln!("[Init] Success!");


        ctx.spawn.flash_test().unwrap();

        init::LateResources {
            green,
            red,
            ext_flash,
        }
    }

    /// Idle thread - Captures the time the cpu is asleep to calculate cpu uasge
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            //wfi(); /* CPU is idle here waiting for interrupt */
        }
    }

    #[task(resources = [green, red, ext_flash])]
    fn flash_test(ctx: flash_test::Context) {
        let flash = ctx.resources.ext_flash;
        let green = ctx.resources.green;
        let red = ctx.resources.red;

        let mut write_data = [0u8; 1024];
        let mut read_data = [0u8; 1024];
        write_data
            .iter_mut()
            .skip(15)
            .enumerate()
            .for_each(|(i, x)| *x = i.wrapping_sub(usize::MAX) as u8);

        // Mass erase entire chip
        if let Err(e) = nb::block!(flash.try_erase()) {
            rprintln!("[Erase] Failure {:?}!", e);
            red.set_low().ok();
            panic!();
        }

        rprintln!("[Erase] Success!");

        let (start, end) = flash.range();
        rprintln!("[Range] (Start, End): ({:?}, {:?})", start, end);

        // Write data
        if let Err(e) = nb::block!(flash.try_write(start, &write_data)) {
            rprintln!("[Write] Failure {:?}!", e);
            red.set_low().ok();
            panic!();
        }
        rprintln!("[Write] Success!");

        // Read data
        if let Err(e) = nb::block!(flash.try_read(start, &mut read_data)) {
            rprintln!("[Read] Failure {:?}!", e);
            red.set_low().ok();
            panic!();
        }

        rprintln!("[Read] Success!");

        // Compare write data with read data
        write_data.iter().zip(read_data.iter()).for_each(|(w, r)| {
            if w != r {
                rprintln!("[Compare] Failure! W: {:?} != R {:?}", w, r);
                red.set_low().ok();
                panic!();
            }
        });

        rprintln!("[Compare] SUCCESS!");

        green.set_low().ok();
    }

    // spare interrupt used for scheduling software tasks
    extern "C" {
        fn UART5();
        fn SPI1();
        fn LCD();
    }
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

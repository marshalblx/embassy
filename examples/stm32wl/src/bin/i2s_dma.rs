// This example is written for an STM32F411 chip communicating with an external
// PCM5102a DAC. Remap pins, change clock speeds, etc. as necessary for your own
// hardware.
//
// NOTE: This example outputs potentially loud audio. Please run responsibly.

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S, Mode};
// use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
// use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Bypass,
            prescaler: HsePrescaler::DIV1,
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL8,
            divp: None,
            divq: Some(PllQDiv::DIV2), // PLL1_Q clock (32 / 2 * 6 / 2), used for RNG
            divr: Some(PllRDiv::DIV2), // sysclk 48Mhz clock (32 / 2 * 6 / 2)
        });
    }

    let p = embassy_stm32::init(config);
    info!("initialized");

    embassy_stm32::pac::RCC.ccipr().modify(|w| {
        w.set_spi2sel(0x01u8); // SPI2 clock from PLL1_Q
    });

    // info!("CCIPR.spi2sel value: {:x}", embassy_stm32::pac::RCC.ccipr().read().spi2sel());

    // stereo wavetable generation
    // let mut wavetable = [0u16; 1200];
    // for (i, frame) in wavetable.chunks_mut(2).enumerate() {
    //     frame[0] = ((((i / 150) % 2) * 2048) as i16 - 1024) as u16; // 160 Hz square wave in left channel
    //     frame[1] = ((((i / 100) % 2) * 2048) as i16 - 1024) as u16; // 240 Hz square wave in right channel
    // }

    // i2s configuration
    let mut dma_buffer = [0u16; 2400];
    let mut recv_buffer = [0u16; 800];

    let mut i2s_config = Config::default();
    i2s_config.format = Format::Data16Channel16;
    i2s_config.master_clock = true;
    i2s_config.mode = Mode::Master;
    // i2s_config.standard = embassy_stm32::i2s::Standard::PcmShortSync;
    let mut i2s = I2S::new_rxonly(
        p.SPI2,
        p.PC1,
        p.PA9,
        p.PB10,
        p.PA3,
        p.DMA1_CH2,
        &mut dma_buffer,
        Hertz(48_000),
        i2s_config,
    );
    info!("starting i2s");
    i2s.start();

    info!("looping");
    let mut count = 0;

    loop {
        // i2s.write(&wavetable).await.ok();
        match i2s.read(&mut recv_buffer).await {
            // Ok(_) => info!("buffer read"),
            Ok(_) => {
                count += 1;
                if count == 200 {
                    info!("buffer read: {:X}", recv_buffer[0..10]);
                    count = 0;
                }
            },
            Err(x) => error!("buffer read error {}", x),
        }
    }

    /*
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(500_000); // 1 MHz SPI clock
    let mut spi = Spi::new(
        p.SPI2,
        p.PB10, // sck
        p.PC1, // miso
        p.PA5, // mosi
        p.DMA1_CH2,
        p.DMA1_CH3,
        spi_config,
    );

    info!("starting spi");

    loop {
        spi.write(&mut dma_buffer).await.unwrap();
        info!("spi write done");
        Timer::after_millis(1000).await;
    }
    */


}

/*
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = {
        use embassy_stm32::rcc::*;

        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL192,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV4),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        // reference your chip's manual for proper clock settings; this config
        // is recommended for a 32 bit frame at 48 kHz sample rate
        config.rcc.plli2s = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL384,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV5),
        });
        config.enable_debug_during_sleep = true;

        config
    };

    let p = embassy_stm32::init(config);

    // stereo wavetable generation
    let mut wavetable = [0u16; 1200];
    for (i, frame) in wavetable.chunks_mut(2).enumerate() {
        frame[0] = ((((i / 150) % 2) * 2048) as i16 - 1024) as u16; // 160 Hz square wave in left channel
        frame[1] = ((((i / 100) % 2) * 2048) as i16 - 1024) as u16; // 240 Hz square wave in right channel
    }

    // i2s configuration
    let mut dma_buffer = [0u16; 2400];

    let mut i2s_config = Config::default();
    i2s_config.format = Format::Data16Channel32;
    i2s_config.master_clock = false;
    let mut i2s = I2S::new_txonly_nomck(
        p.SPI3,
        p.PB5,  // sd
        p.PA15, // ws
        p.PB3,  // ck
        p.DMA1_CH7,
        &mut dma_buffer,
        Hertz(48_000),
        i2s_config,
    );
    i2s.start();

    loop {
        i2s.write(&wavetable).await.ok();
    }
}
*/

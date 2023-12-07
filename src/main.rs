#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f4xx_hal::{pac, prelude::*} ;
use stm32f4xx_hal::otg_fs::{USB, UsbBus} ;
use usb_device::prelude::* ;

mod uvc_class;

#[entry]
fn main() -> ! {
    rtt_init_print!() ;

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    let dp = pac::Peripherals::take().unwrap() ;

    let rcc = dp.RCC.constrain() ;

    // let clocks = rcc.cfgr.sysclk((168).MHz()).pclk1((8).MHz()).freeze() ;
    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(48.MHz()).pclk1(24.MHz()).require_pll48clk().freeze() ;

    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();
    led.set_high();

    let gpioa = dp.GPIOA.split() ;

    let usb = USB {
        usb_global: dp.OTG_FS_GLOBAL,
        usb_device: dp.OTG_FS_DEVICE,
        usb_pwrclk: dp.OTG_FS_PWRCLK,
        pin_dm: gpioa.pa11.into_alternate(),
        pin_dp: gpioa.pa12.into_alternate(),
        hclk: clocks.hclk(),
    } ;

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut uvc = uvc_class::UvcClass::new(&usb_bus) ;

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xC251, 0x1706))
    .manufacturer("CITIZEN")
    .product("STM32 VIDEO Streaming")
    .device_class(uvc_class::USB_VIDEO_CAP_CLASS)
    .device_sub_class(0x02)
    .device_protocol(0x01)
    .max_packet_size_0(32)
    .device_release(0x0002)
    .build();
    
    rprintln!("UVC STM32F407 Camera Starting ...") ;
    
    loop {
        if !usb_dev.poll(&mut [&mut uvc ]) {
            continue;
        }

        led.set_high();
    }
}

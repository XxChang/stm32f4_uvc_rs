use core::marker::PhantomData;
use usb_device::class_prelude::*;
use usb_device::bus::StringIndex ;

pub const USB_VIDEO_CAP_CLASS: u8 = 0xEF ;
const USB_VIDEO_CC_VIDEO: u8 = 0x0E ;
const USB_VIDEO_SC_VIDEOCONTROL: u8 = 0x01 ;
const USB_VIDEO_SC_VIDEOSTREAMING: u8 = 0x02 ;
const USB_VIDEO_SC_VIDEO_INTERFACE_COLLECTION: u8 = 0x03 ;
const CS_INTERFACE: u8 = 0x24;
const VC_HEADER: u8 = 0x01;
const VC_INPUT_TERMINAL: u8 = 0x02;
const VC_OUTPUT_TERMINAL: u8 = 0x02;
const VS_INPUT_HEADER: u8 = 0x01 ;
const VS_FORMAT_MJPEG: u8 = 0x06 ;
const VS_FRAME_UNCOMPRESSED: u8 = 0x05 ;
const CAM_FPS: u32 = 8 ;

macro_rules! uvc_vc_interface_header_desc_size {
    ($e:expr) => {
        12+$e
    };
}

macro_rules! uvc_camera_terminal_desc_size {
    ($e:expr) => {
        15+$e
    };
}

macro_rules! uvc_output_terminal_desc_size {
    ($e:expr) => {
        9+$e
    };
}

macro_rules! uvc_vs_interface_input_header_desc_size {
    ($a:expr,$b:expr) => {
        13+$a*$b
    };
}

pub struct UvcClass<B: UsbBus> {
    p: PhantomData<B>,
    vc_if: InterfaceNumber,
    vs_if: InterfaceNumber,
    config_str: StringIndex,
    // data_ep: EndpointIn<'a, B>,
}

impl<B: UsbBus> UvcClass<B> {
    pub fn new(alloc: &UsbBusAllocator<B>) -> UvcClass<B> {
        UvcClass { 
            p: PhantomData,
            vc_if: alloc.interface(),
            vs_if: alloc.interface(),
            config_str: alloc.string(),
            // data_ep: alloc.isochronous(IsochronousSynchronizationType::NoSynchronization, IsochronousUsageType::Data, 768+2, 0x01)
         }
    }
}

impl<B: UsbBus> UsbClass<B> for UvcClass<B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> usb_device::Result<()> {
        writer.iad(self.vc_if, 0x02, USB_VIDEO_CC_VIDEO, USB_VIDEO_SC_VIDEO_INTERFACE_COLLECTION, 0)?;
        
        writer.interface_alt(self.vc_if, 0x00, USB_VIDEO_CC_VIDEO, USB_VIDEO_SC_VIDEOCONTROL, 0, Some(self.config_str))?;

        let cs_vc_total_size: u16 = uvc_vc_interface_header_desc_size!(1) + uvc_camera_terminal_desc_size!(2) + uvc_output_terminal_desc_size!(0) ;
        let cs_vc_total_size_le = cs_vc_total_size.to_le_bytes() ;
        let clk_freq = 0x005b8d80u32.to_le_bytes() ; // 6.0MHZ

        /* Class-specific VC Interface Descriptor */
        writer.write(CS_INTERFACE,             
            &[
            VC_HEADER, // bDescriptorSubtype
            0x10,
            0x00, // bcdCDC (1.10)
            cs_vc_total_size_le[0],
            cs_vc_total_size_le[1],
            clk_freq[0],
            clk_freq[1],
            clk_freq[2],
            clk_freq[3],
            0x01,
            0x01
        ],)? ;

        /* Input Terminal Descriptor (Camera) */
        writer.write(CS_INTERFACE, 
            &[
            VC_INPUT_TERMINAL,
            0x01,                   // bTerminalID
            0x01,
            0x02,
            0x00,
            0x00,
            /* wObjectiveFocalLengthMin 0 */
            0x00,
            0x00,
            /* wObjectiveFocalLengthMax 0 */
            0x00,
            0x00,
            /* wOcularFocalLength       0 */
            0x00,
            0x00,
            0x02,
            0x00,
            0x00
        ],)?;
        
        /* Output Terminal Descriptor */
        writer.write(CS_INTERFACE, 
            &[
            VC_OUTPUT_TERMINAL,
            0x02,           // bTerminalID
            0x01,
            0x01,
            0x00,
            0x01,
            0x00
            ],
        )?;

        writer.interface(self.vs_if, USB_VIDEO_CC_VIDEO, USB_VIDEO_SC_VIDEOSTREAMING, 0)?;

        let vc_header_size: u16 = uvc_vs_interface_input_header_desc_size!(1, 1) + 27 + 30 + 6 ;
        let vc_header_size_le = vc_header_size.to_le_bytes() ;

        /* Class-specific VS Header Descriptor (Input) */
        writer.write(CS_INTERFACE, 
            &[
            VS_INPUT_HEADER,
            0x01,
            vc_header_size_le[0],
            vc_header_size_le[1],
            0x01u8 | 0x80u8,            // bEndPointAddress      0x83 EP 3 IN
            0x00,                       // bmInfo                0 no dynamic format change supported
            0x02,
            0x00,
            0x01,
            0x00,
            0x01,
            0x00,                       // bmaControls(0)       0 no VS specific controls
            ], 
        )?;

        /* Class-specific VS Format Descriptor  */
        writer.write(CS_INTERFACE, 
            &[
            VS_FORMAT_MJPEG,
            0x01,
            0x01,
            0x01,
            0x01,
            0x00,
            0x00,
            0x00,
            0x00,
            ],
        )?;

        let width = 320u16 ;
        let width_le = width.to_le_bytes() ;
        let height = 240u16 ;
        let height_le = height.to_le_bytes() ;

        // let min_bit_rate = (width*height) as u32 *16*CAM_FPS ;
        let min_bit_rate = 16u32 ;
        let min_bit_rate_le = min_bit_rate.to_le_bytes() ;
        let max_bit_rate_le = min_bit_rate.to_le_bytes() ;
        // let max_frame_size = (height*width) as u32 *3/2 ;
        let max_frame_size = 1u32 ;
        let max_frame_size_le = max_frame_size.to_le_bytes() ;
        let interval = 1u32 ;
        // let interval = 10000000u32/CAM_FPS ;
        let interval_le = interval.to_le_bytes() ;

        /* Class-specific VS Frame Descriptor */
        writer.write(CS_INTERFACE, 
            &[
            VS_FRAME_UNCOMPRESSED,
            0x01,
            0x02,
            width_le[0], width_le[1],
            height_le[0], height_le[1],
            min_bit_rate_le[0], min_bit_rate_le[1], min_bit_rate_le[2], min_bit_rate_le[3],
            max_bit_rate_le[0], max_bit_rate_le[1], max_bit_rate_le[2], max_bit_rate_le[3],
            max_frame_size_le[0], max_frame_size_le[1], max_frame_size_le[2], max_frame_size_le[3],
            interval_le[0], interval_le[1], interval_le[2], interval_le[3],
            0x00,
            interval_le[0], interval_le[1], interval_le[2], interval_le[3],
            interval_le[0], interval_le[1], interval_le[2], interval_le[3],
            0x00, 0x00, 0x00, 0x00
            ],
        )?;

        /* Color Matching Descriptor */
        writer.write(CS_INTERFACE, 
            &[
            0x0D,
            0x01,
            0x01,
            0x04,
            ],
        )?;

        writer.interface_alt(self.vs_if, 0x01, USB_VIDEO_CC_VIDEO, USB_VIDEO_SC_VIDEOSTREAMING, 0x00, None)? ;

        // writer.endpoint(&self.data_ep).unwrap();
        
        Ok(())
    }

    fn get_string(&self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        let string_id = u8::from(index) ;
        match string_id {
            0x04 => Some("VIDEO Config"),
            0x05 => Some("VIDEO Interface"),
            _ => None,
        }
    }
}


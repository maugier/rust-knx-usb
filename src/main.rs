use std::fmt::Debug;

use libusb::{Context, Device};
use anyhow::{anyhow, Result};

trait KNXContext {
    fn find_device<'a>(&'a self) -> Result<Device<'a>>;
    fn knx(&self) -> Result<KNX>;
}

trait MyOption {
    fn none_or<E>(&self, err: E) -> Result<(), E>;
}

impl<T> MyOption for Option<T> {
    fn none_or<E>(&self, err: E) -> Result<(), E> {
        if self.is_none() { Ok(()) } else { Err(err) }
    }
}

impl KNXContext for Context {

    fn knx(&self) -> Result<KNX> {
        let dev = self.find_device()?;

        
        let config = dev.active_config_descriptor()?;
        let iface = config.interfaces().next().ok_or(anyhow!("device has no interface"))?;
        let idesc = iface.descriptors().next().ok_or(anyhow!("interface has no descriptors"))?;

        let mut input = None;
        let mut output = None;

        for ep in idesc.endpoint_descriptors() {
            if let libusb::TransferType::Interrupt = ep.transfer_type() {
                match ep.direction() {
                    libusb::Direction::In => input.replace(ep.address()).none_or(anyhow!("ambiguous IN endpoints"))?,
                    libusb::Direction::Out => output.replace(ep.address()).none_or(anyhow!("ambiguous OUT endpoints"))?,
                }
            }
        }

        let input = input.ok_or(anyhow!("no IN endpoint"))?;
        let output = output.ok_or(anyhow!("no OUT endpoint"))?;

        Ok(KNX { dev, input, output })

    }

    fn find_device<'a>(&'a self) -> Result<Device<'a>> {
        for dev in self.devices()?.iter() {
            let dev_desc = dev.device_descriptor()?;
            if dev_desc.vendor_id() == 0x135e {
                return Ok(dev)
            }
        }

        Err(anyhow!("No compatible device found"))
    }
}

struct KNX<'a> {
    dev: Device<'a>,
    input: u8,
    output: u8,
}

impl Debug for KNX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dev = format!("<device {}:{}>", self.dev.bus_number(), self.dev.address());
        f.debug_struct("KNX")
            .field("dev", &dev)
            .field("input", &self.input)
            .field("output", &self.output)
            .finish()
    }
}

fn main() -> Result<()> {

    let ctx = Context::new()?;

    if !ctx.has_hid_access() {
        eprintln!("WARNING: no hid access");
    }

    let knx = ctx.knx()?;


    
    Ok(())
}



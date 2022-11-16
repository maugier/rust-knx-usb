use libusb::{Context, Device};
use anyhow::{anyhow, Result};

trait KNXContext {
    fn find_device<'a>(&'a self) -> Result<Device<'a>>;
}


impl KNXContext for Context {
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

fn main() -> Result<()> {

    let ctx = Context::new()?;

    if !ctx.has_hid_access() {
        eprintln!("WARNING: no hid access");
    }
    
    let dev = ctx.find_device()?;
    let config = dev.active_config_descriptor()?;
    let iface = config.interfaces().next().ok_or(anyhow!("device has no interface"))?;
    let idesc = iface.descriptors().next().ok_or(anyhow!("interface has no descriptors"))?;

    eprintln!("Descriptor has {} endpoints", idesc.num_endpoints());

    for ep in idesc.endpoint_descriptors() {
        eprintln!("endpoint: {:?} {:?} {:?}", ep.direction(), ep.transfer_type(), ep);
    }

    Ok(())
}



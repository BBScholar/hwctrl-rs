#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct HardwareId(u32);

pub trait PeriodicUpdate {
    fn update(&mut self);
}

pub trait CanEnabled {
    fn can_callback(&mut self);

    fn can_id(&self) -> u32;
}

pub trait SpiEnabled {
    fn get_chip_select(&mut self) -> gpio::sysfs::SysFsGpioOutput;

    fn select_chip(&mut self) {
        use gpio::GpioOut;
        self.get_chip_select().set_low().unwrap();
    }

    fn deselect_chip(&mut self) {
        use gpio::GpioOut;
        self.get_chip_select().set_high().unwrap();
    }
}

pub trait HardwareBase {
    fn name(&self) -> &'static str;
    fn hardware_id(&self) -> HardwareId;
}

pub trait Motor {
    type Setpoint;
    fn init(&mut self);
    fn set(&mut self, setpoint: Self::Setpoint);
    fn stop(&mut self);
}

pub trait Calibrate {
    fn calibrate(&mut self);
}

pub trait Sensor {
    type Data;
    fn init(&mut self);
    fn get_value(&mut self) -> Self::Data;
    fn shutdown(&mut self);
}

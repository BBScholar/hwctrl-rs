#![allow(unused_imports, dead_code)]

mod hardware_traits;

use env_logger;

use socketcan;

use std::collections::HashMap;

use rosrust_msg::can_msgs;

const CAN_QUEUE_SIZE: usize = 256;

const MOTOR_NAMES: &'static [&'static str] = &[
    "port_drive",
    "starboard_drive",
    "dep",
    "exc_belt",
    "exc_translation",
    "exc_port_act",
    "exc_starboard_act",
];
const SENSOR_NAMES: &'static [&'static str] = &[
    "uwb_node_1",
    "uwb_node_2",
    "uwb_node_3",
    "uwb_node_4",
    "ebay_temperature",
    "quad_encoder_1",
    "imu",
    "adc_1",
    "adc_2",
    "limit_1",
    "limit_2",
    "limit_3",
    "limit_4",
    "power_sense",
    "estop",
];
const HARDWARE_BASE: &'static str = "/hardware";
const SPI_DEV_PATH: &'static str = "/dev/spidev1.0";
const CAL_FILE_DEFAULT: &'static str = "sensor_calibration.dat";
const CAN_IFACE: &'static str = "can1";

fn copy_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

// fn read_param_server<M, S>(
//     motor_map: &mut HashMap<hardware_traits::HardwareId, &dyn M>,
//     sensor_map: &mut HashMap<hardware_traits::HardwareId, S>,
// ) where
//     M: hardware_traits::Motor,
//     S: hardware_traits::Sensor,
// {
//     use std::string::String;
//
//     let base = HARDWARE_BASE.to_owned() + "/sensor";
//     for sensor_name in SENSOR_NAMES {
//         let full_name = base + "/" + *sensor_name;
//
//         let name_param = rosrust::param(&(full_name + "/name")).unwrap();
//
//         if name_param.exists().unwrap() {}
//     }
//
//     let base = HARDWARE_BASE.to_owned() + "/motor";
//     for motor_name in MOTOR_NAMES {
//         let full_name = base + "/" + *motor_name;
//     }
// }

fn main() {
    env_logger::init();

    let args: Vec<_> = rosrust::args();

    let calibrate = args[1].parse::<bool>().unwrap_or(false);

    rosrust::init("hwctrl");

    // let motor_map = HashMap::<hardware_traits::HardwareId, &dyn hardware_traits::Motor>::new();
    // let sensor_map = HashMap::new();

    // read_param_server(&mut motor_map, &mut sensor_map);

    let can_handle = std::thread::spawn(move || {
        let mut count: u64 = 0;
        let rate = rosrust::rate(1000.0);

        let can_sock = socketcan::CANSocket::open(CAN_IFACE).unwrap();
        can_sock.set_nonblocking(true);
        can_sock.filter_accept_all().unwrap();

        let can_rx = rosrust::publish("can_rx", CAN_QUEUE_SIZE).unwrap();

        let _can_tx = rosrust::subscribe("can_tx", CAN_QUEUE_SIZE, |frame: can_msgs::Frame| {
            can_sock.write_frame_insist(
                &socketcan::CANFrame::new(frame.id, &frame.data, frame.is_rtr, frame.is_error)
                    .unwrap(),
            );
        })
        .unwrap();

        while rosrust::is_ok() {
            while let Ok(raw_frame) = can_sock.read_frame() {
                let mut frame = can_msgs::Frame::default();
                let mut array = [0u8; 8];

                for idx in 0..raw_frame.data().len() {
                    array[idx] = raw_frame.data()[idx];
                }

                frame.id = raw_frame.id();
                frame.data = array;
                frame.dlc = raw_frame.data().len() as u8;
                frame.is_rtr = raw_frame.is_rtr();
                frame.is_error = raw_frame.is_error();
                can_rx.send(frame);
                count.wrapping_add(1);
            }
            rate.sleep();
        }
    });

    let motor_handle = std::thread::spawn(move || {
        let rate = rosrust::rate(1000.0);

        let vesc_rx_buffer = [0_u8, 1024];

        let limit_switch_fn = |state: bool| {};

        let can_rx_callback =
            rosrust::subscribe("can_rx", 256, |frame: can_msgs::Frame| {}).unwrap();

        let _estop_callback =
            rosrust::subscribe("estop", 256, |estop: rosrust_msg::std_msgs::Bool| {}).unwrap();
        let _setpoint_callback = rosrust::subscribe("motor_setpoints", 256, |setpoint| {}).unwrap();

        while rosrust::is_ok() {
            rate.sleep();
        }
    });

    let sensor_thread = std::thread::spawn(move || {
        if calibrate {
            // calibrate here
            rosrust::shutdown();
        }
    });

    rosrust::ros_info!("Spinning...");

    while rosrust::is_ok() {
        rosrust::spin();
    }

    can_handle.join().unwrap();
    motor_handle.join().unwrap();
    sensor_thread.join().unwrap();
}

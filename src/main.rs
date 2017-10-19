extern crate linkbot_core;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;

use std::f32::consts::PI;

use linkbot_core::{DaemonProxy, Robot, Goal};

extern {
    fn emscripten_exit_with_live_runtime();
}

#[no_mangle]
pub extern fn daemon_new() -> *mut DaemonProxy {
    //! Create and return a handle to a daemon proxy
    let d = DaemonProxy::new();
    Box::into_raw( Box::new(d) )
}

#[no_mangle]
pub extern fn daemon_set_write_callback(daemon: *mut DaemonProxy, 
                                        cb: extern fn(Vec<u8>)) {
    //! Set the function the daemon will use to send messages to the Daemon-Server.
    let mut d = unsafe {
        Box::from_raw(daemon)
    };

    d.set_write_callback( move |buf| {
        cb(buf);
    });

    Box::into_raw(d);
}

#[no_mangle]
pub extern fn daemon_deliver(daemon: *mut DaemonProxy, 
                             buffer: *mut u8, 
                             len: usize) {
    //! Pass messages coming from the Daemon-Server to this function
    let mut d = unsafe {
        Box::from_raw(daemon)
    };

    let vec = unsafe {
        Vec::from_raw_parts(buffer, len, len)
    };

    if let Err(msg) = d.deliver(&vec) {
        println!("Error delivering bytes to daemon proxy: {}", msg);
    }

    mem::forget(vec);
    Box::into_raw(d);
}

#[no_mangle]
pub extern fn daemon_get_robot(daemon: *mut DaemonProxy, 
                               serial_id: *mut c_char) -> *mut Robot {
    //! Get a handle to a robot object from the daemon proxy
    let mut d = unsafe {
        Box::from_raw(daemon)
    };

    let robot = unsafe {
        let cstring = CString::from_raw(serial_id);
        println!("Getting robot with id: '{}'", cstring.to_str().unwrap());
        let robot = d.get_robot(cstring.to_str().unwrap());
        println!("Getting robot with id: '{}' done", cstring.to_str().unwrap());
        mem::forget(cstring);
        robot
    };

    Box::into_raw(d);
    println!("Returning robot pointer...");
    Box::into_raw( Box::new(robot) )
}

#[no_mangle]
pub extern fn daemon_connect_robot(daemon: *mut DaemonProxy,
                                   robot: *mut Robot,
                                   serial_id: *mut c_char,
                                   cb: extern fn() ) {
    //! Make the daemon send a "connect robot" signal
    let mut d = unsafe {
        Box::from_raw(daemon)
    };
    let mut r = unsafe {
        Box::from_raw(robot)
    };

    r.set_connect_event_handler(move |_| {
        cb();
    });
    
    unsafe {
        let cstring = CString::from_raw(serial_id);
        d.connect_robot(cstring.to_str().unwrap()).unwrap();
        mem::forget(cstring);
    }

    Box::into_raw(d);
    Box::into_raw(r);
}

#[no_mangle]
pub extern fn daemon_stop_all(daemon: *mut DaemonProxy, cb: extern fn() ) {
    //! Stop all robots connected to this daemon proxy
    let mut d = unsafe {
        Box::from_raw(daemon)
    };

    d.stop_all_robots(move || { cb(); }).unwrap();
    
    Box::into_raw(d);
}

//
// Robot Interface
//

#[no_mangle]
pub extern fn robot_get_form_factor(robot: *mut Robot, cb: extern fn(u32)) 
{
    //! Get the robot's form factor.
    //! 0: Linkbot-I
    //! 1: Linkbot-L
    //! 2: Linkbot-T
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    r.get_form_factor(move |form| { 
        cb(form as u32); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_get_accelerometer(robot: *mut Robot,
                                      cb: extern fn(f32, f32, f32))
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    r.get_accelerometer_data(move |x, y, z| { 
        cb(x, y, z); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_get_joint_angles(robot: *mut Robot,
                                     cb: extern fn(f32, f32, f32, u32))
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    r.get_encoder_values(move |timestamp, angles| { 
        cb(angles[0].clone(), angles[1].clone(), angles[2].clone(), timestamp); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_reset_encoders(robot: *mut Robot, cb: extern fn())
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    r.reset_encoder_revs(move || { 
        cb(); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_buzzer_frequency(robot: *mut Robot,
                                         frequency: f32,
                                         cb: extern fn())
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    r.set_buzzer_frequency(frequency, move || { 
        cb(); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_led_color(robot: *mut Robot,
                                  red: u8,
                                  green: u8,
                                  blue: u8,
                                  cb: extern fn()) 
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
    
    println!("Setting led color to: {}, {}, {}", red, green, blue);
    r.set_led_color(red, green, blue, move || { 
        println!("Robot set_led_color() received reply!");
        cb(); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_motor_speeds(robot: *mut Robot,
                                     mask: u8,
                                     speed1: f32,
                                     speed2: f32,
                                     speed3: f32,
                                     cb: extern fn())
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };
  
    let values = vec![speed1, speed2, speed3].iter().map(|x| x*PI/180.0).collect();
    r.set_motor_controller_omega(mask as u32, values, 
                                 move|| {cb();} ).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_move(robot: *mut Robot,
                         mask: u8,
                         relative_mask: u8,
                         angle1: f32,
                         angle2: f32,
                         angle3: f32,
                         cb: extern fn())
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };

    let goals:Vec<Option<Goal>> = vec![angle1, angle2, angle3].iter().enumerate().map(|(i, angle)| {
        let a:f32 = angle * ::std::f32::consts::PI / 180.0;
        if mask & (1<<i) != 0 {
            let mut g = Goal::new();
            if relative_mask & (1<<i) != 0 {
                g.set_field_type(linkbot_core::Goal_Type::RELATIVE);
            } else {
                g.set_field_type(linkbot_core::Goal_Type::ABSOLUTE);
            }
            g.set_goal(a);
            g.set_controller(linkbot_core::Goal_Controller::CONSTVEL);
            Some(g)
        } else {
            None
        }
    }).collect();

    r.robot_move(goals[0].clone(), goals[1].clone(), goals[2].clone(), move || { 
        cb(); 
    }).unwrap();

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_accelerometer_event_handler(robot: *mut Robot,
                                             handler: Option<extern fn(f32, f32, f32, u32)>,
                                             completion_cb: extern fn()
                                             )
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };

    if let Some(cb) = handler {
        r.set_accelerometer_event_handler(move |timestamp, x, y, z| {
            cb(x, y, z, timestamp);
        });
        r.enable_accelerometer_event(true, None, move|| {
            completion_cb();
        }).unwrap();
    } else {
        r.enable_accelerometer_event(false, None, move|| {
            completion_cb();
        }).unwrap();
    }

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_button_event_handler(robot: *mut Robot,
                                             handler: Option<extern fn(u32, u32, u32)>,
                                             completion_cb: extern fn()
                                             )
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };

    if let Some(cb) = handler {
        r.set_button_event_handler(move |timestamp, button_no, state| {
            cb(button_no as u32, state as u32, timestamp);
        });
        r.enable_button_event(true, move|| {
            completion_cb();
        }).unwrap();
    } else {
        r.enable_button_event(false, move|| {
            completion_cb();
        }).unwrap();
    }

    Box::into_raw(r);
}

#[no_mangle]
pub extern fn robot_set_joint_event_handler(robot: *mut Robot, 
                                            handler: Option<extern fn(u32, u32, u32, f32)>,
                                            completion_cb: extern fn()
                                            )
{
    let mut r = unsafe{
        Box::from_raw(robot)
    };

    if let Some(cb) = handler {
        r.set_joint_event_handler(move |timestamp, joint, state, angle| {
            cb(timestamp, joint, state as u32, angle);
        });
        r.enable_joint_event(true, move|| {
            completion_cb();
        }).unwrap();
    } else {
        r.enable_joint_event(false, move|| {
            completion_cb();
        }).unwrap();
    }

    Box::into_raw(r);
}

fn main() {
    #[cfg(target_os="emscripten")]
    unsafe {
        emscripten_exit_with_live_runtime();
    }
}

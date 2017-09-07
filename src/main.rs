extern crate linkbot_core;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;

use linkbot_core::{DaemonProxy, Robot};

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

//
// Robot Interface
//

#[no_mangle]
pub extern fn robot_set_led_color(robot: &mut Robot,
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

fn main() {
    #[cfg(target_os="emscripten")]
    unsafe {
        emscripten_exit_with_live_runtime();
    }
}

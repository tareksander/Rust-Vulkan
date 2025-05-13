




use std::ffi::{c_char, CStr};

use ash::{self, vk::{self, InstanceCreateFlags, MAX_EXTENSION_NAME_SIZE}};
use winit::{dpi::PhysicalSize, event_loop::EventLoopBuilder, raw_window_handle::HasDisplayHandle, window::WindowBuilder};

fn main() {
    let l = EventLoopBuilder::new().build().unwrap();
    let w = WindowBuilder::new().with_inner_size(PhysicalSize::new(500.0, 500.0)).with_title("Gravity").build(&l).unwrap();
    
    
    let entry = unsafe { ash::Entry::load().unwrap() };
    let appinfo = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 3, 0));
    let layer_names = [c"VK_LAYER_KHRONOS_validation"];
    let layers_names_raw: Vec<*const c_char> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();
    let create_info = vk::InstanceCreateInfo::default().enabled_layer_names(&layers_names_raw).application_info(&appinfo)
    .enabled_extension_names(ash_window::enumerate_required_extensions(w.display_handle().unwrap().as_raw()).unwrap());
    let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };
    
    let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    for d in devices {
        let exts: Vec<[i8; MAX_EXTENSION_NAME_SIZE]> = unsafe { instance.enumerate_device_extension_properties(d).unwrap() }.into_iter().map(|e| e.extension_name).collect();
        
    }
    
    l.set_control_flow(winit::event_loop::ControlFlow::Wait);
    l.run(move |e, h| {
        match e {
            winit::event::Event::WindowEvent { window_id, event } => {
                match event {
                    winit::event::WindowEvent::Resized(physical_size) => {
                        
                    },
                    winit::event::WindowEvent::CloseRequested => h.exit(),
                    winit::event::WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                        
                    },
                    winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                        
                    },
                    winit::event::WindowEvent::CursorMoved { device_id, position } => {
                        
                    },
                    winit::event::WindowEvent::MouseWheel { device_id, delta, phase } => {
                        
                    },
                    winit::event::WindowEvent::MouseInput { device_id, state, button } => {
                        
                    },
                    winit::event::WindowEvent::Touch(touch) => {
                        
                    },
                    winit::event::WindowEvent::RedrawRequested => {
                        
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        
        
    }) .unwrap();
}

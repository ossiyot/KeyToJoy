extern crate winapi;

use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use winapi::{
    shared::minwindef::{HINSTANCE, LPVOID},
    um::winuser::{GetRawInputData, RegisterRawInputDevices, RAWINPUTHEADER},
};

use self::winapi::{
    shared::{
        hidusage::{HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_PAGE_GENERIC},
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::winuser::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
        TranslateMessage, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, HRAWINPUT, MSG,
        RAWINPUTDEVICE, RIDEV_INPUTSINK, RID_INPUT, WM_INPUT, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        WS_VISIBLE, RAWINPUT
    },
};

fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            let mut pcb_size = 0;
            GetRawInputData(
                l_param as HRAWINPUT,
                RID_INPUT,
                ptr::null_mut(),
                &mut pcb_size,
                mem::size_of::<RAWINPUTHEADER>() as UINT,
            );

            let mut data = vec![0; pcb_size as usize];

            GetRawInputData(
                l_param as HRAWINPUT,
                RID_INPUT,
                data.as_mut_ptr() as LPVOID,
                &mut pcb_size,
                mem::size_of::<RAWINPUTHEADER>() as UINT,
            );

            let ri = ptr::read(data.as_ptr() as *const RAWINPUT);
            
            println!("{}", ri.data.keyboard().VKey);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, w_param, l_param)
}

fn main() {
    let name = win32_string("KeyToJoy");
    let title = win32_string("KeyToJoy");
    unsafe {
        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: 0 as HINSTANCE,
            lpszClassName: name.as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null_mut(),
        };

        RegisterClassW(&wnd_class);

        let h_window = CreateWindowExW(
            0,
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            ptr::null_mut(),
            ptr::null_mut(),
            0 as HINSTANCE,
            ptr::null_mut(),
        );

        let rid = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: h_window,
        };
        RegisterRawInputDevices([rid].as_ptr(), 1, mem::size_of::<RAWINPUTDEVICE>() as UINT);

        let mut message: MSG = mem::zeroed();
        while GetMessageW(&mut message as *mut MSG, h_window, 0, 0) > 0 {
            TranslateMessage(&message as *const MSG);
            DispatchMessageW(&message as *const MSG);
        }
    }
}

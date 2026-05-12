#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::iter::once;
use std::ptr::{null, null_mut};

type Bool = i32;
type Dword = u32;
type Hbrush = *mut c_void;
type Hcursor = *mut c_void;
type Hdc = *mut c_void;
type Hfont = *mut c_void;
type Hicon = *mut c_void;
type Hinstance = *mut c_void;
type Hmenu = *mut c_void;
type Hwnd = *mut c_void;
type Lparam = isize;
type Lpcwstr = *const u16;
type Lresult = isize;
type Uint = u32;
type Wparam = usize;

type WndProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;

#[derive(Copy, Clone)]
struct LayoutItem {
    hwnd: Hwnd,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

const EMPTY_LAYOUT_ITEM: LayoutItem = LayoutItem {
    hwnd: null_mut(),
    x: 0,
    y: 0,
    width: 0,
    height: 0,
};

const WS_OVERLAPPEDWINDOW: Dword = 0x00CF0000;
const WS_VISIBLE: Dword = 0x10000000;
const WS_CHILD: Dword = 0x40000000;
const WS_BORDER: Dword = 0x00800000;
const WS_TABSTOP: Dword = 0x00010000;
const WS_VSCROLL: Dword = 0x00200000;
const WS_EX_CLIENTEDGE: Dword = 0x00000200;

const ES_MULTILINE: Dword = 0x00000004;
const ES_AUTOHSCROLL: Dword = 0x00000080;
const ES_AUTOVSCROLL: Dword = 0x00000040;
const ES_READONLY: Dword = 0x00000800;

const LBS_NOTIFY: Dword = 0x00000001;
const CBS_DROPDOWNLIST: Dword = 0x00000003;

const BS_PUSHBUTTON: Dword = 0x00000000;
const BS_DEFPUSHBUTTON: Dword = 0x00000001;
const BS_AUTOCHECKBOX: Dword = 0x00000003;
const BS_GROUPBOX: Dword = 0x00000007;

const SS_CENTER: Dword = 0x00000001;
const SS_CENTERIMAGE: Dword = 0x00000200;

const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
const SW_SHOW: i32 = 5;

const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_SIZE: Uint = 0x0005;
const WM_GETMINMAXINFO: Uint = 0x0024;
const WM_COMMAND: Uint = 0x0111;
const WM_SETFONT: Uint = 0x0030;
const WM_CTLCOLORMSGBOX: Uint = 0x0132;
const WM_CTLCOLOREDIT: Uint = 0x0133;
const WM_CTLCOLORLISTBOX: Uint = 0x0134;
const WM_CTLCOLORBTN: Uint = 0x0135;
const WM_CTLCOLORSTATIC: Uint = 0x0138;

const LB_ADDSTRING: Uint = 0x0180;
const LB_DELETESTRING: Uint = 0x0182;
const LB_RESETCONTENT: Uint = 0x0184;
const LB_GETCURSEL: Uint = 0x0188;
const LB_ERR: Lresult = -1;

const CB_ADDSTRING: Uint = 0x0143;
const CB_GETCURSEL: Uint = 0x0147;
const CB_SETCURSEL: Uint = 0x014E;

const BM_GETCHECK: Uint = 0x00F0;
const BM_SETCHECK: Uint = 0x00F1;
const BST_CHECKED: Lresult = 1;

const DEFAULT_GUI_FONT: i32 = 17;
const IDC_ARROW: usize = 32512;
const IDI_APPLICATION: usize = 32512;

const TRANSPARENT: i32 = 1;
const DEFAULT_CHARSET: Dword = 1;
const CLEARTYPE_QUALITY: Dword = 5;
const VARIABLE_PITCH: Dword = 2;
const FF_SWISS: Dword = 32;
const FW_NORMAL: i32 = 400;
const FW_SEMIBOLD: i32 = 600;
const FW_BOLD: i32 = 700;

const BASE_CLIENT_WIDTH: i32 = 1145;
const BASE_CLIENT_HEIGHT: i32 = 721;
const MIN_TRACK_WIDTH: i32 = 980;
const MIN_TRACK_HEIGHT: i32 = 620;
const MAX_LAYOUT_ITEMS: usize = 128;

const ID_MODULE_DASHBOARD: i32 = 1001;
const ID_MODULE_TICKETS: i32 = 1002;
const ID_MODULE_INVENTORY: i32 = 1003;
const ID_MODULE_JOBS: i32 = 1004;
const ID_MODULE_REPORTS: i32 = 1005;

const ID_TICKET_CREATE: i32 = 1101;
const ID_TICKET_ESCALATE: i32 = 1102;
const ID_AUDIT_RUN: i32 = 1201;
const ID_SYNC_QUEUE: i32 = 1202;
const ID_RECALC_SLA: i32 = 1203;
const ID_CLEAR_LOGS: i32 = 1204;
const ID_ASSET_REFRESH: i32 = 1301;
const ID_ASSET_RESERVE: i32 = 1302;
const ID_ASSET_MISSING: i32 = 1303;
const ID_TASK_ADD: i32 = 1401;
const ID_TASK_DONE: i32 = 1402;
const ID_TASK_REQUEUE: i32 = 1403;
const ID_TOGGLE_AUTO: i32 = 1501;
const ID_TOGGLE_STRICT: i32 = 1502;
const ID_TOGGLE_DRY_RUN: i32 = 1503;

static mut STATUS_LABEL: Hwnd = null_mut();
static mut METRIC_TICKETS: Hwnd = null_mut();
static mut METRIC_RISK: Hwnd = null_mut();
static mut METRIC_JOBS: Hwnd = null_mut();
static mut METRIC_SYNC: Hwnd = null_mut();

static mut OWNER_EDIT: Hwnd = null_mut();
static mut SUMMARY_EDIT: Hwnd = null_mut();
static mut IMPACT_EDIT: Hwnd = null_mut();
static mut PRIORITY_COMBO: Hwnd = null_mut();
static mut TICKET_LIST: Hwnd = null_mut();
static mut TASK_EDIT: Hwnd = null_mut();
static mut TASK_LIST: Hwnd = null_mut();
static mut ASSET_LIST: Hwnd = null_mut();
static mut LOG_LIST: Hwnd = null_mut();
static mut DETAIL_EDIT: Hwnd = null_mut();
static mut AUTO_REFRESH_CHECK: Hwnd = null_mut();
static mut STRICT_MODE_CHECK: Hwnd = null_mut();
static mut DRY_RUN_CHECK: Hwnd = null_mut();

static mut TICKET_COUNTER: u32 = 4200;
static mut OPEN_TICKETS: u32 = 0;
static mut OPEN_TASKS: u32 = 0;
static mut AUDIT_RUNS: u32 = 0;
static mut SYNC_CYCLES: u32 = 0;
static mut RISK_SCORE: u32 = 18;

static mut FONT_NORMAL: Hfont = null_mut();
static mut FONT_TITLE: Hfont = null_mut();
static mut FONT_METRIC: Hfont = null_mut();
static mut BRUSH_BACKGROUND: Hbrush = null_mut();
static mut BRUSH_PANEL: Hbrush = null_mut();
static mut BRUSH_FIELD: Hbrush = null_mut();

static mut LAYOUT_ITEMS: [LayoutItem; MAX_LAYOUT_ITEMS] = [EMPTY_LAYOUT_ITEM; MAX_LAYOUT_ITEMS];
static mut LAYOUT_COUNT: usize = 0;

#[repr(C)]
struct WndClassW {
    style: Uint,
    lpfn_wnd_proc: WndProc,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: Hicon,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: Lpcwstr,
    lpsz_class_name: Lpcwstr,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    pt: Point,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MinMaxInfo {
    pt_reserved: Point,
    pt_max_size: Point,
    pt_max_position: Point,
    pt_min_track_size: Point,
    pt_max_track_size: Point,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: Lpcwstr) -> Hinstance;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: Dword,
        underline: Dword,
        strike_out: Dword,
        char_set: Dword,
        out_precision: Dword,
        clip_precision: Dword,
        quality: Dword,
        pitch_and_family: Dword,
        face_name: Lpcwstr,
    ) -> Hfont;
    fn CreateSolidBrush(color: Dword) -> Hbrush;
    fn DeleteObject(object: *mut c_void) -> Bool;
    fn GetStockObject(index: i32) -> *mut c_void;
    fn SetBkColor(hdc: Hdc, color: Dword) -> Dword;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: Dword) -> Dword;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        ex_style: Dword,
        class_name: Lpcwstr,
        window_name: Lpcwstr,
        style: Dword,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Hmenu,
        instance: Hinstance,
        param: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn DispatchMessageW(msg: *const Msg) -> Lresult;
    fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min_filter: Uint, max_filter: Uint) -> Bool;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
    fn LoadCursorW(instance: Hinstance, cursor_name: Lpcwstr) -> Hcursor;
    fn LoadIconW(instance: Hinstance, icon_name: Lpcwstr) -> Hicon;
    fn MoveWindow(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: Bool) -> Bool;
    fn PostQuitMessage(exit_code: i32);
    fn RegisterClassW(window_class: *const WndClassW) -> u16;
    fn SendMessageW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn SetWindowTextW(hwnd: Hwnd, text: Lpcwstr) -> Bool;
    fn ShowWindow(hwnd: Hwnd, command_show: i32) -> Bool;
    fn TranslateMessage(msg: *const Msg) -> Bool;
    fn UpdateWindow(hwnd: Hwnd) -> Bool;
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(once(0)).collect()
}

fn menu_id(id: i32) -> Hmenu {
    id as usize as Hmenu
}

fn resource_id(id: usize) -> Lpcwstr {
    id as Lpcwstr
}

fn main() {
    unsafe {
        init_theme();

        let instance = GetModuleHandleW(null());
        let class_name = wide("RustInternalOpsWin32");

        let window_class = WndClassW {
            style: 0,
            lpfn_wnd_proc: Some(window_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: LoadIconW(null_mut(), resource_id(IDI_APPLICATION)),
            h_cursor: LoadCursorW(null_mut(), resource_id(IDC_ARROW)),
            hbr_background: BRUSH_BACKGROUND,
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
        };

        if RegisterClassW(&window_class) == 0 {
            panic!("could not register Win32 window class");
        }

        let title = wide("Rust Win32 Internal Operations Desk");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1180,
            760,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );

        if hwnd.is_null() {
            panic!("could not create main Win32 window");
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg = Msg {
            hwnd: null_mut(),
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point { x: 0, y: 0 },
        };

        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    msg: Uint,
    w_param: Wparam,
    l_param: Lparam,
) -> Lresult {
    match msg {
        WM_CREATE => {
            create_controls(hwnd);
            0
        }
        WM_SIZE => {
            let width = low_word(l_param) as i32;
            let height = high_word(l_param) as i32;
            if width > 0 && height > 0 {
                layout_controls(width, height);
            }
            0
        }
        WM_GETMINMAXINFO => {
            set_minimum_track_size(l_param);
            0
        }
        WM_COMMAND => {
            dispatch_command((w_param & 0xffff) as i32);
            0
        }
        WM_CTLCOLORMSGBOX | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX | WM_CTLCOLORBTN
        | WM_CTLCOLORSTATIC => color_control(msg, w_param, l_param),
        WM_DESTROY => {
            cleanup_theme();
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

fn create_controls(hwnd: Hwnd) {
    unsafe {
        let title = label(hwnd, "RUST WIN32 OPS DESK", 20, 18, 290, 28);
        set_control_font(title, FONT_TITLE);
        STATUS_LABEL = label(hwnd, "Ready - local simulation mode", 325, 18, 820, 28);

        group(hwnd, "Modules", 16, 58, 150, 640);
        button(hwnd, "Dashboard", 32, 88, 112, 30, ID_MODULE_DASHBOARD);
        button(hwnd, "Tickets", 32, 126, 112, 30, ID_MODULE_TICKETS);
        button(hwnd, "Inventory", 32, 164, 112, 30, ID_MODULE_INVENTORY);
        button(hwnd, "Jobs", 32, 202, 112, 30, ID_MODULE_JOBS);
        button(hwnd, "Reports", 32, 240, 112, 30, ID_MODULE_REPORTS);
        label(hwnd, "Mode: Offline / Internal", 32, 300, 112, 26);
        label(hwnd, "Source: Win32 API", 32, 330, 112, 26);
        label(hwnd, "No GUI crates", 32, 360, 112, 26);

        METRIC_TICKETS = metric(hwnd, "Open Tickets", "0", 190, 58);
        METRIC_RISK = metric(hwnd, "Risk Score", "18", 420, 58);
        METRIC_JOBS = metric(hwnd, "Open Jobs", "0", 650, 58);
        METRIC_SYNC = metric(hwnd, "Sync Cycles", "0", 880, 58);

        group(hwnd, "Ticket Intake", 190, 145, 405, 255);
        label(hwnd, "Owner", 210, 175, 70, 22);
        OWNER_EDIT = edit(hwnd, "AES", 285, 172, 130, 25);
        label(hwnd, "Priority", 425, 175, 65, 22);
        PRIORITY_COMBO = combo(hwnd, 492, 172, 80, 120);
        combo_add(PRIORITY_COMBO, "Low");
        combo_add(PRIORITY_COMBO, "Medium");
        combo_add(PRIORITY_COMBO, "High");
        combo_add(PRIORITY_COMBO, "Critical");
        SendMessageW(PRIORITY_COMBO, CB_SETCURSEL, 1, 0);

        label(hwnd, "Summary", 210, 210, 70, 22);
        SUMMARY_EDIT = edit(hwnd, "Payment reconciliation mismatch", 285, 207, 287, 25);
        label(hwnd, "Impact", 210, 245, 70, 22);
        IMPACT_EDIT = edit(hwnd, "Blocks daily finance report", 285, 242, 287, 25);
        button(hwnd, "Create Ticket", 210, 284, 112, 30, ID_TICKET_CREATE);
        button(hwnd, "Escalate", 332, 284, 96, 30, ID_TICKET_ESCALATE);
        button(hwnd, "Recalc SLA", 438, 284, 96, 30, ID_RECALC_SLA);
        TICKET_LIST = listbox(hwnd, 210, 325, 362, 58);

        group(hwnd, "Operations", 615, 145, 245, 255);
        button(hwnd, "Run Audit", 635, 176, 92, 30, ID_AUDIT_RUN);
        button(hwnd, "Sync Queue", 738, 176, 96, 30, ID_SYNC_QUEUE);
        button(hwnd, "Clear Logs", 635, 216, 92, 30, ID_CLEAR_LOGS);
        button(hwnd, "Reports", 738, 216, 96, 30, ID_MODULE_REPORTS);
        label(hwnd, "Audit checks: users, jobs, assets, SLA, retry windows", 635, 262, 195, 42);
        label(hwnd, "Queue sync updates local counters and pushes status events.", 635, 318, 195, 52);

        group(hwnd, "Rules / Execution Flags", 880, 145, 245, 255);
        AUTO_REFRESH_CHECK = checkbox(hwnd, "Auto refresh", 900, 178, 160, 24, ID_TOGGLE_AUTO);
        STRICT_MODE_CHECK = checkbox(hwnd, "Strict validation", 900, 210, 160, 24, ID_TOGGLE_STRICT);
        DRY_RUN_CHECK = checkbox(hwnd, "Dry run actions", 900, 242, 160, 24, ID_TOGGLE_DRY_RUN);
        SendMessageW(AUTO_REFRESH_CHECK, BM_SETCHECK, BST_CHECKED as Wparam, 0);
        SendMessageW(DRY_RUN_CHECK, BM_SETCHECK, BST_CHECKED as Wparam, 0);
        label(hwnd, "Flags change how simulated actions are logged. Dry run never mutates external systems.", 900, 286, 195, 62);

        group(hwnd, "Inventory Watch", 190, 418, 315, 230);
        ASSET_LIST = listbox(hwnd, 210, 450, 275, 112);
        button(hwnd, "Refresh", 210, 575, 78, 30, ID_ASSET_REFRESH);
        button(hwnd, "Reserve", 300, 575, 78, 30, ID_ASSET_RESERVE);
        button(hwnd, "Missing", 390, 575, 78, 30, ID_ASSET_MISSING);
        label(hwnd, "Assets are local records for operator triage.", 210, 615, 260, 22);

        group(hwnd, "Work Queue", 525, 418, 330, 230);
        TASK_EDIT = edit(hwnd, "Review nightly variance report", 545, 450, 285, 25);
        button(hwnd, "Add Task", 545, 485, 85, 30, ID_TASK_ADD);
        button(hwnd, "Done", 640, 485, 72, 30, ID_TASK_DONE);
        button(hwnd, "Requeue", 722, 485, 85, 30, ID_TASK_REQUEUE);
        TASK_LIST = listbox(hwnd, 545, 525, 285, 92);

        group(hwnd, "Event Stream", 875, 418, 250, 230);
        LOG_LIST = listbox(hwnd, 895, 450, 210, 160);
        label(hwnd, "Latest actions, warnings, and simulated system events.", 895, 618, 210, 22);

        DETAIL_EDIT = multiline_edit(hwnd, "Select or trigger an operation to see details.", 190, 665, 935, 42);

        seed_data();
        update_metrics();
        update_options_status();
        layout_current_client(hwnd);
    }
}

fn dispatch_command(id: i32) {
    match id {
        ID_MODULE_DASHBOARD => show_module("Dashboard", "Dashboard loaded: metrics, queue depth, and risk posture are visible."),
        ID_MODULE_TICKETS => show_module("Tickets", "Ticket console loaded: intake form, escalation flow, and SLA recalculation."),
        ID_MODULE_INVENTORY => show_module("Inventory", "Inventory watch loaded: reserve, refresh, and missing-asset workflows."),
        ID_MODULE_JOBS => show_module("Jobs", "Work queue loaded: add, complete, and requeue operator jobs."),
        ID_MODULE_REPORTS => show_module("Reports", "Reports module staged: audit digest, incident summary, and daily export."),
        ID_TICKET_CREATE => create_ticket(),
        ID_TICKET_ESCALATE => escalate_ticket(),
        ID_AUDIT_RUN => run_audit(),
        ID_SYNC_QUEUE => sync_queue(),
        ID_RECALC_SLA => recalc_sla(),
        ID_CLEAR_LOGS => clear_logs(),
        ID_ASSET_REFRESH => refresh_assets(),
        ID_ASSET_RESERVE => reserve_asset(),
        ID_ASSET_MISSING => mark_asset_missing(),
        ID_TASK_ADD => add_task(),
        ID_TASK_DONE => complete_task(),
        ID_TASK_REQUEUE => requeue_task(),
        ID_TOGGLE_AUTO | ID_TOGGLE_STRICT | ID_TOGGLE_DRY_RUN => update_options_status(),
        _ => {}
    }
}

fn create_ticket() {
    unsafe {
        TICKET_COUNTER += 1;
        OPEN_TICKETS += 1;
        RISK_SCORE = (RISK_SCORE + priority_weight()).min(99);

        let ticket_id = TICKET_COUNTER;
        let owner = read_text(OWNER_EDIT);
        let summary = fallback(read_text(SUMMARY_EDIT), "Untitled incident");
        let impact = fallback(read_text(IMPACT_EDIT), "Impact not provided");
        let priority = priority_name();
        let item = format!("#{ticket_id} [{priority}] {summary} - {owner}");

        list_add(TICKET_LIST, &item);
        log_event(&format!("created ticket #{ticket_id} with {priority} priority"));
        set_detail(&format!(
            "Ticket #{ticket_id}\r\nOwner: {owner}\r\nPriority: {priority}\r\nSummary: {summary}\r\nImpact: {impact}"
        ));
        set_status(&format!("Ticket #{ticket_id} created and queued for triage."));
        update_metrics();
    }
}

fn escalate_ticket() {
    unsafe {
        RISK_SCORE = (RISK_SCORE + 9).min(99);
        log_event("manual escalation requested");
        set_detail("Escalation package prepared: owner notified, audit marker added, and SLA warning raised.");
        set_status("Escalation flow completed in local simulation mode.");
        update_metrics();
    }
}

fn run_audit() {
    unsafe {
        AUDIT_RUNS += 1;
        let audit_runs = AUDIT_RUNS;
        RISK_SCORE = RISK_SCORE.saturating_sub(3);

        log_event(&format!("audit run {audit_runs}: users ok, queue ok, assets need review"));
        log_event("audit warning: 2 assets have stale ownership metadata");
        set_detail(
            "Audit completed:\r\n- Access map checked\r\n- Queue retries checked\r\n- Asset ownership sampled\r\n- SLA clock recalculated",
        );
        set_status("Audit completed. Two inventory warnings remain open.");
        update_metrics();
    }
}

fn sync_queue() {
    unsafe {
        SYNC_CYCLES += 1;
        let sync_cycles = SYNC_CYCLES;
        log_event(&format!("queue sync cycle {sync_cycles}: 12 inbound, 4 deferred, 0 failed"));
        set_detail("Queue sync completed in dry-run style. Local counters and event stream were updated.");
        set_status("Queue sync simulated successfully.");
        update_metrics();
    }
}

fn recalc_sla() {
    unsafe {
        let open_tickets = OPEN_TICKETS;
        let priority = priority_name();
        let target = match priority.as_str() {
            "Critical" => "15 minutes",
            "High" => "1 hour",
            "Medium" => "4 hours",
            _ => "1 business day",
        };
        log_event(&format!("SLA recalculated for {open_tickets} open ticket(s)"));
        set_detail(&format!("SLA policy selected from current priority: {priority}\r\nTarget response: {target}"));
        set_status("SLA calculation refreshed.");
    }
}

fn clear_logs() {
    list_reset(unsafe { LOG_LIST });
    log_event("event stream cleared");
    set_detail("Log view reset. Operational counters were left unchanged.");
    set_status("Logs cleared.");
}

fn refresh_assets() {
    list_reset(unsafe { ASSET_LIST });
    for asset in [
        "Laptop-FIN-014 | assigned | finance",
        "Scanner-WH-009 | idle | warehouse",
        "POS-Front-003 | stale owner | retail",
        "NAS-Backups-02 | healthy | infra",
        "Tablet-QA-017 | pending return | QA",
        "Printer-HR-004 | low toner | HR",
    ] {
        list_add(unsafe { ASSET_LIST }, asset);
    }
    log_event("inventory refreshed from local sample records");
    set_detail("Inventory refresh rebuilt the visible asset list and preserved current ticket data.");
    set_status("Inventory watch refreshed.");
}

fn reserve_asset() {
    log_event("asset reservation staged");
    set_detail("Reservation staged. In dry-run mode this records intent without changing external inventory.");
    set_status("Asset reservation staged.");
}

fn mark_asset_missing() {
    unsafe {
        RISK_SCORE = (RISK_SCORE + 5).min(99);
        log_event("asset marked as missing for investigation");
        set_detail("Missing asset workflow opened: ownership review, last-seen timestamp, and follow-up task should be attached.");
        set_status("Missing asset marker added.");
        update_metrics();
    }
}

fn add_task() {
    unsafe {
        let task = fallback(read_text(TASK_EDIT), "Untitled follow-up task");
        OPEN_TASKS += 1;
        let open_tasks = OPEN_TASKS;

        list_add(TASK_LIST, &format!("[open] {task}"));
        log_event(&format!("task added: {task}"));
        set_detail(&format!("Task added to work queue.\r\nOpen tasks: {open_tasks}\r\nTask: {task}"));
        set_status("Task added.");
        update_metrics();
    }
}

fn complete_task() {
    unsafe {
        let selected = SendMessageW(TASK_LIST, LB_GETCURSEL, 0, 0);
        if selected == LB_ERR {
            log_event("complete task requested with no selection");
            set_status("Choose a task before marking it done.");
            return;
        }

        SendMessageW(TASK_LIST, LB_DELETESTRING, selected as Wparam, 0);
        OPEN_TASKS = OPEN_TASKS.saturating_sub(1);
        log_event("task completed and removed from queue");
        set_detail("Selected task was completed and removed from the visible queue.");
        set_status("Task completed.");
        update_metrics();
    }
}

fn requeue_task() {
    log_event("task requeue requested");
    set_detail("Task requeue keeps the visible item in place and adds a priority marker to the event stream.");
    set_status("Task requeue marker added.");
}

fn show_module(name: &str, detail: &str) {
    log_event(&format!("module selected: {name}"));
    set_detail(detail);
    set_status(&format!("{name} module active."));
}

fn update_options_status() {
    let auto = checkbox_checked(unsafe { AUTO_REFRESH_CHECK });
    let strict = checkbox_checked(unsafe { STRICT_MODE_CHECK });
    let dry = checkbox_checked(unsafe { DRY_RUN_CHECK });
    let status = format!(
        "Flags: auto-refresh={}, strict-validation={}, dry-run={}",
        on_off(auto),
        on_off(strict),
        on_off(dry)
    );
    set_status(&status);
    log_event(&status);
}

fn update_metrics() {
    unsafe {
        let open_tickets = OPEN_TICKETS;
        let risk_score = RISK_SCORE;
        let open_tasks = OPEN_TASKS;
        let sync_cycles = SYNC_CYCLES;

        set_text(METRIC_TICKETS, &open_tickets.to_string());
        set_text(METRIC_RISK, &format!("{risk_score}/99"));
        set_text(METRIC_JOBS, &open_tasks.to_string());
        set_text(METRIC_SYNC, &sync_cycles.to_string());
    }
}

fn seed_data() {
    for asset in [
        "Laptop-FIN-014 | assigned | finance",
        "Scanner-WH-009 | idle | warehouse",
        "POS-Front-003 | stale owner | retail",
        "NAS-Backups-02 | healthy | infra",
        "Tablet-QA-017 | pending return | QA",
    ] {
        list_add(unsafe { ASSET_LIST }, asset);
    }

    for task in [
        "[open] Review nightly variance report",
        "[open] Confirm warehouse scanner owner",
        "[open] Validate queue retry policy",
    ] {
        list_add(unsafe { TASK_LIST }, task);
    }

    unsafe {
        OPEN_TASKS = 3;
    }

    log_event("application initialized");
    log_event("local sample data loaded");
    set_detail("Internal operations desk is ready. Use the crowded controls to simulate tickets, tasks, audits, and inventory workflows.");
}

fn metric(parent: Hwnd, title: &str, value: &str, x: i32, y: i32) -> Hwnd {
    label(parent, title, x, y, 210, 24);
    let value_hwnd = control_ex(
        WS_EX_CLIENTEDGE,
        "STATIC",
        value,
        WS_CHILD | WS_VISIBLE | WS_BORDER | SS_CENTER | SS_CENTERIMAGE,
        x,
        y + 27,
        210,
        42,
        parent,
        0,
    );
    set_control_font(value_hwnd, unsafe { FONT_METRIC });
    value_hwnd
}

fn group(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control(
        "BUTTON",
        text,
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
        x,
        y,
        width,
        height,
        parent,
        0,
    )
}

fn label(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control("STATIC", text, WS_CHILD | WS_VISIBLE, x, y, width, height, parent, 0)
}

fn button(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32, id: i32) -> Hwnd {
    control(
        "BUTTON",
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON | BS_DEFPUSHBUTTON,
        x,
        y,
        width,
        height,
        parent,
        id,
    )
}

fn checkbox(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32, id: i32) -> Hwnd {
    control(
        "BUTTON",
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
        x,
        y,
        width,
        height,
        parent,
        id,
    )
}

fn edit(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control_ex(
        WS_EX_CLIENTEDGE,
        "EDIT",
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL,
        x,
        y,
        width,
        height,
        parent,
        0,
    )
}

fn multiline_edit(parent: Hwnd, text: &str, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control_ex(
        WS_EX_CLIENTEDGE,
        "EDIT",
        text,
        WS_CHILD | WS_VISIBLE | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY | WS_VSCROLL,
        x,
        y,
        width,
        height,
        parent,
        0,
    )
}

fn listbox(parent: Hwnd, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control_ex(
        WS_EX_CLIENTEDGE,
        "LISTBOX",
        "",
        WS_CHILD | WS_VISIBLE | WS_BORDER | WS_VSCROLL | LBS_NOTIFY,
        x,
        y,
        width,
        height,
        parent,
        0,
    )
}

fn combo(parent: Hwnd, x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    control(
        "COMBOBOX",
        "",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | CBS_DROPDOWNLIST,
        x,
        y,
        width,
        height,
        parent,
        0,
    )
}

fn control(
    class_name: &str,
    text: &str,
    style: Dword,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: Hwnd,
    id: i32,
) -> Hwnd {
    control_ex(0, class_name, text, style, x, y, width, height, parent, id)
}

fn control_ex(
    ex_style: Dword,
    class_name: &str,
    text: &str,
    style: Dword,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: Hwnd,
    id: i32,
) -> Hwnd {
    unsafe {
        let instance = GetModuleHandleW(null());
        let class_name = wide(class_name);
        let text = wide(text);
        let hwnd = CreateWindowExW(
            ex_style,
            class_name.as_ptr(),
            text.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            parent,
            if id == 0 { null_mut() } else { menu_id(id) },
            instance,
            null_mut(),
        );
        apply_font(hwnd);
        register_layout(hwnd, x, y, width, height);
        hwnd
    }
}

fn apply_font(hwnd: Hwnd) {
    unsafe {
        let font = if FONT_NORMAL.is_null() {
            GetStockObject(DEFAULT_GUI_FONT)
        } else {
            FONT_NORMAL
        };
        SendMessageW(hwnd, WM_SETFONT, font as Wparam, 1);
    }
}

fn set_control_font(hwnd: Hwnd, font: Hfont) {
    unsafe {
        if !hwnd.is_null() && !font.is_null() {
            SendMessageW(hwnd, WM_SETFONT, font as Wparam, 1);
        }
    }
}

fn set_text(hwnd: Hwnd, text: &str) {
    unsafe {
        SetWindowTextW(hwnd, wide(text).as_ptr());
    }
}

fn read_text(hwnd: Hwnd) -> String {
    let mut buffer = [0_u16; 512];
    let length = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if length <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buffer[..length as usize])
}

fn combo_add(hwnd: Hwnd, text: &str) {
    unsafe {
        let text = wide(text);
        SendMessageW(hwnd, CB_ADDSTRING, 0, text.as_ptr() as Lparam);
    }
}

fn list_add(hwnd: Hwnd, text: &str) {
    unsafe {
        let text = wide(text);
        SendMessageW(hwnd, LB_ADDSTRING, 0, text.as_ptr() as Lparam);
    }
}

fn list_reset(hwnd: Hwnd) {
    unsafe {
        SendMessageW(hwnd, LB_RESETCONTENT, 0, 0);
    }
}

fn log_event(text: &str) {
    let line = format!("> {text}");
    list_add(unsafe { LOG_LIST }, &line);
}

fn set_detail(text: &str) {
    set_text(unsafe { DETAIL_EDIT }, text);
}

fn set_status(text: &str) {
    set_text(unsafe { STATUS_LABEL }, text);
}

fn priority_name() -> String {
    let index = unsafe { SendMessageW(PRIORITY_COMBO, CB_GETCURSEL, 0, 0) };
    match index {
        0 => "Low",
        2 => "High",
        3 => "Critical",
        _ => "Medium",
    }
    .to_string()
}

fn priority_weight() -> u32 {
    match priority_name().as_str() {
        "Critical" => 14,
        "High" => 9,
        "Medium" => 5,
        _ => 2,
    }
}

fn checkbox_checked(hwnd: Hwnd) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) == BST_CHECKED }
}

fn fallback(value: String, default_value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        default_value.to_string()
    } else {
        value.to_string()
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}

fn init_theme() {
    unsafe {
        FONT_NORMAL = create_font(-15, FW_NORMAL);
        FONT_TITLE = create_font(-24, FW_BOLD);
        FONT_METRIC = create_font(-26, FW_SEMIBOLD);
        BRUSH_BACKGROUND = CreateSolidBrush(rgb(242, 245, 249));
        BRUSH_PANEL = CreateSolidBrush(rgb(242, 245, 249));
        BRUSH_FIELD = CreateSolidBrush(rgb(255, 255, 255));
    }
}

fn cleanup_theme() {
    unsafe {
        for object in [
            FONT_NORMAL as *mut c_void,
            FONT_TITLE as *mut c_void,
            FONT_METRIC as *mut c_void,
            BRUSH_BACKGROUND as *mut c_void,
            BRUSH_PANEL as *mut c_void,
            BRUSH_FIELD as *mut c_void,
        ] {
            if !object.is_null() {
                DeleteObject(object);
            }
        }
    }
}

fn create_font(height: i32, weight: i32) -> Hfont {
    unsafe {
        let face = wide("Segoe UI");
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            0,
            0,
            CLEARTYPE_QUALITY,
            VARIABLE_PITCH | FF_SWISS,
            face.as_ptr(),
        )
    }
}

fn color_control(msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult {
    unsafe {
        let hdc = w_param as Hdc;
        let control_hwnd = l_param as Hwnd;

        match msg {
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                SetBkColor(hdc, rgb(255, 255, 255));
                SetTextColor(hdc, rgb(31, 41, 55));
                BRUSH_FIELD as Lresult
            }
            WM_CTLCOLORBTN => {
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, rgb(31, 41, 55));
                BRUSH_PANEL as Lresult
            }
            _ => {
                SetBkMode(hdc, TRANSPARENT);
                if control_hwnd == METRIC_RISK {
                    SetTextColor(hdc, rgb(170, 58, 44));
                } else if is_metric_control(control_hwnd) {
                    SetTextColor(hdc, rgb(18, 82, 128));
                } else {
                    SetTextColor(hdc, rgb(31, 41, 55));
                }
                BRUSH_PANEL as Lresult
            }
        }
    }
}

fn is_metric_control(hwnd: Hwnd) -> bool {
    unsafe {
        hwnd == METRIC_TICKETS || hwnd == METRIC_RISK || hwnd == METRIC_JOBS || hwnd == METRIC_SYNC
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> Dword {
    red as Dword | ((green as Dword) << 8) | ((blue as Dword) << 16)
}

fn low_word(value: Lparam) -> u16 {
    (value as usize & 0xffff) as u16
}

fn high_word(value: Lparam) -> u16 {
    ((value as usize >> 16) & 0xffff) as u16
}

fn set_minimum_track_size(l_param: Lparam) {
    unsafe {
        let info = l_param as *mut MinMaxInfo;
        if !info.is_null() {
            (*info).pt_min_track_size.x = MIN_TRACK_WIDTH;
            (*info).pt_min_track_size.y = MIN_TRACK_HEIGHT;
        }
    }
}

fn layout_current_client(hwnd: Hwnd) {
    unsafe {
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        if GetClientRect(hwnd, &mut rect) != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            layout_controls(width, height);
        }
    }
}

fn register_layout(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32) {
    if hwnd.is_null() {
        return;
    }

    unsafe {
        let index = LAYOUT_COUNT;
        if index >= MAX_LAYOUT_ITEMS {
            return;
        }

        let item = LayoutItem {
            hwnd,
            x,
            y,
            width,
            height,
        };
        let ptr = std::ptr::addr_of_mut!(LAYOUT_ITEMS).cast::<LayoutItem>().add(index);
        ptr.write(item);
        LAYOUT_COUNT = index + 1;
    }
}

fn layout_controls(client_width: i32, client_height: i32) {
    if client_width <= 0 || client_height <= 0 {
        return;
    }

    let count = unsafe { LAYOUT_COUNT };
    for index in 0..count {
        let item = unsafe {
            std::ptr::addr_of!(LAYOUT_ITEMS)
                .cast::<LayoutItem>()
                .add(index)
                .read()
        };

        if item.hwnd.is_null() {
            continue;
        }

        let x = scale_pos(item.x, client_width, BASE_CLIENT_WIDTH);
        let y = scale_pos(item.y, client_height, BASE_CLIENT_HEIGHT);
        let width = scale_size(item.width, client_width, BASE_CLIENT_WIDTH).max(18);
        let height = scale_size(item.height, client_height, BASE_CLIENT_HEIGHT).max(18);

        unsafe {
            MoveWindow(item.hwnd, x, y, width, height, 1);
        }
    }
}

fn scale_pos(value: i32, current: i32, base: i32) -> i32 {
    ((value as i64 * current as i64) / base as i64) as i32
}

fn scale_size(value: i32, current: i32, base: i32) -> i32 {
    (((value as i64 * current as i64) + (base as i64 / 2)) / base as i64) as i32
}

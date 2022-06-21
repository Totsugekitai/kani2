use spin::Mutex;
use x86_64::{registers::control::Cr3Flags, structures::paging::frame::PhysFrame, PhysAddr};

/// タスクが今どのような状態なのかを表す
#[derive(Debug, Clone, Copy)]
pub enum TaskStatus {
    /// 初期化中のタスク
    Init,
    /// 実行中のタスク
    Run,
    /// 実行可能だが実行していないタスク
    Wait,
    /// 休眠中のタスク
    Sleep,
    /// 終了中のタスク
    Dead,
}

static TID_COUNTER: Mutex<u64> = Mutex::new(0);

#[derive(Debug, Clone, Copy)]
struct Tid(u64);

impl Tid {
    fn new() -> Self {
        let mut tid_counter = TID_COUNTER.lock();
        let tid = Self(*tid_counter);
        *tid_counter += 1;
        tid
    }
}

/// プログラムの実行単位
#[derive(Debug, Clone)]
pub struct Task {
    /// Task ID
    /// 実行中のタスクは一意に割り振られる
    tid: Tid,
    /// タスクの状態
    status: TaskStatus,
    /// 汎用レジスタ
    /// タスクスイッチの際に保存領域として用いられる
    regs: Registers,
    /// P4 tableの物理フレーム
    p4_table_address: PhysFrame,
    /// cr3のフラグ
    cr3_flags: Cr3Flags,
}

impl Task {
    /// タスクを生成する
    pub fn new() -> Self {
        Self {
            tid: Tid::new(),
            status: TaskStatus::Init,
            regs: Registers::new(),
            p4_table_address: PhysFrame::from_start_address(PhysAddr::new(0)).unwrap(),
            cr3_flags: Cr3Flags::empty(),
        }
    }

    /// タスクに割り振られているTIDを返す
    pub fn tid(&self) -> Tid {
        self.tid
    }

    /// タスクの状態を返す
    pub fn status(&self) -> TaskStatus {
        self.status
    }

    /// 保存している汎用レジスタの参照を返す
    fn regs(&self) -> &Registers {
        &self.regs
    }

    /// 保存しているP4 tableの物理フレームの参照を返す
    fn p4_table_address(&self) -> &PhysFrame {
        &self.p4_table_address
    }

    /// 保存しているcr3のフラグを返す
    fn cr3_flags(&self) -> Cr3Flags {
        self.cr3_flags
    }
}

/// タスクがスイッチする際に保存するレジスタ
#[repr(C)]
#[derive(Debug, Clone)]
struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rbp: u64,
    rsp: u64,
    rip: u64,
    rflags: u64,
}

impl Registers {
    fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rdi: 0,
            rsi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: 0,
            rsp: 0,
            rip: 0,
            rflags: 0,
        }
    }
}

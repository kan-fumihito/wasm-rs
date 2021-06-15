#[derive(Serialize, Deserialize, Debug)]
pub struct Stack {
    // not empty
    pub stack: Vec<FrameStack>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameStack {
    pub frame: Frame,
    // not empty
    // stack[0]の継続は空
    pub stack: Vec<LabelStack>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabelStack {
    pub label: Label,
    // 後ろから実行
    // #[serde(skip)]
    pub instrs: Vec<AdminInstr>,
    pub stack: Vec<Val>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub locals: Vec<Val>,
    pub n: usize,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Label {
    // 前から実行
    pub instrs: Vec<Instr>,
    pub n: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AdminInstr {
    Instr(Instr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Br(LabelIdx),
    Return,
}
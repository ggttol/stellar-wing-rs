//! 全局可调常量。等价于原 shooter.html 的 CFG 对象。
//! 后续里程碑会在此处补充武器、卡池、波次等数值。

pub struct Config {
    pub w: f32,
    pub h: f32,
}

pub const CFG: Config = Config { w: 480.0, h: 800.0 };

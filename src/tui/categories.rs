pub const WORKSPACE_INDEX: usize = 22;

pub const COUNT: usize = 28;

pub struct Category {
    pub icon: &'static str,
    pub name_key: &'static str,
}

pub const LIST: [Category; COUNT] = [
    Category { icon: "🔍", name_key: "category.1" },
    Category { icon: "🎯", name_key: "category.2" },
    Category { icon: "🚪", name_key: "category.3" },
    Category { icon: "📦", name_key: "category.4" },
    Category { icon: "🌐", name_key: "category.5" },
    Category { icon: "🔐", name_key: "category.6" },
    Category { icon: "📧", name_key: "category.7" },
    Category { icon: "📁", name_key: "category.8" },
    Category { icon: "🗂️", name_key: "category.9" },
    Category { icon: "🌍", name_key: "category.10" },
    Category { icon: "🔒", name_key: "category.11" },
    Category { icon: "📡", name_key: "category.12" },
    Category { icon: "🎥", name_key: "category.13" },
    Category { icon: "🗄️", name_key: "category.14" },
    Category { icon: "🖥️", name_key: "category.15" },
    Category { icon: "🕵️", name_key: "category.16" },
    Category { icon: "🧬", name_key: "category.17" },
    Category { icon: "📄", name_key: "category.18" },
    Category { icon: "🔧", name_key: "category.19" },
    Category { icon: "🌐", name_key: "category.20" },
    Category { icon: "📊", name_key: "category.21" },
    Category { icon: "🧠", name_key: "category.22" },
    Category { icon: "⚙️", name_key: "category.23" },
    Category { icon: "📶", name_key: "category.24" },
    Category { icon: "📡", name_key: "category.25" },
    Category { icon: "🏭", name_key: "category.26" },
    Category { icon: "🔵", name_key: "category.27" },
    Category { icon: "🎭", name_key: "category.34" },
];

pub fn number(index: usize) -> u16 {
    if index == COUNT - 1 {
        34
    } else {
        (index + 1) as u16
    }
}

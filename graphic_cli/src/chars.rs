pub const FULL: char = '█';
pub const TOP: char = '▀';
pub const BOTTOM: char = '▄';
pub const LEFT: char = '▌';
pub const RIGHT: char = '▐';

pub struct Lines {
    pub vertical: char,
    pub horizontal: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
}

pub const THIN: Lines = Lines {
    vertical: '│',
    horizontal:'─',
    top_left: '┌',
    top_right: '┐',
    bottom_left: '└',
    bottom_right: '┘',
};

pub const HEAVY: Lines = Lines {
    vertical: '┃',
    horizontal:'━',
    top_left: '┏',
    top_right: '┓',
    bottom_left: '┗',
    bottom_right: '┛',
};
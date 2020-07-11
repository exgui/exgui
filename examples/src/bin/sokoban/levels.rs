#[derive(Copy, Clone, PartialEq)]
pub enum Cell {
    Wall,
    Box,
    Docker,
    Place,
    BoxOnPlace,
    DockerOnPlace,
    Space,
}

impl Cell {
    pub const WALL: u8 = b'O';
    pub const BOX: u8 = b'#';
    pub const DOCKER: u8 = b'*';
    pub const PLACE: u8 = b'+';
    pub const BOX_ON_PLACE: u8 = b'!';
    pub const DOCKER_ON_PLACE: u8 = b'@';
    pub const SPACE: u8 = b'-';

    pub fn contains_docker(&self) -> bool {
        match self {
            Self::Docker | Self::DockerOnPlace => true,
            _ => false,
        }
    }

    pub fn contains_box(&self) -> bool {
        match self {
            Self::Box | Self::BoxOnPlace => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Space | Self::Place => true,
            _ => false,
        }
    }
}

impl From<u8> for Cell {
    fn from(cell: u8) -> Self {
        match cell {
            Self::WALL => Self::Wall,
            Self::BOX => Self::Box,
            Self::DOCKER => Self::Docker,
            Self::PLACE => Self::Place,
            Self::BOX_ON_PLACE => Self::BoxOnPlace,
            Self::DOCKER_ON_PLACE => Self::DockerOnPlace,
            _ => Self::Space,
        }
    }
}

impl From<&u8> for Cell {
    fn from(cell: &u8) -> Self {
        (*cell).into()
    }
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Wall => Cell::WALL,
            Cell::Box => Cell::BOX,
            Cell::Docker => Cell::DOCKER,
            Cell::Place => Cell::PLACE,
            Cell::BoxOnPlace => Cell::BOX_ON_PLACE,
            Cell::DockerOnPlace => Cell::DOCKER_ON_PLACE,
            Cell::Space => Cell::SPACE,
        }
    }
}

pub struct Level {
    number: usize,
    current: Vec<Vec<u8>>,
    levels: Vec<Vec<Vec<u8>>>,
}

impl Level {
    pub fn new() -> Self {
        let mut levels = vec![];
        for line in LEVELS.lines() {
            if !line.is_empty() {
                if line.trim().starts_with("level") {
                    levels.push(vec![]);
                } else if let Some(current_level) = levels.last_mut() {
                    current_level.push(line.trim().into());
                }
            }
        }
        Self {
            number: 0,
            current: levels[0].clone(),
            levels,
        }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn field(&self) -> &[Vec<u8>] {
        &self.current
    }

    pub fn field_mut(&mut self) -> &mut [Vec<u8>] {
        &mut self.current
    }

    pub fn cols(&self) -> usize {
        self.current
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
    }

    pub fn rows(&self) -> usize {
        self.current.len()
    }

    pub fn next(&mut self) -> Option<usize> {
        if self.number < self.levels.len() - 1 {
            self.number += 1;
            self.current = self.levels[self.number].clone();
            Some(self.number)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.current = self.levels[self.number].clone();
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<Cell> {
        self.current
            .get(row)
            .and_then(|line| line.get(col))
            .map(Into::into)
    }

    pub fn is_complete(&self) -> bool {
        for row in 0..self.rows() {
            for col in 0..self.cols() {
                if self.cell(row, col).unwrap() == Cell::Box {
                    return false;
                }
            }
        }
        true
    }

    pub fn go_docker(&mut self, row: usize, col: usize) -> bool {
        if let Some(cell) = self.cell(row, col) {
            let (old_row, old_col) = self.docker_pos();
            match cell {
                Cell::Space => self.current[row][col] = Cell::Docker.into(),
                Cell::Place => self.current[row][col] = Cell::DockerOnPlace.into(),
                _ => return false,
            }
            match self.cell(old_row, old_col).unwrap() {
                Cell::Docker => self.current[old_row][old_col] = Cell::Space.into(),
                Cell::DockerOnPlace => self.current[old_row][old_col] = Cell::Place.into(),
                _ => (),
            }
            true
        } else {
            false
        }
    }

    pub fn go_box(&mut self, old_row: usize, old_col: usize, row: usize, col: usize) -> bool {
        if !self.cell(old_row, old_col).map(|cell| cell.contains_box()).unwrap_or(false) {
            return false;
        }

        if let Some(cell) = self.cell(row, col) {
            match cell {
                Cell::Space => self.current[row][col] = Cell::Box.into(),
                Cell::Place => self.current[row][col] = Cell::BoxOnPlace.into(),
                _ => return false,
            }
            match self.cell(old_row, old_col).unwrap() {
                Cell::Box => self.current[old_row][old_col] = Cell::Space.into(),
                Cell::BoxOnPlace => self.current[old_row][old_col] = Cell::Place.into(),
                _ => (),
            }
            true
        } else {
            false
        }
    }

    pub fn docker_pos(&self) -> (usize, usize) {
        for row in 0..self.rows() {
            for col in 0..self.cols() {
                if self.cell(row, col).unwrap().contains_docker() {
                    return (row, col);
                }
            }
        }
        panic!("Docker on level {} does not exist", self.number);
    }
}

const LEVELS: &'static str = r"
level 0
OOOOO
O--*O
O-#+O
OOOOO

level 1
----OOOOO----------
----O---O----------
----O#--O----------
--OOO--#OO---------
--O--#-#-O---------
OOO-O-OO-O---OOOOOO
O---O-OO-OOOOO--++O
O-#--#----------++O
OOOOO-OOO-O*OO--++O
----O-----OOOOOOOOO
----OOOOOOO--------

level 2
OOOOOOOOOOOO--
O++--O-----OOO
O++--O-#--#--O
O++--O#OOOO--O
O++------OO--O
O++--O-O*-#-OO
OOOOOO-OO#-#-O
--O-#--#-#-#-O
--O----O-----O
--OOOOOOOOOOOO

level 3
--------OOOOOOOO-
--------O-----*O-
--------O-#O#-OO-
--------O-#--#O--
--------OO#-#-O--
OOOOOOOOO-#-O-OOO
O++++--OO-#--#--O
OO+++----#--#---O
O++++--OOOOOOOOOO
OOOOOOOO---------

level 4
--------------OOOOOOOO
--------------O--++++O
---OOOOOOOOOOOO--++++O
---O----O--#-#---++++O
---O-###O#--#-O--++++O
---O--#-----#-O--++++O
---O-##-O#-#-#OOOOOOOO
OOOO--#-O-----O-------
O---O-OOOOOOOOO-------
O----#--OO------------
O-##O##-*O------------
O---O---OO------------
OOOOOOOOO-------------

level 5
--------OOOOO----
--------O---OOOOO
--------O-O#OO--O
--------O-----#-O
OOOOOOOOO-OOO---O
O++++--OO-#--#OOO
O++++----#-##-OO-
O++++--OO#--#-*O-
OOOOOOOOO--#--OO-
--------O-#-#--O-
--------OOO-OO-O-
----------O----O-
----------OOOOOO-
";
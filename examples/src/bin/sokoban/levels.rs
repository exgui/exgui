pub struct Level {
    number: usize,
    levels: Vec<Vec<String>>,
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
            levels,
        }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn field(&self) -> &[String] {
        &self.levels[self.number]
    }

    pub fn field_mut(&mut self) -> &mut [String] {
        &mut self.levels[self.number]
    }

    pub fn cols(&self) -> usize {
        self.levels[self.number]
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
    }

    pub fn rows(&self) -> usize {
        self.levels[self.number].len()
    }

    pub fn next(&mut self) -> Option<usize> {
        if self.number < self.levels.len() - 1 {
            self.number += 1;
            Some(self.number)
        } else {
            None
        }
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
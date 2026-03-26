use std::collections::HashSet;

use crate::map::World;

/// Direções absolutas no mapa (útil para mover/atirar no terminal).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn delta(self) -> (isize, isize) {
        match self {
            Direction::Up => (1, 0),
            Direction::Right => (0, 1),
            Direction::Down => (-1, 0),
            Direction::Left => (0, -1),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Up => "↑",
            Direction::Right => "→",
            Direction::Down => "↓",
            Direction::Left => "←",
        }
    }
}

/// Percepções na casa atual.
#[derive(Clone, Copy, Debug, Default)]
pub struct Percepts {
    pub breeze: bool,
    pub stench: bool,
    pub glitter: bool,
    pub bump: bool,
    pub scream: bool,
}

/// Estado de conhecimento da casa no mapa mental.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellKnowledge {
    pub visited: bool,
    pub breeze_seen: bool,
    pub stench_seen: bool,
    pub glitter_seen: bool,
}

/// Agente baseado em modelo (rastreador de conhecimento),
/// para jogo controlado por humano via terminal.
/// Ele NÃO escolhe ação automática.
#[derive(Debug)]
pub struct ModelAgent {
    pub pos: (usize, usize),
    pub dir: Direction,

    pub has_gold: bool,
    pub has_arrow: bool,
    pub wumpus_alive: bool,

    pub score: i32,
    pub actions_taken: usize,
    pub is_dead: bool,

    size: usize,
    visited: HashSet<(usize, usize)>,
    safe: HashSet<(usize, usize)>,
    possible_pits: HashSet<(usize, usize)>,
    possible_wumpus: HashSet<(usize, usize)>,
    percept_breeze: HashSet<(usize, usize)>,
    percept_stench: HashSet<(usize, usize)>,
    percept_glitter: HashSet<(usize, usize)>,
}

impl ModelAgent {
    pub fn new(start: (usize, usize), size: usize) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);

        let mut safe = HashSet::new();
        safe.insert(start);

        Self {
            pos: start,
            dir: Direction::Right,
            has_gold: false,
            has_arrow: true,
            wumpus_alive: true,
            score: 0,
            actions_taken: 0,
            is_dead: false,
            size,
            visited,
            safe,
            possible_pits: HashSet::new(),
            possible_wumpus: HashSet::new(),
            percept_breeze: HashSet::new(),
            percept_stench: HashSet::new(),
            percept_glitter: HashSet::new(),
        }
    }

    pub fn visited(&self) -> &HashSet<(usize, usize)> {
        &self.visited
    }

    pub fn safe_cells(&self) -> &HashSet<(usize, usize)> {
        &self.safe
    }

    pub fn possible_pits(&self) -> &HashSet<(usize, usize)> {
        &self.possible_pits
    }

    pub fn possible_wumpus(&self) -> &HashSet<(usize, usize)> {
        &self.possible_wumpus
    }

    pub fn percept_breeze_cells(&self) -> &HashSet<(usize, usize)> {
        &self.percept_breeze
    }

    pub fn percept_stench_cells(&self) -> &HashSet<(usize, usize)> {
        &self.percept_stench
    }

    pub fn percept_glitter_cells(&self) -> &HashSet<(usize, usize)> {
        &self.percept_glitter
    }

    /// Percepção real no mundo, para a posição atual do jogador.
    pub fn sense(&self, world: &World, bump: bool, scream: bool) -> Percepts {
        let (r, c) = self.pos;
        Percepts {
            breeze: world.has_adjacent_pit(r, c),
            stench: self.wumpus_alive && world.has_adjacent_wumpus(r, c),
            glitter: world.is_gold(r, c),
            bump,
            scream,
        }
    }

    /// Atualiza o modelo interno com as percepções da casa atual.
    pub fn update_beliefs(&mut self, world: &World, percepts: Percepts) {
        let here = self.pos;

        self.visited.insert(here);
        self.safe.insert(here);

        if percepts.breeze {
            self.percept_breeze.insert(here);
        }
        if percepts.stench {
            self.percept_stench.insert(here);
        }
        if percepts.glitter {
            self.percept_glitter.insert(here);
        }
        if percepts.scream {
            self.wumpus_alive = false;
            self.possible_wumpus.clear();
        }

        let neighbors = world.neighbors(here.0, here.1);

        if !percepts.breeze {
            for n in &neighbors {
                self.possible_pits.remove(n);
            }
        } else {
            for n in &neighbors {
                if !self.visited.contains(n) && !self.safe.contains(n) {
                    self.possible_pits.insert(*n);
                }
            }
        }

        if !percepts.stench || !self.wumpus_alive {
            for n in &neighbors {
                self.possible_wumpus.remove(n);
            }
        } else {
            for n in &neighbors {
                if !self.visited.contains(n) && !self.safe.contains(n) {
                    self.possible_wumpus.insert(*n);
                }
            }
        }

        if !percepts.breeze && !percepts.stench {
            for n in neighbors {
                self.safe.insert(n);
                self.possible_pits.remove(&n);
                self.possible_wumpus.remove(&n);
            }
        }
    }

    /// Move o jogador para frente. Retorna percepções após a ação.
    pub fn move_forward(&mut self, world: &World) -> Percepts {
        self.actions_taken += 1;
        self.score -= 1;

        let (dr, dc) = self.dir.delta();
        let nr = self.pos.0 as isize + dr;
        let nc = self.pos.1 as isize + dc;

        let mut bump = false;
        if !world.in_bounds(nr, nc) {
            bump = true;
        } else {
            self.pos = (nr as usize, nc as usize);
            self.visited.insert(self.pos);

            if world.is_hazard(self.pos.0, self.pos.1) {
                self.is_dead = true;
                self.score -= 100;
            }
        }

        self.sense(world, bump, false)
    }

    pub fn turn_left(&mut self, world: &World) -> Percepts {
        self.actions_taken += 1;
        self.score -= 1;
        self.dir = self.dir.turn_left();
        self.sense(world, false, false)
    }

    pub fn turn_right(&mut self, world: &World) -> Percepts {
        self.actions_taken += 1;
        self.score -= 1;
        self.dir = self.dir.turn_right();
        self.sense(world, false, false)
    }

    /// Atira a flecha na direção atual.
    pub fn shoot(&mut self, world: &mut World) -> Percepts {
        self.actions_taken += 1;
        self.score -= 1;

        let mut scream = false;
        if self.has_arrow {
            self.has_arrow = false;
            let hit = world.shoot_arrow(self.pos, self.dir.delta());
            if hit {
                self.wumpus_alive = false;
                self.score += 50;
                scream = true;
                self.possible_wumpus.clear();
            }
        }

        self.sense(world, false, scream)
    }

    /// Pega ouro na casa atual.
    pub fn grab_gold(&mut self, world: &mut World) -> Percepts {
        self.actions_taken += 1;
        self.score -= 1;

        if world.pickup_gold_at(self.pos.0, self.pos.1) {
            self.has_gold = true;
        }

        self.sense(world, false, false)
    }

    /// Escala para sair da caverna (só vale no start com ouro).
    /// Retorna true quando a partida termina com sucesso.
    pub fn climb_out(&mut self, start: (usize, usize)) -> bool {
        self.actions_taken += 1;
        self.score -= 1;

        if self.pos == start && self.has_gold {
            self.score += 50;
            return true;
        }

        false
    }

    /// Visão do mapa mental do agente:
    /// - A = posição atual do jogador
    /// - B = sentiu brisa nessa casa
    /// - S = sentiu cheiro nessa casa
    /// - G = sentiu brilho nessa casa
    /// - . = sem marcação
    ///
    /// Observação: a linha 0 é impressa embaixo.
    pub fn render_knowledge_map(&self) -> String {
        let mut out = String::new();
        out.push_str("Mapa mental (agente baseado em modelo)\n");
        out.push_str("Legenda: A=agente, B=brisa, S=cheiro, G=brilho, .=sem marcação\n");
        out.push_str("Orientação: linha 0 fica embaixo.\n\n");

        for r in (0..self.size).rev() {
            for c in 0..self.size {
                let pos = (r, c);

                let token = if pos == self.pos {
                    format!("A{}", self.dir.as_str())
                } else if self.visited.contains(&pos) {
                    if self.percept_glitter.contains(&pos) {
                        "G".to_string()
                    } else if self.percept_stench.contains(&pos) && self.percept_breeze.contains(&pos) {
                        "SB".to_string()
                    } else if self.percept_stench.contains(&pos) {
                        "S".to_string()
                    } else if self.percept_breeze.contains(&pos) {
                        "B".to_string()
                    } else {
                        ".".to_string()
                    }
                } else {
                    ".".to_string()
                };

                out.push_str(&format!("{:<6}", token));
            }
            out.push('\n');
        }

        out
    }

    /// Texto curto com estado da partida para UI no terminal.
    pub fn status_line(&self) -> String {
        format!(
            "pos={:?} dir={} score={} gold={} arrow={} wumpus_alive={} dead={}",
            self.pos,
            self.dir.as_str(),
            self.score,
            self.has_gold,
            self.has_arrow,
            self.wumpus_alive,
            self.is_dead
        )
    }
}

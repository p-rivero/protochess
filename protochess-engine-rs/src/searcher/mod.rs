use crate::types::chess_move::{Move, MoveType};
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::evaluator::Evaluator;
use crate::transposition_table::{TranspositionTable, Entry, EntryFlag};


pub(crate) struct Searcher {
    transposition_table: TranspositionTable,
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[u16;256];256],
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher{
            transposition_table: TranspositionTable::new(),
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
        }
    }


    pub fn get_best_move(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8) -> Option<(Move, u8)> {
        self.get_best_move_impl(position, eval, movegen, depth, u64::MAX)
    }

    pub fn get_best_move_timeout(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, time_sec: u64) -> Option<(Move, u8)> {
        self.get_best_move_impl(position, eval, movegen, u8::MAX, time_sec)
    }
    
    fn get_best_move_impl(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8, time_sec: u64) -> Option<(Move, u8)> {
        //Iterative deepening
        self.clear_heuristics();
        self.transposition_table.set_ancient();
        let start = instant::Instant::now();
        let max_time = instant::Duration::from_secs(time_sec);
        
        for d in 1..=depth {
            if start.elapsed() >= max_time {
                break;
            }
            let alpha = -isize::MAX;
            let beta = isize::MAX;
            let best_score = self.alphabeta(position, eval, movegen, d,alpha, beta, true);

            //Print PV info
            println!("score:{} depth: {}", best_score, d);
        }

        match self.transposition_table.retrieve(position.get_zobrist()) {
            Some(entry) => {Some(((&entry.mv).to_owned(), entry.depth))},
            None => None
        }
    }

    fn alphabeta(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8, mut alpha: isize, beta: isize, do_null: bool) -> isize {

        if depth <= 0 {
            return self.quiesce(position, eval, movegen, 0, alpha, beta);
        }

        let is_pv = alpha != beta - 1;
        if let Some(entry) = self.transposition_table.retrieve(position.get_zobrist()) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::EXACT => {
                        if entry.value < alpha {
                            return alpha;
                        }
                        if entry.value >= beta{
                            return beta;
                        }
                        return entry.value;
                    }
                    EntryFlag::BETA => {
                        if !is_pv && beta <= entry.value {
                            return beta;
                        }
                    }
                    EntryFlag::ALPHA => {
                        if !is_pv && alpha >= entry.value {
                            return alpha;
                        }
                    }
                    EntryFlag::NULL => {}
                }
            }
        }

        //Null move pruning
        if !is_pv && do_null {
            if let Some(beta) = self.try_null_move(position, eval, movegen, depth, alpha, beta) {
                return beta;
            }
        }

        let mut best_move = Move::null();
        let mut num_legal_moves = 0;
        let old_alpha = alpha;
        let mut best_score = -isize::MAX;
        let in_check = movegen.in_check(position);
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        for (_move_score, mv) in self.sort_moves_by_score(eval, movegen.get_pseudo_moves(position), position, depth) {
            
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }

            num_legal_moves += 1;
            position.make_move((&mv).to_owned());
            let mut score: isize;
            if num_legal_moves == 1 {
                score = -self.alphabeta(position, eval, movegen,
                                        depth - 1, -beta, -alpha, true);
            } else {

                //Try late move reduction
                if num_legal_moves > 4
                    && mv.get_move_type() == MoveType::Quiet
                    && !is_pv
                    && depth >= 5
                    && !in_check {
                    //Null window search with depth - 2
                    let mut reduced_depth = depth - 2;
                    if num_legal_moves > 10 {
                        reduced_depth = depth - 3;
                    }
                    score = -self.alphabeta(position, eval, movegen,
                                            reduced_depth, -alpha - 1, -alpha, true);
                } else {
                    //Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if score > alpha {
                    //PVS
                    //Null window search
                    score = -self.alphabeta(position, eval, movegen,
                                            depth - 1, -alpha - 1, -alpha, true);
                    //Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta(position, eval, movegen,
                                                depth - 1, -beta, -alpha, true);
                    }
                }

            }

            position.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mv;

                if score > alpha {
                    if score >= beta {
                        //Record new killer moves
                        self.update_killers(depth, (&mv).to_owned());
                        //Beta cutoff, store in transpositon table
                        self.transposition_table.insert(position.get_zobrist(), Entry{
                            key: position.get_zobrist(),
                            flag: EntryFlag::BETA,
                            value: beta,
                            mv,
                            depth,
                            ancient: false
                        });
                        return beta;
                    }
                    alpha = score;

                    //History heuristic
                    self.update_history_heuristic(depth, &mv);
                }
            }
        }

        if num_legal_moves == 0 {
            return if in_check {
                //Checkmate
                -99999
            } else {
                //Stalemate
                0
            };
        }

        if alpha != old_alpha {
            //Alpha improvement, record PV
            self.transposition_table.insert(position.get_zobrist(), Entry{
                key: position.get_zobrist(),
                flag: EntryFlag::EXACT,
                value: best_score,
                mv: (&best_move).to_owned(),
                depth,
                ancient: false
            })
        } else {
            self.transposition_table.insert(position.get_zobrist(), Entry{
                key: position.get_zobrist(),
                flag: EntryFlag::ALPHA,
                value: alpha,
                mv: best_move,
                depth,
                ancient: false
            })
        }
        alpha
    }


    fn quiesce(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth:u8, mut alpha: isize, beta: isize) -> isize {
        let score = eval.evaluate(position, movegen);
        if score >= beta{
            return beta;
        }
        if score > alpha {
            alpha = score;
        }

        // Get only captures, sorted by move ordering heuristics (try the most promising moves first)
        for (_move_score, mv) in self.sort_moves_by_score(eval, movegen.get_capture_moves(position), position, depth) {
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }

            position.make_move((&mv).to_owned());
            let score = -self.quiesce(position, eval, movegen, depth, -beta, -alpha);
            position.unmake_move();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                // best_move = mv;
            }
        }

        alpha
    }


    //Resets heuristics
    fn clear_heuristics(&mut self) {
        for i in 0..self.killer_moves.len() {
            for j in 0..self.killer_moves[i].len() {
                self.killer_moves[i][j] = Move::null();
            }
        }
        for i in 0..self.history_moves.len() {
            for j in 0..self.history_moves[i].len() {
                    self.history_moves[i][j] = 0;
            }
        }
    }


    #[inline]
    fn update_killers(&mut self, depth: u8, mv: Move) {
        if !mv.get_is_capture() {
            if mv != self.killer_moves[depth as usize][0] && mv != self.killer_moves[depth as usize][1] {
                self.killer_moves[depth as usize][1] = self.killer_moves[depth as usize][0];
                self.killer_moves[depth as usize][0] = mv;
            }
        }
    }

    #[inline]
    fn update_history_heuristic(&mut self, depth: u8, mv:&Move) {
        if !mv.get_is_capture() {
            self.history_moves
                [mv.get_from() as usize]
                [mv.get_to() as usize] += depth as u16;
        }
    }

    #[inline]
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, eval: &mut Evaluator, moves: I, position: &mut Position, depth: u8) -> Vec<(usize, Move)> {
        let mut moves_and_score:Vec<(usize, Move)> = moves.map(|mv| {
            (eval.score_move(&self.history_moves, &self.killer_moves[depth as usize], position, &mv), mv)
        }).collect();

        //Assign PV/hash moves to usize::MAX
        if let Some(entry) = self.transposition_table.retrieve(position.get_zobrist()) {
            let best_move = &entry.mv;
            for i in 0..moves_and_score.len() {
                if moves_and_score[i].1 == *best_move {
                    moves_and_score[i] = (usize::MAX, moves_and_score[i].1);
                    break;
                }
            }
        }
        
        // Sort moves by decreasing score
        moves_and_score.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        
        moves_and_score
    }

    #[inline]
    fn try_null_move(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8, _alpha: isize, beta: isize) -> Option<isize> {
        
        if depth > 3 && eval.can_do_null_move(position) && !movegen.in_check(position) {
            position.make_move(Move::null());
            let nscore = -self.alphabeta(position,eval, movegen, depth - 3, -beta, -beta + 1, false);
            position.unmake_move();
            if nscore >= beta {
                return Some(beta);
            }
        }
        None
    }

}
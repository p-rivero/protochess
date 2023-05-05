use crate::MoveInfo;
use crate::types::{Move, Centipawns, Depth};
use crate::searcher::alphabeta::GAME_OVER_SCORE;


/// The result of `get_best_move`. Contains the result (principal variation, score) as well as
/// some stats (number of nodes searched, depth reached, number of the thread that found it).
/// 
/// The user of the function must initialize the struct (for example, with `SearchResult::default()`)
/// and pass its mutable reference as an argument. This is more complex than just returning the struct as the
/// result of the function, but it allows the user to watch the value of the struct as it is being updated
/// by the search in real time, while preserving full search speed (it's not being constantly interrupted).
/// This is particularly useful for infinite searches (like in the Analysis Board GUI).
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchResult {
    /// The principal variation found by the search. Guaranteed to be non-empty.
    pub pv: Vec<Move>,
    /// String equivalent to `pv[0]`. For example, "e2e4".
    pub best_move: MoveInfo,
    /// The score of the position, in centipawns.
    pub score: Centipawns,
    /// The depth reached by the search.
    pub depth: Depth,
    /// The number of nodes searched by this thread at this depth.
    /// Does not include other threads or previous depths in iterative deepening.
    pub nodes_searched: u64,
    /// The number of the thread that found this result, between `0` and `num_threads - 1`.
    #[cfg(feature = "parallel")]
    pub thread_num: u32,
}

// Implement compare for SearchRes so that it can be sorted by depth, then score, then pv length
// ">" means "better" (more depth, higher score, longer pv)
impl std::cmp::PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &SearchResult) -> Option<std::cmp::Ordering> {
        if self.depth > other.depth {
            Some(std::cmp::Ordering::Greater)
        } else if self.depth < other.depth {
            Some(std::cmp::Ordering::Less)
        } else if self.score > other.score {
            Some(std::cmp::Ordering::Greater)
        } else if self.score < other.score {
            Some(std::cmp::Ordering::Less)
        } else if self.pv.len() > other.pv.len() {
            Some(std::cmp::Ordering::Greater)
        } else if self.pv.len() < other.pv.len() {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

// Writes a line like "T6 Depth 3 Score: MATE -3 nodes: 123456 | PV: e2e4 e7e5"
impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        const MAX_PV_LEN: i32 = 200;
        
        #[cfg(feature = "parallel")]
        write!(f, "T{:<2} ", self.thread_num)?;
        
        write!(f, "Depth {:<2} Score: ", self.depth)?;
        let diff = -(self.score.abs() + GAME_OVER_SCORE);
        if diff < MAX_PV_LEN {
            write!(f, "MATE ")?;
            if self.score < 0 { write!(f, "-")? };
            write!(f, "{}", (diff+1) / 2)?;
        } else {
            write!(f, "cp {:<4}", self.score)?;
        }
        
        write!(f, " nodes: {:<8} | PV: ", self.nodes_searched)?;
        for m in &self.pv {
            write!(f, "{m} ")?;
        }
        Ok(())
    }
}


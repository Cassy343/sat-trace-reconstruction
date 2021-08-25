use bitvec::{prelude::BitVec, slice::BitSlice};
use std::{
    borrow::Borrow,
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Term {
    id: u32,
    negated: bool,
}

impl Term {
    pub fn new(id: u32, negated: bool) -> Self {
        Self { id, negated }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}x{}",
            if self.negated { "\u{AC}" } else { "" },
            self.id
        )
    }
}

// We can derive `PartialEq` and `Eq` because the `terms` vec must always be sorted
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Conjunction {
    terms: Vec<Term>,
}

impl Conjunction {
    pub fn new(terms: Vec<Term>) -> Self {
        Self { terms }
    }

    pub fn merge(&self, rhs: &Self) -> Option<Self> {
        let lhs = self;
        let mut terms = Vec::with_capacity(usize::max(self.terms.len(), rhs.terms.len()));

        let mut lhs_index = 0;
        let mut rhs_index = 0;

        loop {
            let lhs_term = match lhs.terms.get(lhs_index) {
                Some(&term) => term,
                None => {
                    terms.extend(rhs.terms[rhs_index..].iter().copied());
                    return Some(Self { terms });
                }
            };
            let rhs_term = match rhs.terms.get(rhs_index) {
                Some(&term) => term,
                None => {
                    terms.extend(lhs.terms[lhs_index..].iter().copied());
                    return Some(Self { terms });
                }
            };

            if rhs_term.id < lhs_term.id {
                terms.push(rhs_term);
                rhs_index += 1;
            } else if rhs_term.id > lhs_term.id {
                terms.push(lhs_term);
                lhs_index += 1;
            } else {
                if lhs_term.negated ^ rhs_term.negated {
                    return None;
                }

                terms.push(lhs_term);
                rhs_index += 1;
                lhs_index += 1;
            }
        }
    }
}

impl Display for Conjunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.terms.len() {
            0 => write!(f, "()"),
            1 => write!(f, "({})", self.terms[0]),
            _ => {
                write!(f, "({}", self.terms[0])?;
                for i in 1..self.terms.len() {
                    write!(f, " \u{2227} {}", self.terms[i])?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug)]
pub struct TraceDisjunction {
    clauses: HashSet<Conjunction>,
}

impl TraceDisjunction {
    pub fn from_trace(trace: &BitSlice, message_len: usize) -> Self {
        Self {
            clauses: Self::clauses_from_trace(trace, message_len).collect(),
        }
    }

    pub fn clauses_from_trace(trace: &BitSlice, message_len: usize) -> TraceClauseIter<'_> {
        TraceClauseIter::new(trace, message_len)
    }

    pub fn and<I>(&mut self, other: I)
    where
        I: Iterator,
        I::Item: Borrow<Conjunction>,
    {
        let mut clauses = HashSet::new();

        for rhs_clause in other {
            for lhs_clause in self.clauses.iter() {
                if let Some(clause) = lhs_clause.merge(rhs_clause.borrow()) {
                    clauses.insert(clause);
                }
            }
        }

        self.clauses = clauses;
    }

    pub fn message(&self) -> Option<BitVec> {
        if self.clauses.len() != 1 {
            return None;
        }

        Some(
            self.clauses
                .iter()
                .next()
                .unwrap()
                .terms
                .iter()
                .map(|term| !term.negated)
                .collect(),
        )
    }
}

impl Display for TraceDisjunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.clauses.len() {
            0 => write!(f, "[]"),
            1 => write!(f, "[\n    {}\n]", self.clauses.iter().next().unwrap()),
            _ => {
                let mut iter = self.clauses.iter();

                write!(f, "[\n    {}", iter.next().unwrap())?;
                for conj in iter {
                    write!(f, " \u{2228}\n    {}", conj)?;
                }
                write!(f, "\n]")
            }
        }
    }
}

pub struct TraceClauseIter<'a> {
    trace: &'a BitSlice,
    message_len: usize,
    combination: Option<Vec<usize>>,
}

impl<'a> TraceClauseIter<'a> {
    fn new(trace: &'a BitSlice, message_len: usize) -> Self {
        Self {
            trace,
            message_len,
            combination: Some((0..trace.len()).collect()),
        }
    }
}

impl<'a> Iterator for TraceClauseIter<'a> {
    type Item = Conjunction;

    fn next(&mut self) -> Option<Self::Item> {
        let combination = self.combination.as_deref()?;

        let terms = combination
            .iter()
            .enumerate()
            .map(|(index, &id)| Term::new(id as u32, !self.trace[index]))
            .collect();

        let clause = Conjunction::new(terms);

        if !next_combination(self.combination.as_mut().unwrap(), self.message_len) {
            self.combination = None;
        }

        Some(clause)
    }
}

/// Takes an input of numbers between 0 (inclusive) and bound (exclusive) in strictly increasing
/// order and computes the next combination of non-duplicated numbers between 0 and bound.
fn next_combination(set: &mut [usize], bound: usize) -> bool {
    let len = set.len();

    // Empty set, no combinations
    if len == 0 {
        return false;
    }

    // Increment the final index
    set[len - 1] += 1;

    // If we hit the bound, find the highest-indexed number we can increase, and then count up from
    // that number by one for the remaining numbers
    if set[len - 1] == bound {
        // If the set is length one then we just need to count up to bound
        if len == 1 {
            return false;
        }

        // Find the highest index we can increase by one without collision
        let mut offset = 2;
        loop {
            // We can increase the number without exceeding the bound of violating the invariant
            // that the set is strictly increasing
            if set[len - offset] < bound - offset {
                break;
            }

            // We could not find a number to increase, all combinations have been taken
            if offset == len {
                return false;
            }

            offset += 1;
        }

        set[len - offset] += 1;
        let mut value = set[len - offset];

        // Set the remaining numbers to increase by one after the number we just incremented
        for i in len - offset + 1..len {
            value += 1;
            set[i] = value;
        }
    }

    true
}

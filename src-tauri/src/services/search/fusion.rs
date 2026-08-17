use std::collections::HashMap;

pub const K: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Metadata,
    Fts,
    Vector,
}

impl EngineKind {
    pub fn weight(&self) -> f64 {
        match self {
            EngineKind::Metadata => 2.0,
            EngineKind::Fts => 2.0,
            EngineKind::Vector => 1.0,
        }
    }
}

impl std::fmt::Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineKind::Metadata => write!(f, "metadata"),
            EngineKind::Fts => write!(f, "fts"),
            EngineKind::Vector => write!(f, "vector"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedFile {
    pub file_id: i64,
}

#[derive(Debug, Clone)]
pub struct EngineVote {
    pub kind: EngineKind,
    pub ranked: Vec<RankedFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FusedScore {
    pub file_id: i64,
    pub score: f64,
}

/// Fuses per-engine rankings via Reciprocal Rank Fusion: `weight / (K + rank)` per vote.
pub fn fuse(votes: &[EngineVote]) -> Vec<FusedScore> {
    let mut acc: HashMap<i64, f64> = HashMap::new();

    for vote in votes {
        let weight = vote.kind.weight();
        for (position, ranked) in vote.ranked.iter().enumerate() {
            let rank = (position + 1) as f64;
            *acc.entry(ranked.file_id).or_insert(0.0) += weight / (K + rank);
        }
    }

    let mut fused: Vec<FusedScore> = acc
        .into_iter()
        .map(|(file_id, score)| FusedScore { file_id, score })
        .collect();

    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.file_id.cmp(&b.file_id))
    });

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vote(kind: EngineKind, ids: &[i64]) -> EngineVote {
        EngineVote {
            kind,
            ranked: ids.iter().map(|id| RankedFile { file_id: *id }).collect(),
        }
    }

    #[test]
    fn empty_votes_produce_empty_fusion() {
        assert!(fuse(&[]).is_empty());
    }

    #[test]
    fn single_engine_keeps_its_order() {
        let fused = fuse(&[vote(EngineKind::Metadata, &[7, 3, 9])]);
        let ids: Vec<i64> = fused.iter().map(|f| f.file_id).collect();
        assert_eq!(ids, vec![7, 3, 9]);
    }

    #[test]
    fn consensus_accumulates_across_engines() {
        let metadata = vote(EngineKind::Metadata, &[1, 2, 3, 4]);
        let fts = vote(EngineKind::Fts, &[2, 1]);
        let fused = fuse(&[metadata, fts]);

        assert!(fused.len() == 4);
        assert_eq!(fused[0].file_id, 1);
        assert_eq!(fused[1].file_id, 2);
    }

    #[test]
    fn score_matches_rrf_formula() {
        let fused = fuse(&[vote(EngineKind::Metadata, &[5])]);
        let expected = EngineKind::Metadata.weight() / (K + 1.0);
        assert!((fused[0].score - expected).abs() < 1e-9);
    }

    #[test]
    fn weight_scales_contribution() {
        let fused = fuse(&[vote(EngineKind::Vector, &[5])]);
        let expected = EngineKind::Vector.weight() / (K + 1.0);
        assert!((fused[0].score - expected).abs() < 1e-9);
    }

    #[test]
    fn ties_break_by_file_id() {
        let fused = fuse(&[
            vote(EngineKind::Metadata, &[1]),
            vote(EngineKind::Fts, &[2]),
        ]);
        assert_eq!(fused[0].file_id, 1);
        assert_eq!(fused[1].file_id, 2);
    }
}

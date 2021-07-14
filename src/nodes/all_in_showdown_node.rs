use crate::{cfr::traversal::Traversal, nodes::showdown_node::showdown, ranges::{combination::Board, utility::check_card_overlap}};

use super::node::CfrNode;

pub struct AllInShowdownNode {
    win_utility: f64,
    street: u8
}

impl CfrNode for AllInShowdownNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
        self.all_in_showdown_node_utility(traversal, op_reach_prob, board)
    }

    fn best_response(
        &self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
        self.all_in_showdown_node_utility(traversal, op_reach_prob, board)
    }
}

impl AllInShowdownNode {
    pub fn new(pot_size: f64, street: u8) -> Self {
        Self {
            win_utility: pot_size / 2.0,
            street
        }
    }
    /*
    std::vector<float> AllInShowdownNode::allInShowdownNodeUtility(Traversal* traversal, std::vector<float>& opponentReachProb, Board board)
{
    std::vector<float> utility(traversal->rm->getNumCombinations(traversal->traverser, board));

    int traverser = traversal->traverser;

    if (street == 1) {
        for (uint8_t turn; turn < 52; turn++) {
            if (!checkOverlap(turn, board)) {
                auto nextBoard = board;
                nextBoard[3] = turn;
                auto turnProbs = traversal->rm->getReachProbs(traverser, nextBoard, opponentReachProb);
                std::vector<float> turnUtility(turnProbs.size());

                for (uint8_t river = turn + 1; river < 52; river++) {
                    if (!checkOverlap(river, nextBoard)) {
                        nextBoard[4] = river;
                        auto riverProbs = traversal->rm->getReachProbs(traverser, nextBoard, turnProbs);
                        auto riverHands = traversal->rm->getHands(traverser, nextBoard);
                        auto riverUtility = ShowdownNode::Showdown(riverHands, riverProbs, winUtility);

                        traversal->rm->mapUtilityBackwards(traverser, nextBoard, riverUtility, turnUtility);
                    }
                }
                nextBoard[4] = 52;
                traversal->rm->mapUtilityBackwards(traverser, nextBoard, turnUtility, utility);
            }
        }
        // each value above is equivalent to two runouts, 44 * 45 = 1980 possible runouts thus we arrive at 990.0
        for (auto& val : utility) {
            val /= 990.0;
        }
    }
    else {
        for (uint8_t river = 0; river < 52; river++) {
            if (!checkOverlap(river, board)) {
                auto nextBoard = board;
                nextBoard[4] = river;
                auto riverProbs = traversal->rm->getReachProbs(traverser, nextBoard, opponentReachProb);
                auto riverHands = traversal->rm->getHands(traverser, nextBoard);
                auto riverUtility = ShowdownNode::Showdown(riverHands, riverProbs, winUtility);

                traversal->rm->mapUtilityBackwards(traverser, nextBoard, riverUtility, utility);
            }
        }
        for (auto& val : utility) {
            val /= 44.0;
        }
    }

    return utility;
} 
    */
    fn all_in_showdown_node_utility(&self, traversal: &Traversal, op_reach_probs: &Vec<f64>, board: &Board) -> Vec<f64> {
        let mut utility = vec![0.0; traversal.get_num_hands_for_traverser(board)];

        if self.street == 1 {

        } else {
            for river in 0..52 {
                if !check_card_overlap(river, board) {
                    let mut next_board = *board;
                    next_board[4] = river;
                    let river_probs = traversal.get_next_reach_probs(&next_board, op_reach_probs);
                    let hands = traversal.get_range_for_opponent(board);
                    let river_utility = showdown(hands, &river_probs, self.win_utility);

                    traversal.map_utility_backwards(&next_board, &river_utility, &mut utility);
                }
            }
        }

        utility
    }
}
